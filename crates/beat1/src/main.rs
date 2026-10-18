//! BEAT-1 orchestrator (docs/BEAT-1-HEARTBEAT.md): the heartbeat. The Arc transmits a known bit
//! pattern as load (Vulkan buffer fills over the existing driver, `arc-vk-fill`); the receiver is
//! SENSE-1's stack (the Arc's existing sensors, the dongle through Jesse's unchanged `sdr-stream`),
//! with the FFT band-power neuron; the Arc's and the dongle's readings also feed the thermal base
//! through ports, four arms in lock-step, and the probe's heat is decoded too. Usage:
//! beat1 SDR_STREAM_BINARY OUT_DIR. Every setting is here.

use arc_sense::{energy, monotonic_ns, slow, Paths, Slow};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thermo_bath::{damp, exchange};
use thermo_host::Medium;

const STEP_NS: u64 = 10_000_000;
const MS: u64 = 1_000_000;
const SEC: u64 = 1_000_000_000;
const CENTER: u32 = 93_000_000;
const CHANNEL: u32 = 93_300_000;
const RATE: u32 = 2_048_000;
const STREAM_SECONDS: u64 = 126;
const THROTTLE: [&str; 8] = ["pl1", "pl2", "pl4", "prochot", "ratl", "thermal", "vr_tdc", "vr_thermalert"];
const SLOW_B: [u8; 10] = [1, 1, 0, 0, 1, 0, 1, 1, 0, 1];

fn arc_paths() -> Paths {
    let hw = std::fs::read_dir("/sys/class/hwmon").unwrap().map(|e| e.unwrap().path()).find(|p| std::fs::read_to_string(p.join("name")).map(|n| n.trim() == "xe").unwrap_or(false)).expect("no xe hwmon");
    let freq = std::fs::canonicalize(hw.join("device")).unwrap().join("tile0/gt0/freq0");
    Paths {
        energy: hw.join("energy2_input"),
        pkg_temp: hw.join("temp2_input"),
        vram_temp: hw.join("temp3_input"),
        act_freq: freq.join("act_freq"),
        throttle: THROTTLE.iter().map(|n| (n.to_string(), freq.join(format!("throttle/reason_{n}")))).collect(),
    }
}

/// The 64-bit pattern: SplitMix64 from 0xBEA7, its first 8 bits forced to the preamble 10101010.
fn pattern() -> Vec<u8> {
    let mut z = 0xBEA7u64.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (0..64).map(|k| if k < 8 { (1 - k % 2) as u8 } else { (z >> k & 1) as u8 }).collect()
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    part: char,
    index: usize,
    bit: u8,
    start: u64,
    end: u64,
    transmit: bool,
}

fn transport(m: &mut Medium) {
    for f in m.fields.iter_mut() {
        for _ in 0..4 {
            exchange(f, 0.0, 0.5, &|_| 0.0, &|_, _| 0.0);
            damp(f, 0.0, &|_| 0.1, &|_| 0.0, &|_| 0.0);
        }
    }
}

fn latest(series: &[(u64, f64)], t: u64) -> Option<f64> {
    let i = series.partition_point(|x| x.0 <= t);
    (i > 0).then(|| series[i - 1].1)
}

fn mean_in(series: &[(u64, f64)], from: u64, to: u64) -> f64 {
    let v: Vec<f64> = series.iter().filter(|x| x.0 > from && x.0 <= to).map(|x| x.1).collect();
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

/// Decode slots by the preamble's midpoint; (accuracy over the scored bits, mean 1 − mean 0).
fn decode(values: &[f64], bits: &[u8]) -> (f64, f64) {
    let m = |b: u8, r: std::ops::Range<usize>| {
        let v: Vec<f64> = r.filter(|&k| bits[k] == b).map(|k| values[k]).collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    let thr = (m(1, 0..8) + m(0, 0..8)) / 2.0;
    let ok = (8..bits.len()).filter(|&k| (values[k] > thr) == (bits[k] == 1)).count();
    (ok as f64 / (bits.len() - 8) as f64, m(1, 8..bits.len()) - m(0, 8..bits.len()))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (bin, out) = (PathBuf::from(&args[1]), PathBuf::from(&args[2]));
    std::fs::create_dir_all(&out).unwrap();
    let paths = arc_paths();
    let bits = pattern();
    println!("BEAT-1: the heartbeat. pattern {}", bits.iter().map(|b| b.to_string()).collect::<String>());
    let mut child = Command::new(&bin).args(["/dev/swradio0", &CENTER.to_string(), &CHANNEL.to_string(), &RATE.to_string(), &STREAM_SECONDS.to_string()]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("sdr-stream");
    let t_stream = monotonic_ns();
    let mut stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut frames = Vec::new();
        while let Some(f) = sdr_frames::read(&mut stdout).expect("TDR1 stream") {
            frames.push(f);
        }
        frames
    });
    // The schedule, in absolute CLOCK_MONOTONIC time.
    let t0 = t_stream + 4 * SEC;
    let mut slots = Vec::new();
    for (k, &b) in bits.iter().enumerate() {
        slots.push(Slot { part: 'A', index: k, bit: b, start: t0 + k as u64 * 250 * MS, end: t0 + (k as u64 + 1) * 250 * MS, transmit: b == 1 });
    }
    let tc = t0 + 18 * SEC;
    for (k, &b) in bits.iter().enumerate() {
        slots.push(Slot { part: 'C', index: k, bit: b, start: tc + k as u64 * 250 * MS, end: tc + (k as u64 + 1) * 250 * MS, transmit: false });
    }
    let tb = t0 + 36 * SEC;
    for (k, &b) in SLOW_B.iter().enumerate() {
        slots.push(Slot { part: 'B', index: k, bit: b, start: tb + k as u64 * 8 * SEC, end: tb + (k as u64 + 1) * 8 * SEC, transmit: b == 1 });
    }
    let (tx_send, tx_recv) = std::sync::mpsc::channel();
    let tx_slots = slots.clone();
    let transmitter = std::thread::spawn(move || {
        let mut t = match arc_vk_fill::Transmitter::open(0x8086, "Arc", 256 << 20, 4) {
            Ok(t) => t,
            Err(e) => {
                tx_send.send(Err(e)).unwrap();
                return Vec::new();
            }
        };
        tx_send.send(Ok((t.name.clone(), t.vendor, t.device_id, t.check_ok, t.check_words))).unwrap();
        let mut done = Vec::new();
        for s in tx_slots.iter().filter(|s| s.transmit) {
            while monotonic_ns() < s.start {
                std::thread::sleep(std::time::Duration::from_micros(200));
            }
            let a = monotonic_ns();
            let n = t.run_until(s.end).expect("fenced submission failed");
            done.push((s.part, s.index, a, monotonic_ns(), n));
        }
        done
    });
    let h0_open = tx_recv.recv().unwrap();
    println!("H0 adapter: {h0_open:?}");
    let (mut en, mut sl): (Vec<(u64, u64)>, Vec<Slow>) = (Vec::new(), Vec::new());
    let mut k = 0u64;
    while !reader.is_finished() {
        en.push(energy(&paths).unwrap());
        if k % 100 == 0 {
            sl.push(slow(&paths).unwrap());
        }
        k += 1;
        std::thread::sleep(std::time::Duration::from_micros(1000));
    }
    let tx_done = transmitter.join().unwrap();
    let frames = reader.join().unwrap();
    drop(child.stdin.take());
    let status = child.wait().unwrap();
    let mut err = String::new();
    std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut err).unwrap();
    std::fs::write(out.join("sdr-stream-stderr.txt"), &err).unwrap();
    println!("sdr-stream exit {status}; {} frames; Arc {} energy / {} slow samples; {} transmit slots done", frames.len(), en.len(), sl.len(), tx_done.len());
    // Evidence.
    let mut w = std::fs::File::create(out.join("arc-energy.csv")).unwrap();
    writeln!(w, "t_ns,energy_uj").unwrap();
    for (t, e) in &en {
        writeln!(w, "{t},{e}").unwrap();
    }
    let mut w = std::fs::File::create(out.join("arc-slow.csv")).unwrap();
    writeln!(w, "t_ns,pkg_mC,vram_mC,act_freq_MHz,throttle_bits({})", THROTTLE.join("|")).unwrap();
    for s in &sl {
        writeln!(w, "{},{},{},{},{}", s.t_ns, s.pkg_mc, s.vram_mc, s.freq_mhz, s.throttle).unwrap();
    }
    let mut w = std::fs::File::create(out.join("schedule.csv")).unwrap();
    writeln!(w, "part,index,bit,start_ns,end_ns,transmit,actual_start_ns,actual_end_ns,submissions").unwrap();
    for s in &slots {
        let d = tx_done.iter().find(|d| d.0 == s.part && d.1 == s.index);
        writeln!(w, "{},{},{},{},{},{},{},{},{}", s.part, s.index, s.bit, s.start, s.end, s.transmit, d.map(|d| d.2).unwrap_or(0), d.map(|d| d.3).unwrap_or(0), d.map(|d| d.4).unwrap_or(0)).unwrap();
    }
    let raw: Vec<u8> = frames.iter().filter(|f| f.kind == 1).flat_map(|f| f.data.iter().copied()).take(RATE as usize / 2).collect();
    std::fs::write(out.join("sdr-first-0.25s.cu8"), &raw).unwrap();
    // H0.
    let h0 = matches!(&h0_open, Ok((name, 0x8086, _, true, _)) if name.contains("Arc")) && tx_done.iter().all(|d| d.4 > 0);
    // H1, H2 (power channel).
    let watts = arc_flux::power(&en, STEP_NS);
    let part = |p: char| -> Vec<Slot> { slots.iter().filter(|s| s.part == p).copied().collect() };
    let pa: Vec<f64> = part('A').iter().map(|s| mean_in(&watts, s.start + 50 * MS, s.end)).collect();
    let (acc_a, diff_a) = decode(&pa, &bits);
    let pc: Vec<f64> = part('C').iter().map(|s| mean_in(&watts, s.start + 50 * MS, s.end)).collect();
    let (_, diff_c) = decode(&pc, &bits);
    println!("H1 power channel: {:.1}% of 56 bits; mean 1-bit minus 0-bit power {diff_a:.2} W (1-bits {:.2} W)", acc_a * 100.0, pa.iter().zip(&bits).filter(|x| *x.1 == 1).map(|x| x.0).sum::<f64>() / 32.0);
    println!("H2 disconnected control: mean power over the clock's 1-slots minus 0-slots {diff_c:+.3} W");
    // H3 (temperature channel).
    let temp_at = |t: u64, after: bool| -> f64 {
        let i = sl.partition_point(|s| s.t_ns <= t);
        let s = if after { sl[i.min(sl.len() - 1)] } else { sl[i.max(1) - 1] };
        s.pkg_mc as f64 / 1000.0
    };
    let rises: Vec<f64> = part('B').iter().map(|s| temp_at(s.end, false) - temp_at(s.start, true)).collect();
    let acc_b = rises.iter().zip(SLOW_B.iter()).filter(|(r, &b)| (**r > 0.0) == (b == 1)).count();
    println!("H3 temperature channel: rises per 8 s bit {:?} (bits {:?}); {acc_b} of 10 decoded; pkg range {:.1}-{:.1} C", rises, SLOW_B, sl.iter().map(|s| s.pkg_mc).min().unwrap() as f64 / 1000.0, sl.iter().map(|s| s.pkg_mc).max().unwrap() as f64 / 1000.0);
    let max_one_rise = rises.iter().zip(SLOW_B.iter()).filter(|x| *x.1 == 1).map(|x| *x.0).fold(f64::MIN, f64::max);
    // H4 (the die's law).
    let e_at = |x: u64| en[en.partition_point(|p| p.0 <= x).max(1) - 1];
    let pts: Vec<(f64, f64, f64)> = (1..)
        .map(|s| en[0].0 + s * SEC)
        .take_while(|&tt| tt <= en.last().unwrap().0)
        .map(|tt| {
            let (a, b) = (e_at(tt - SEC), e_at(tt));
            ((tt - en[0].0) as f64 * 1e-9, temp_at(tt, false), (b.1 - a.1) as f64 * 1e-6 / ((b.0 - a.0) as f64 * 1e-9))
        })
        .collect();
    let p_range = pts.iter().map(|p| p.2).fold(f64::MIN, f64::max) - pts.iter().map(|p| p.2).fold(f64::MAX, f64::min);
    let fit = arc_rc::fit(&pts);
    println!("H4 die law: power range {p_range:.2} W over {} points; fit {fit:?}", pts.len());
    // H6 (radio), per frame.
    let data: Vec<&sdr_frames::Frame> = frames.iter().filter(|f| f.kind == 1).collect();
    let bands = [((CHANNEL - CENTER) as f64, 100_000.0), (-500_000.0, 100_000.0)];
    let fp: Vec<(u64, f64, f64)> = data
        .iter()
        .map(|f| {
            let (p, s) = sdr_band_fft::band_powers(&f.data, 4096, RATE as f64, &bands);
            (f.stamp_ns, p[0] / s.max(1) as f64, p[1] / s.max(1) as f64)
        })
        .collect();
    let contrast = 10.0 * (fp.iter().map(|x| x.1).sum::<f64>() / fp.iter().map(|x| x.2).sum::<f64>()).log10();
    let chan: Vec<(u64, f64)> = fp.iter().map(|x| (x.0, x.1)).collect();
    let ca: Vec<f64> = part('A').iter().map(|s| mean_in(&chan, s.start, s.end)).collect();
    let mean_by = |v: &[f64], b: u8| v.iter().zip(&bits).filter(|x| *x.1 == b).map(|x| *x.0).sum::<f64>() / bits.iter().filter(|&&x| x == b).count() as f64;
    let rf_db = 10.0 * (mean_by(&ca, 1) / mean_by(&ca, 0)).log10();
    let gaps = frames.iter().map(|f| f.gaps).max().unwrap_or(0);
    println!("H6 radio: FFT contrast {contrast:.2} dB; channel power 1-bits vs 0-bits {rf_db:+.3} dB; stream gaps {gaps}");
    let mut w = std::fs::File::create(out.join("sdr-frame-power.csv")).unwrap();
    writeln!(w, "stamp_ns,channel_power,offchannel_power").unwrap();
    for x in &fp {
        writeln!(w, "{},{:e},{:e}", x.0, x.1, x.2).unwrap();
    }
    // H5: the thermal base, four arms in lock-step.
    let t_feed = en[0].0.max(data[0].stamp_ns) + 100 * MS;
    let t_end = en.last().unwrap().0.min(data.last().unwrap().stamp_ns);
    let cal = |s: &[(u64, f64)]| {
        let v: Vec<f64> = s.iter().filter(|x| x.0 >= t_feed && x.0 < t_feed + 3 * SEC).map(|x| x.1).collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    let (ref_arc, ref_sdr) = (cal(&watts), cal(&chan));
    let l = 16;
    let site = |x: usize| x + l * 8 + l * l * 8;
    let (sa, sd, sp) = (site(6), site(10), site(8));
    let mut arms: Vec<Medium> = (0..4).map(|_| Medium::new(&[l; 3], 1)).collect();
    let (mut max_dev, mut max_mag, mut plain_zero) = (0.0f64, 0.0f64, true);
    let mut probe_arc: Vec<(u64, f64)> = Vec::new();
    let mut w = std::fs::File::create(out.join("probe.csv")).unwrap();
    writeln!(w, "t_ns,arc_W,sdr_power,probe_plain,probe_arc,probe_sdr,probe_both").unwrap();
    let mut t = t_feed;
    while t <= t_end {
        let (a, d) = (latest(&watts, t).unwrap_or(ref_arc), latest(&chan, t).unwrap_or(ref_sdr));
        for (k, m) in arms.iter_mut().enumerate() {
            if k == 1 || k == 3 {
                m.inject(sa, 0, &[a / ref_arc], 1.0);
            }
            if k == 2 || k == 3 {
                m.inject(sd, 0, &[d / ref_sdr], 1.0);
            }
        }
        let v: Vec<f64> = arms.iter().map(|m| m.sense(sp, 0, 1)[0]).collect();
        writeln!(w, "{t},{a:.4},{d:.6e},{},{},{},{}", v[0], v[1], v[2], v[3]).unwrap();
        probe_arc.push((t, v[1]));
        plain_zero &= arms[0].fields[0].e.iter().all(|x| x.to_bits() == 0);
        for i in 0..l * l * l {
            let (x, y, z) = (arms[1].fields[0].e[i], arms[2].fields[0].e[i], arms[3].fields[0].e[i]);
            max_dev = max_dev.max((z - (x + y)).abs());
            max_mag = max_mag.max(z.abs());
        }
        for m in arms.iter_mut() {
            transport(m);
        }
        t += STEP_NS;
    }
    let pp: Vec<f64> = part('A').iter().map(|s| mean_in(&probe_arc, s.start + 50 * MS, s.end)).collect();
    let (acc_p, _) = decode(&pp, &bits);
    println!("H5 thermal base: the probe's heat decodes {:.1}% of 56 bits; together - (Arc + dongle) max {max_dev:.2e} of {max_mag:.3}; plain exactly zero {plain_zero}", acc_p * 100.0);
    // H7.
    let flags: Vec<u32> = sl.iter().map(|s| s.throttle).collect();
    let ev = arc_boil::events(&flags, THROTTLE.len());
    println!("H7 boiling: events per flag {ev:?} ({})", THROTTLE.join(","));
    let h4 = p_range >= 3.0 && fit.map(|f| f.r2 >= 0.5 && (15.0..=40.0).contains(&f.t_amb_c)).unwrap_or(false);
    let v = [
        ("H0 the adapter runs (Arc found, readback exact, every submission fenced)", h0),
        ("H1 the heartbeat is received (>= 95%, >= 3 W)", acc_a >= 0.95 && diff_a >= 3.0),
        ("H2 no signal when disconnected (< 0.5 W)", diff_c.abs() < 0.5),
        ("H3 the heartbeat is in real temperature (>= 1 C rise, >= 8 of 10)", max_one_rise >= 1.0 && acc_b >= 8),
        ("H4 the die's law identifiable (R^2 >= 0.5, T_amb 15-40 C)", h4),
        ("H5 the heartbeat crosses the thermal base (>= 95%; sources add)", acc_p >= 0.95 && max_dev <= 1e-12 * max_mag && plain_zero),
        ("H6 radio: FFT contrast >= 10 dB; GPU load leaves 93.3 MHz within 0.5 dB", contrast >= 10.0 && rf_db.abs() <= 0.5),
        ("H7 no boiling", ev.iter().all(|&e| e == 0)),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
