//! SENSE-1 orchestrator (docs/SENSE-1-RECEIVE.md): receive only. Records the Arc's existing sensors
//! (read only) and the dongle through Jesse's unchanged `sdr-stream`, both on CLOCK_MONOTONIC;
//! perceives power, the die's thermal law, boiling events and band power; feeds both sources into
//! the thermal base (T5's medium, one heat field) through calibrated ports; runs the four arms in
//! lock-step and the checks. Usage: sense1 SDR_STREAM_BINARY OUT_DIR. Every setting is here.

use arc_sense::{energy, slow, Paths, Slow};
use sdr_band_power::{power as band_power, Band};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thermo_bath::{damp, exchange};
use thermo_host::Medium;

const SECONDS: u64 = 60;
const STEP_NS: u64 = 10_000_000;
const SDR_BIN_NS: u64 = 100_000_000;
const CALIBRATE_NS: u64 = 5_000_000_000;
const CENTER: u32 = 93_000_000;
const CHANNEL: u32 = 93_300_000;
const RATE: u32 = 2_048_000;
const THROTTLE: [&str; 8] = ["pl1", "pl2", "pl4", "prochot", "ratl", "thermal", "vr_tdc", "vr_thermalert"];

fn arc_paths() -> Paths {
    let hw = std::fs::read_dir("/sys/class/hwmon").unwrap().map(|e| e.unwrap().path()).find(|p| std::fs::read_to_string(p.join("name")).map(|n| n.trim() == "xe").unwrap_or(false)).expect("no xe hwmon");
    let dev = std::fs::canonicalize(hw.join("device")).unwrap();
    let freq = dev.join("tile0/gt0/freq0");
    Paths {
        energy: hw.join("energy2_input"),
        pkg_temp: hw.join("temp2_input"),
        vram_temp: hw.join("temp3_input"),
        act_freq: freq.join("act_freq"),
        throttle: THROTTLE.iter().map(|n| (n.to_string(), freq.join(format!("throttle/reason_{n}")))).collect(),
    }
}

/// The thermal base's step: T5's medium at T = 0.
fn transport(m: &mut Medium) {
    for f in m.fields.iter_mut() {
        for _ in 0..4 {
            exchange(f, 0.0, 0.5, &|_| 0.0, &|_, _| 0.0);
            damp(f, 0.0, &|_| 0.1, &|_| 0.0, &|_| 0.0);
        }
    }
}

/// The latest value whose time is at or before t.
fn latest(series: &[(u64, f64)], t: u64) -> Option<f64> {
    let i = series.partition_point(|x| x.0 <= t);
    (i > 0).then(|| series[i - 1].1)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (bin, out) = (PathBuf::from(&args[1]), PathBuf::from(&args[2]));
    std::fs::create_dir_all(&out).unwrap();
    let paths = arc_paths();
    println!("SENSE-1: receive only. Arc sensors at {:?}; dongle via {:?}", paths.energy.parent().unwrap(), bin);
    // Record: the dongle on a thread (raw frames kept in memory), the Arc polled on this one.
    let mut child = Command::new(&bin).args(["/dev/swradio0", &CENTER.to_string(), &CHANNEL.to_string(), &RATE.to_string(), &SECONDS.to_string()]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("sdr-stream");
    let mut stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut frames = Vec::new();
        while let Some(f) = sdr_frames::read(&mut stdout).expect("TDR1 stream") {
            frames.push(f);
        }
        frames
    });
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
    let frames = reader.join().unwrap();
    drop(child.stdin.take());
    let status = child.wait().unwrap();
    let mut err = String::new();
    std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut err).unwrap();
    std::fs::write(out.join("sdr-stream-stderr.txt"), &err).unwrap();
    println!("sdr-stream exit {status}; {} frames; Arc: {} energy samples, {} slow samples", frames.len(), en.len(), sl.len());
    // Evidence: raw Arc records, the first 0.25 s of raw dongle bytes.
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
    let raw: Vec<u8> = frames.iter().filter(|f| f.kind == 1).flat_map(|f| f.data.iter().copied()).take(RATE as usize / 2).collect();
    std::fs::write(out.join("sdr-first-0.25s.cu8"), &raw).unwrap();
    // S1.
    let dur = (en.last().unwrap().0 - en[0].0) as f64 * 1e-9;
    let monotone = en.windows(2).all(|w| w[1].1 >= w[0].1);
    let distinct = en.windows(2).filter(|w| w[1].1 != w[0].1).count() as f64 / dur;
    let watts = arc_flux::power(&en, STEP_NS);
    let mean_w = (en.last().unwrap().1 - en[0].1) as f64 * 1e-6 / dur;
    let temps: Vec<f64> = sl.iter().map(|s| s.pkg_mc as f64 / 1000.0).collect();
    let (tmin, tmax) = (temps.iter().cloned().fold(f64::MAX, f64::min), temps.iter().cloned().fold(f64::MIN, f64::max));
    println!(
        "S1 Arc: {dur:.1} s; counter monotone {monotone}; {distinct:.0} distinct updates/s; mean power {mean_w:.2} W; pkg temp {tmin:.1}-{tmax:.1} C; vram {:.1} C; act_freq range {}-{} MHz",
        sl[0].vram_mc as f64 / 1000.0,
        sl.iter().map(|s| s.freq_mhz).min().unwrap(),
        sl.iter().map(|s| s.freq_mhz).max().unwrap()
    );
    let s1 = monotone && distinct >= 500.0 && tmin >= 20.0 && tmax <= 95.0 && (5.0..=75.0).contains(&mean_w);
    // S2: band power per frame, binned by stamp.
    let chan = Band { offset_hz: CHANNEL as f64 - CENTER as f64, rate_hz: RATE as f64, taps: 8 };
    let off = Band { offset_hz: -500_000.0, rate_hz: RATE as f64, taps: 8 };
    let data: Vec<&sdr_frames::Frame> = frames.iter().filter(|f| f.kind == 1).collect();
    let gap_markers = frames.iter().filter(|f| f.kind == 2).count();
    let gaps = frames.iter().map(|f| f.gaps).max().unwrap_or(0);
    let mut first = 0u64;
    let t_sdr0 = data[0].stamp_ns;
    let mut bins: Vec<(u64, f64, usize, f64, usize)> = Vec::new();
    for f in &data {
        let (cs, cb) = band_power(&f.data, first, &chan);
        let (os, ob) = band_power(&f.data, first, &off);
        first += f.data.len() as u64 / 2;
        let end = t_sdr0 + ((f.stamp_ns - t_sdr0) / SDR_BIN_NS + 1) * SDR_BIN_NS;
        match bins.last_mut() {
            Some(b) if b.0 == end => {
                b.1 += cs;
                b.2 += cb;
                b.3 += os;
                b.4 += ob;
            }
            _ => bins.push((end, cs, cb, os, ob)),
        }
    }
    let chan_series: Vec<(u64, f64)> = bins.iter().map(|b| (b.0, b.1 / b.2 as f64)).collect();
    let pc = bins.iter().map(|b| b.1).sum::<f64>() / bins.iter().map(|b| b.2).sum::<usize>() as f64;
    let po = bins.iter().map(|b| b.3).sum::<f64>() / bins.iter().map(|b| b.4).sum::<usize>() as f64;
    let contrast = 10.0 * (pc / po).log10();
    let mut w = std::fs::File::create(out.join("sdr-bins.csv")).unwrap();
    writeln!(w, "bin_end_ns,channel_power,offchannel_power").unwrap();
    for b in &bins {
        writeln!(w, "{},{:e},{:e}", b.0, b.1 / b.2 as f64, b.3 / b.4 as f64).unwrap();
    }
    println!("S2 dongle: {} data frames ({first} samples), gap markers {gap_markers}, reported gaps {gaps}; 93.3 MHz channel {contrast:.2} dB over the off-channel", data.len());
    let s2 = contrast >= 10.0 && gaps == 0 && gap_markers == 0;
    // The feed: four arms in lock-step on 10 ms steps.
    let t0 = en[0].0.max(t_sdr0) + SDR_BIN_NS;
    let t_end = en.last().unwrap().0.min(bins.last().unwrap().0);
    let cal = |s: &[(u64, f64)]| {
        let v: Vec<f64> = s.iter().filter(|x| x.0 >= t0 && x.0 < t0 + CALIBRATE_NS).map(|x| x.1).collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    let (ref_arc, ref_sdr) = (cal(&watts), cal(&chan_series));
    println!("calibration (first 5 s): Arc {ref_arc:.3} W, dongle channel power {ref_sdr:.4e}");
    let l = 16;
    let site = |x: usize| x + l * 8 + l * l * 8;
    let (sa, sd, sp) = (site(6), site(10), site(8));
    let mut arms: Vec<Medium> = (0..4).map(|_| Medium::new(&[l; 3], 1)).collect();
    let (mut plain_zero, mut max_dev, mut max_mag) = (true, 0.0f64, 0.0f64);
    let mut probe = [0.0f64; 4];
    let mut steps = 0usize;
    let mut t = t0;
    let mut w = std::fs::File::create(out.join("probe.csv")).unwrap();
    writeln!(w, "t_ns,arc_W,sdr_power,probe_plain,probe_arc,probe_sdr,probe_both").unwrap();
    while t <= t_end {
        let (pa, ps) = (latest(&watts, t).unwrap_or(ref_arc), latest(&chan_series, t).unwrap_or(ref_sdr));
        for (k, m) in arms.iter_mut().enumerate() {
            if k == 1 || k == 3 {
                m.inject(sa, 0, &[pa / ref_arc], 1.0);
            }
            if k == 2 || k == 3 {
                m.inject(sd, 0, &[ps / ref_sdr], 1.0);
            }
        }
        let v: Vec<f64> = arms.iter().map(|m| m.sense(sp, 0, 1)[0]).collect();
        writeln!(w, "{t},{pa:.4},{ps:.6e},{},{},{},{}", v[0], v[1], v[2], v[3]).unwrap();
        for k in 0..4 {
            probe[k] += v[k];
        }
        plain_zero &= arms[0].fields[0].e.iter().all(|x| x.to_bits() == 0);
        for i in 0..l * l * l {
            let (a, d, b) = (arms[1].fields[0].e[i], arms[2].fields[0].e[i], arms[3].fields[0].e[i]);
            max_dev = max_dev.max((b - (a + d)).abs());
            max_mag = max_mag.max(b.abs());
        }
        for m in arms.iter_mut() {
            transport(m);
        }
        steps += 1;
        t += STEP_NS;
    }
    for p in probe.iter_mut() {
        *p /= steps as f64;
    }
    println!("feed: {steps} steps of 10 ms; probe mean heat: plain {:e}, Arc alone {:.6}, dongle alone {:.6}, together {:.6}", probe[0], probe[1], probe[2], probe[3]);
    println!("S5 together - (Arc + dongle): max {max_dev:.2e} of the largest value {max_mag:.4}");
    let (s3, s4, s5) = (plain_zero && probe[0] == 0.0, probe[1] > 0.0 && probe[2] > 0.0, max_dev <= 1e-12 * max_mag);
    // S6: the die's law on 1 s points.
    let e_at = |x: u64| en[en.partition_point(|p| p.0 <= x).max(1) - 1];
    let pts: Vec<(f64, f64, f64)> = (1..)
        .map(|s| en[0].0 + s * 1_000_000_000)
        .take_while(|&tt| tt <= en.last().unwrap().0)
        .map(|tt| {
            let (a, b) = (e_at(tt - 1_000_000_000), e_at(tt));
            let temp = sl[sl.partition_point(|s| s.t_ns <= tt).max(1) - 1].pkg_mc as f64 / 1000.0;
            ((tt - en[0].0) as f64 * 1e-9, temp, (b.1 - a.1) as f64 * 1e-6 / ((b.0 - a.0) as f64 * 1e-9))
        })
        .collect();
    let p_range = pts.iter().map(|p| p.2).fold(f64::MIN, f64::max) - pts.iter().map(|p| p.2).fold(f64::MAX, f64::min);
    let fit = arc_rc::fit(&pts);
    let s6 = if p_range >= 3.0 { fit.map(|f| f.r2 >= 0.5 && (15.0..=40.0).contains(&f.t_amb_c)).unwrap_or(false) } else { true };
    println!("S6 die law: power range {p_range:.2} W over {} points; fit {fit:?}; {}", pts.len(), if p_range < 3.0 { "UNIDENTIFIABLE AT IDLE (power varied < 3 W)" } else { "identifiable" });
    // S7.
    let flags: Vec<u32> = sl.iter().map(|s| s.throttle).collect();
    let ev = arc_boil::events(&flags, THROTTLE.len());
    let ever = flags.iter().fold(0u32, |a, &f| a | f);
    let raised: Vec<&str> = THROTTLE.iter().enumerate().filter(|(k, _)| ever & (1 << k) != 0).map(|(_, n)| *n).collect();
    println!("S7 boiling: events per flag {ev:?} ({}); flags raised at any time: {raised:?}", THROTTLE.join(","));
    let s7 = ev.iter().all(|&e| e == 0);
    let v = [
        ("S1 the Arc senses", s1),
        ("S2 the dongle senses (>= 10 dB, no gaps)", s2),
        ("S3 zero in gives exactly zero heat", s3),
        ("S4 each source alone delivers heat", s4),
        ("S5 the sources add (<= 1e-12)", s5),
        ("S6 the die's thermal law (or unidentifiable at idle)", s6),
        ("S7 no boiling at idle", s7),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
