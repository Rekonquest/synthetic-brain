//! PHYS3 R3 (docs/PHYS3-POC.md): mass segregation as a cache. 4,096 memories, a 256-slot core (cost 1)
//! and a halo (cost 20). Bars: random static core, LRU. The segregation law: each memory's mass decays
//! (gamma per access step) and grows by 1 per use; every K accesses the heaviest 256 settle into the
//! core. Streams: stationary Zipf(1.0), and Zipf reshuffled every 50,000.

const N: usize = 4096;
const CORE: usize = 256;
const LEN: usize = 200_000;
const HALO: f64 = 20.0;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn stream(seed: u64, shifting: bool) -> Vec<usize> {
    let mut r = Rng(seed);
    let w: Vec<f64> = (1..=N).map(|k| 1.0 / k as f64).collect();
    let tot: f64 = w.iter().sum();
    let cdf: Vec<f64> = w.iter().scan(0.0, |a, x| { *a += x / tot; Some(*a) }).collect();
    let mut perm: Vec<usize> = (0..N).collect();
    let shuffle = |perm: &mut Vec<usize>, r: &mut Rng| {
        for i in (1..N).rev() {
            let j = (r.next() % (i as u64 + 1)) as usize;
            perm.swap(i, j);
        }
    };
    shuffle(&mut perm, &mut r);
    (0..LEN).map(|t| {
        if shifting && t > 0 && t % 50_000 == 0 {
            shuffle(&mut perm, &mut r);
        }
        let u = r.unit();
        let k = cdf.partition_point(|&c| c < u).min(N - 1);
        perm[k]
    }).collect()
}

fn random_core(s: &[usize], seed: u64) -> f64 {
    let mut r = Rng(seed ^ 0xC0E);
    let mut core = vec![false; N];
    let mut placed = 0;
    while placed < CORE {
        let i = (r.next() % N as u64) as usize;
        if !core[i] { core[i] = true; placed += 1; }
    }
    s.iter().map(|&i| if core[i] { 1.0 } else { HALO }).sum::<f64>() / s.len() as f64
}

fn lru(s: &[usize]) -> f64 {
    let mut last = vec![usize::MAX; N];
    let mut core: Vec<usize> = Vec::new();
    let mut cost = 0.0;
    for (t, &i) in s.iter().enumerate() {
        if let Some(p) = core.iter().position(|&x| x == i) {
            cost += 1.0;
            core.swap_remove(p);
        } else {
            cost += HALO;
            if core.len() == CORE {
                let (p, _) = core.iter().enumerate().min_by_key(|(_, &x)| last[x]).unwrap();
                core.swap_remove(p);
            }
        }
        core.push(i);
        last[i] = t;
    }
    cost / s.len() as f64
}

fn segregation(s: &[usize], gamma: f64, k: usize, chaos: Option<u64>) -> f64 {
    let mut m = vec![0.0f64; N];
    let mut at = vec![0usize; N];
    let mut core = vec![false; N];
    let mut cost = 0.0;
    let mut next = k;
    let mut settles = 0u64;
    for (t, &i) in s.iter().enumerate() {
        cost += if core[i] { 1.0 } else { HALO };
        m[i] = m[i] * gamma.powi((t - at[i]) as i32) + 1.0;
        at[i] = t;
        if t + 1 >= next {
            let eff: Vec<f64> = (0..N).map(|j| m[j] * gamma.powi((t - at[j]) as i32)).collect();
            let mut idx: Vec<usize> = (0..N).collect();
            idx.sort_by(|&a, &b| eff[b].partial_cmp(&eff[a]).unwrap());
            core = vec![false; N];
            for &j in &idx[..CORE] {
                core[j] = true;
            }
            settles += 1;
            let step = match chaos {
                Some(sd) => (k as f64 * (1.0 + 0.5 * (qb_chaos::keyed_unit(sd.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ settles) - 0.5))) as usize,
                None => k,
            };
            next = t + 1 + step.max(1);
        }
    }
    cost / s.len() as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS3 R3: mass segregation as a cache; {N} memories, core {CORE} (cost 1), halo cost {HALO}; {LEN} accesses");
    let gammas = [0.999, 0.9999, 0.99999];
    let dev = |g: f64| (0..4u64).map(|sd| segregation(&stream(sd, false), g, 1000, None) + segregation(&stream(sd, true), g, 1000, None)).sum::<f64>();
    let gamma = *gammas.iter().min_by(|a, b| dev(**a).partial_cmp(&dev(**b)).unwrap()).unwrap();
    println!("  dev gamma: {gamma}");
    let mut res = Vec::new();
    for shifting in [false, true] {
        let ss: Vec<Vec<usize>> = (10..18u64).map(|sd| stream(sd, shifting)).collect();
        let avg = |f: &dyn Fn(&[usize], u64) -> f64| ss.iter().enumerate().map(|(k, s)| f(s, 10 + k as u64)).sum::<f64>() / ss.len() as f64;
        let row = [avg(&|s, sd| random_core(s, sd)), avg(&|s, _| lru(s)), avg(&|s, _| segregation(s, gamma, 1000, None)), avg(&|s, sd| segregation(s, gamma, 1000, Some(sd))), avg(&|s, _| segregation(s, gamma, 250, None))];
        println!("\n  {} stream: mean access cost", if shifting { "shifting" } else { "stationary" });
        for (nm, c) in ["random core (bar)", "LRU (status quo)", "segregation (dynamical friction)", "segregation + chaos interval", "segregation, settle every 250"].iter().zip(&row) {
            println!("    {nm:<34} {c:.3}");
        }
        res.push(row);
    }
    let (st, sh) = (res[0], res[1]);
    let v = [
        (format!("R3a stationary: segregation <= LRU - 5% ({:.3} vs {:.3})", st[2], st[1]), st[2] <= 0.95 * st[1]),
        (format!("R3b shifting: within 10% of LRU ({:.3} vs {:.3})", sh[2], sh[1]), sh[2] <= 1.10 * sh[1]),
        (format!("R3c chaos not kept ({:.3}/{:.3} vs {:.3}/{:.3})", st[3], sh[3], st[2], sh[2]), !(st[3] < st[2] && sh[3] < sh[2])),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
