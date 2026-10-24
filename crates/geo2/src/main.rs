//! GEO-1 G2 orchestrator (docs/GEO-1-PRECOMMIT.md): plan a path across a 64 x 64 grid with random
//! obstacles. Bar: greedy. New stacks from physics: least action as Feynman's sum over paths (a field
//! u_s = Σ over paths (λ/4)^length, climbed from the start) and the renormalization group (solve on
//! 4 x 4 blocks, refine in the coarse path's corridor, fall back to the full grid if needed). Ground
//! truth: BFS. Every setting is here.

use std::collections::VecDeque;

const W: usize = 64;
const BLOCK: usize = 4;
const WORLDS: usize = 64;
const RHOS: [f64; 3] = [0.15, 0.25, 0.35];
const LAMBDA: f64 = 0.5;

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

fn bfs(free: &[bool], w: usize, h: usize, start: usize, goal: usize) -> Option<usize> {
    let mut d = vec![usize::MAX; free.len()];
    let mut q = VecDeque::new();
    d[start] = 0;
    q.push_back(start);
    while let Some(s) = q.pop_front() {
        if s == goal {
            return Some(d[s]);
        }
        let (x, y) = (s % w, s / w);
        let mut nb = Vec::new();
        if x > 0 { nb.push(s - 1); }
        if x + 1 < w { nb.push(s + 1); }
        if y > 0 { nb.push(s - w); }
        if y + 1 < h { nb.push(s + w); }
        for t in nb {
            if free[t] && d[t] == usize::MAX {
                d[t] = d[s] + 1;
                q.push_back(t);
            }
        }
    }
    None
}

/// Relax the sum over paths to its fixed point; returns (field, cost in cell updates).
fn field(free: &[bool], w: usize, h: usize, goal: usize, lambda: f64) -> (Vec<f64>, usize) {
    let mut u = vec![0.0; free.len()];
    let cells = free.iter().filter(|x| **x).count();
    let mut sweeps = 0;
    loop {
        let ch = path_sum::sweep(&mut u, free, w, h, goal, lambda);
        sweeps += 1;
        if (ch < 1e-9 && sweeps > w + h) || sweeps > 20 * (w + h) {
            break;
        }
    }
    (u, sweeps * cells)
}

fn greedy(free: &[bool], start: usize, goal: usize, cap: usize) -> Option<usize> {
    let gx = (goal % W) as f64;
    let gy = (goal / W) as f64;
    let u: Vec<f64> = (0..free.len()).map(|s| -(((s % W) as f64 - gx).powi(2) + ((s / W) as f64 - gy).powi(2)).sqrt()).collect();
    path_climb::climb(&u, free, W, W, start, goal, cap, None)
}

/// The renormalization group: coarse field on blocks, corridor refine, fallback. (length, cost, fell back).
fn rg(free: &[bool], start: usize, goal: usize, cap: usize, lambda: f64) -> (Option<usize>, usize, bool) {
    let cw = W / BLOCK;
    let block = |s: usize| (s % W) / BLOCK + cw * ((s / W) / BLOCK);
    let mut cfree: Vec<bool> = (0..cw * cw)
        .map(|b| {
            let (bx, by) = (b % cw, b / cw);
            (0..BLOCK).flat_map(|dy| (0..BLOCK).map(move |dx| (bx * BLOCK + dx) + W * (by * BLOCK + dy))).filter(|&s| free[s]).count() * 2 >= BLOCK * BLOCK
        })
        .collect();
    cfree[block(start)] = true;
    cfree[block(goal)] = true;
    let (cu, ccost) = field(&cfree, cw, cw, block(goal), lambda);
    // The coarse path's blocks.
    let mut path_blocks = vec![block(start)];
    {
        let mut seen = vec![false; cw * cw];
        let mut s = block(start);
        seen[s] = true;
        for _ in 0..4 * cw * cw {
            if s == block(goal) {
                break;
            }
            let (x, y) = (s % cw, s / cw);
            let mut nb = Vec::new();
            if x > 0 { nb.push(s - 1); }
            if x + 1 < cw { nb.push(s + 1); }
            if y > 0 { nb.push(s - cw); }
            if y + 1 < cw { nb.push(s + cw); }
            match nb.into_iter().filter(|&t| cfree[t] && !seen[t]).max_by(|&a, &b| cu[a].partial_cmp(&cu[b]).unwrap()) {
                Some(t) => {
                    s = t;
                    seen[s] = true;
                    path_blocks.push(s);
                }
                None => break,
            }
        }
    }
    let mut corridor_blocks = vec![false; cw * cw];
    for &b in &path_blocks {
        let (bx, by) = ((b % cw) as i64, (b / cw) as i64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (nx, ny) = (bx + dx, by + dy);
                if nx >= 0 && ny >= 0 && nx < cw as i64 && ny < cw as i64 {
                    corridor_blocks[nx as usize + cw * ny as usize] = true;
                }
            }
        }
    }
    let cfine: Vec<bool> = (0..W * W).map(|s| free[s] && corridor_blocks[block(s)]).collect();
    let (fu, fcost) = field(&cfine, W, W, goal, lambda);
    let len = path_climb::climb(&fu, &cfine, W, W, start, goal, cap, None);
    if len.is_some() {
        return (len, ccost + fcost, false);
    }
    let (u, cost) = field(free, W, W, goal, lambda);
    (path_climb::climb(&u, free, W, W, start, goal, cap, None), ccost + fcost + cost, true)
}

#[derive(Default, Clone, Copy)]
struct Tally {
    ok: [usize; 6],
    ratio: [f64; 6],
    cost: [f64; 6],
    fallback: usize,
}

fn run(seed: u64, rho: f64) -> Tally {
    let mut r = Rng(seed ^ (rho * 1000.0) as u64);
    let mut t = Tally::default();
    let (start, goal) = (0usize, W * W - 1);
    let mut made = 0;
    while made < WORLDS {
        let mut free: Vec<bool> = (0..W * W).map(|_| r.unit() >= rho).collect();
        free[start] = true;
        free[goal] = true;
        let Some(best) = bfs(&free, W, W, start, goal) else { continue };
        made += 1;
        let cap = 4 * best;
        let wi = made as u64;
        // 0 greedy, 1 least action, 2 RG, 3 least action + chaos ties, 4 least action amplified (λ 0.25), 5 RG amplified.
        let mut put = |k: usize, len: Option<usize>, cost: usize| {
            if let Some(l) = len {
                t.ok[k] += 1;
                t.ratio[k] += l as f64 / best as f64;
            }
            t.cost[k] += cost as f64;
        };
        put(0, greedy(&free, start, goal, cap), 0);
        let (u, c) = field(&free, W, W, goal, LAMBDA);
        put(1, path_climb::climb(&u, &free, W, W, start, goal, cap, None), c);
        let tie = |step: usize, n: usize| -> usize { (qb_chaos::keyed_unit(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ wi.wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (step as u64).wrapping_mul(0x1656_67B1_9E37_79F9)) * n as f64) as usize };
        put(3, path_climb::climb(&u, &free, W, W, start, goal, cap, Some(&tie)), c);
        let (rl, rc, fb) = rg(&free, start, goal, cap, LAMBDA);
        put(2, rl, rc);
        t.fallback += fb as usize;
        let (ua, ca) = field(&free, W, W, goal, 0.25);
        put(4, path_climb::climb(&ua, &free, W, W, start, goal, cap, None), ca);
        let (rl2, rc2, _) = rg(&free, start, goal, cap, 0.25);
        put(5, rl2, rc2);
    }
    t
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("GEO-1 G2: path planning on a {W} x {W} grid, {WORLDS} connected worlds per seed and density; test seeds 10-17");
    let names = ["greedy (bar)", "least action (sum over paths)", "renormalization group", "least action + chaos ties", "least action, lambda 0.25", "RG, lambda 0.25"];
    let mut res = std::collections::HashMap::new();
    for &rho in &RHOS {
        let ts: Vec<Tally> = std::thread::scope(|sc| {
            let hs: Vec<_> = (10..18u64).map(|s| sc.spawn(move || run(s, rho))).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let n = (WORLDS * ts.len()) as f64;
        println!("\n  density {rho}: strategy, success, length / BFS (successes), cost (cell updates per world)");
        for k in 0..6 {
            let ok: usize = ts.iter().map(|t| t.ok[k]).sum();
            let ratio = ts.iter().map(|t| t.ratio[k]).sum::<f64>() / ok.max(1) as f64;
            let cost = ts.iter().map(|t| t.cost[k]).sum::<f64>() / n;
            println!("    {:<32} {:.3}  {:.4}  {:.0}", names[k], ok as f64 / n, ratio, cost);
            res.insert((k, (rho * 100.0) as i64), (ok as f64 / n, ratio, cost));
        }
        println!("    RG fallbacks to the full grid: {}", ts.iter().map(|t| t.fallback).sum::<usize>());
    }
    let g = |k: usize, r: f64| res[&(k, (r * 100.0) as i64)];
    let v = [
        (format!("G2a greedy success <= 0.6 at 0.25 ({:.3})", g(0, 0.25).0), g(0, 0.25).0 <= 0.6),
        (format!("G2b least action success 1.000 and length <= 1.02 x BFS ({:.3}, {:.4} / {:.4} / {:.4})", RHOS.iter().map(|&r| g(1, r).0).fold(1.0, f64::min), g(1, 0.15).1, g(1, 0.25).1, g(1, 0.35).1), RHOS.iter().all(|&r| g(1, r).0 >= 0.9999 && g(1, r).1 <= 1.02)),
        (format!("G2c RG cost <= 1/4 ({:.2}), success >= 0.95 ({:.3}), length <= 1.1 ({:.4}) at 0.25", g(2, 0.25).2 / g(1, 0.25).2, g(2, 0.25).0, g(2, 0.25).1), g(2, 0.25).2 <= 0.25 * g(1, 0.25).2 && g(2, 0.25).0 >= 0.95 && g(2, 0.25).1 <= 1.1),
        (format!("G2d chaos ties not kept ({:.4} vs {:.4} at 0.35)", g(3, 0.35).1, g(1, 0.35).1), !(g(3, 0.35).1 < g(1, 0.35).1)),
        (format!("G2e lambda 0.25 closer to BFS by >= 0.005 at 0.35 ({:.4} vs {:.4}), no loss of success", g(4, 0.35).1, g(1, 0.35).1), g(4, 0.35).1 <= g(1, 0.35).1 - 0.005 && g(4, 0.35).0 >= g(1, 0.35).0),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
