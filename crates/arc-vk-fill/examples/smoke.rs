//! A one-second smoke test of the transmitter (run before BEAT-1, not one of its checks).
fn energy() -> u64 {
    std::fs::read_to_string("/sys/class/hwmon/hwmon4/energy2_input").unwrap().trim().parse().unwrap()
}
fn main() {
    let mut t = arc_vk_fill::Transmitter::open(0x8086, "Arc", 256 << 20, 4).expect("open");
    println!("device {:?} vendor {:#x} id {:#x}; self-check {} of {} words", t.name, t.vendor, t.device_id, if t.check_ok { "all" } else { "NOT all" }, t.check_words);
    let (t0, e0) = (arc_sense::monotonic_ns(), energy());
    std::thread::sleep(std::time::Duration::from_secs(1));
    let (t1, e1) = (arc_sense::monotonic_ns(), energy());
    let n = t.run_until(t1 + 1_000_000_000).expect("run");
    let (t2, e2) = (arc_sense::monotonic_ns(), energy());
    let w = |a: u64, b: u64, ta: u64, tb: u64| (b - a) as f64 * 1e-6 / ((tb - ta) as f64 * 1e-9);
    println!("idle {:.2} W; load {:.2} W over {n} submissions ({:.2} ms each, {:.1} GB/s)", w(e0, e1, t0, t1), w(e1, e2, t1, t2), (t2 - t1) as f64 * 1e-6 / n as f64, n as f64 * 4.0 * 0.268435456 / ((t2 - t1) as f64 * 1e-9));
}
