//! SENSE-1 (docs/SENSE-1-RECEIVE.md): read the TDR1 frames Jesse's `sdr-stream` writes — a 64-byte
//! little-endian header (magic "TDR1", version 1, kind, epoch, sequence, flags, rate, center,
//! channel, byte count, CLOCK_MONOTONIC stamp, gaps, discarded), then that many CU8 bytes. The
//! layout and its checks follow the thermal-antenna observer's `wire.rs`.

use std::io::Read;

#[derive(Debug, Clone)]
pub struct Frame {
    /// 1 = samples, 2 = a gap marker.
    pub kind: u32,
    pub sequence: u32,
    pub rate: u32,
    pub center: u32,
    pub channel: u32,
    pub stamp_ns: u64,
    pub gaps: u64,
    pub discarded: u64,
    pub data: Vec<u8>,
}

fn u32le(h: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([h[i], h[i + 1], h[i + 2], h[i + 3]])
}
fn u64le(h: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(h[i..i + 8].try_into().unwrap())
}

/// The next frame, or None at the end of the stream; fails closed on a malformed header.
pub fn read(r: &mut impl Read) -> std::io::Result<Option<Frame>> {
    let bad = |m: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, m.to_owned());
    let mut h = [0u8; 64];
    if r.read(&mut h[..1])? == 0 {
        return Ok(None);
    }
    r.read_exact(&mut h[1..])?;
    if u32le(&h, 0) != 0x3152_4454 || u32le(&h, 4) != 1 {
        return Err(bad("not a TDR1 v1 stream"));
    }
    let kind = u32le(&h, 8);
    let bytes = u32le(&h, 36) as usize;
    if !matches!(kind, 1 | 2) || bytes > 65536 || bytes % 2 != 0 || (kind == 1 && bytes == 0) || (kind == 2 && bytes != 0) {
        return Err(bad("invalid frame length or kind"));
    }
    let mut data = vec![0; bytes];
    r.read_exact(&mut data)?;
    Ok(Some(Frame { kind, sequence: u32le(&h, 16), rate: u32le(&h, 24), center: u32le(&h, 28), channel: u32le(&h, 32), stamp_ns: u64le(&h, 40), gaps: u64le(&h, 48), discarded: u64le(&h, 56), data }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_frame_reads_back() {
        let mut h = [0u8; 64];
        let put = |h: &mut [u8; 64], i: usize, v: u32| h[i..i + 4].copy_from_slice(&v.to_le_bytes());
        put(&mut h, 0, 0x3152_4454);
        put(&mut h, 4, 1);
        put(&mut h, 8, 1);
        put(&mut h, 24, 2_048_000);
        put(&mut h, 36, 4);
        h[40..48].copy_from_slice(&123u64.to_le_bytes());
        let mut bytes = h.to_vec();
        bytes.extend([1, 2, 3, 4]);
        let f = read(&mut bytes.as_slice()).unwrap().unwrap();
        assert_eq!((f.kind, f.rate, f.stamp_ns, f.data), (1, 2_048_000, 123, vec![1, 2, 3, 4]));
    }
}
