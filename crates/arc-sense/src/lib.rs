//! SENSE-1 (docs/SENSE-1-RECEIVE.md): the Arc's sensor neuron. Reads the existing `xe` driver's
//! sysfs sensors — read only, nothing is written — and stamps each reading with CLOCK_MONOTONIC, the
//! same clock Jesse's `sdr-stream` stamps the dongle's frames with. The paths are the orchestrator's.
//! Mechanism only: no reading is interpreted here.

use std::path::PathBuf;

#[repr(C)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

extern "C" {
    fn clock_gettime(clock: i32, tp: *mut Timespec) -> i32;
}

/// CLOCK_MONOTONIC in nanoseconds.
pub fn monotonic_ns() -> u64 {
    let mut t = Timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes one Timespec into a valid pointer.
    let r = unsafe { clock_gettime(1, &mut t) };
    assert_eq!(r, 0, "clock_gettime(CLOCK_MONOTONIC) failed");
    t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64
}

/// Where the driver publishes each reading.
#[derive(Debug, Clone)]
pub struct Paths {
    pub energy: PathBuf,
    pub pkg_temp: PathBuf,
    pub vram_temp: PathBuf,
    pub act_freq: PathBuf,
    /// (name, path) of each throttle-reason flag.
    pub throttle: Vec<(String, PathBuf)>,
}

fn number(p: &PathBuf) -> std::io::Result<i64> {
    std::fs::read_to_string(p)?.trim().parse::<i64>().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// One energy-counter reading: (t ns, µJ).
pub fn energy(p: &Paths) -> std::io::Result<(u64, u64)> {
    let e = number(&p.energy)? as u64;
    Ok((monotonic_ns(), e))
}

/// The slower readings.
#[derive(Debug, Clone, Copy)]
pub struct Slow {
    pub t_ns: u64,
    pub pkg_mc: i64,
    pub vram_mc: i64,
    pub freq_mhz: i64,
    /// Bit k set = throttle flag k (in `Paths::throttle` order) is raised.
    pub throttle: u32,
}

pub fn slow(p: &Paths) -> std::io::Result<Slow> {
    let mut throttle = 0;
    for (k, (_, path)) in p.throttle.iter().enumerate() {
        if number(path)? != 0 {
            throttle |= 1 << k;
        }
    }
    Ok(Slow { t_ns: monotonic_ns(), pkg_mc: number(&p.pkg_temp)?, vram_mc: number(&p.vram_temp)?, freq_mhz: number(&p.act_freq)?, throttle })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_moves_forward() {
        let a = monotonic_ns();
        let b = monotonic_ns();
        assert!(b >= a && a > 0);
    }
}
