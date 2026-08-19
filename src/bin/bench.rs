use bin_fields::{f128, f162};
use std::arch::asm;
use std::arch::x86_64::*;
use std::hint::black_box;
use std::time::Instant;

#[inline(never)]
fn clock_hz() -> f64 {
    let iters: u64 = 20_000_000;
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let t = Instant::now();
        unsafe {
            let mut n = iters;
            asm!(
                "2:",
                "vpaddq zmm0, zmm0, zmm1", "vpaddq zmm0, zmm0, zmm1",
                "vpaddq zmm0, zmm0, zmm1", "vpaddq zmm0, zmm0, zmm1",
                "vpaddq zmm0, zmm0, zmm1", "vpaddq zmm0, zmm0, zmm1",
                "vpaddq zmm0, zmm0, zmm1", "vpaddq zmm0, zmm0, zmm1",
                "sub {n}, 1",
                "jnz 2b",
                n = inout(reg) n,
                out("zmm0") _, out("zmm1") _,
                options(nostack)
            );
            let _ = n;
        }
        let d = t.elapsed().as_secs_f64();
        if d < best {
            best = d;
        }
    }
    (iters * 8) as f64 / best
}

const NB: usize = 48;
const PASSES: usize = 400;
const LATN: usize = 200_000;
const TRIALS: usize = 11;

struct Row {
    name: &'static str,
    bits: u32,
    lanes: usize,
    tput_ns: f64,
    const_ns: f64,
    lat_cyc: f64,
}

fn v512(seed: u64, mask: u64) -> __m512i {
    let mut s = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let mut o = [0u64; 8];
    for x in o.iter_mut() {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        *x = s & mask;
    }
    unsafe { std::mem::transmute(o) }
}

#[inline(always)]
fn pass<T: Copy>(a: &mut [T], b: &[T], f: impl Fn(T, T) -> T) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x = f(*x, *y);
    }
    black_box(a.as_ptr());
}

#[inline(always)]
fn pass_const<T: Copy>(a: &mut [T], r: T, f: impl Fn(T, T) -> T) {
    for x in a.iter_mut() {
        *x = f(*x, r);
    }
    black_box(a.as_ptr());
}

fn best_ns(muls: f64, mut run: impl FnMut()) -> f64 {
    run();
    let mut best = f64::INFINITY;
    for _ in 0..TRIALS {
        let t = Instant::now();
        run();
        let d = t.elapsed().as_secs_f64();
        if d < best {
            best = d;
        }
    }
    best * 1e9 / muls
}

#[inline(always)]
fn measure<T: Copy>(
    name: &'static str,
    bits: u32,
    lanes: usize,
    mut a: Vec<T>,
    b: Vec<T>,
    hz: f64,
    f: impl Fn(T, T) -> T + Copy,
) -> Row {
    let muls = (NB * lanes * PASSES) as f64;
    let tput_ns = best_ns(muls, || {
        for _ in 0..PASSES {
            pass(&mut a, &b, f);
        }
    });
    let r = b[0];
    let const_ns = best_ns(muls, || {
        for _ in 0..PASSES {
            pass_const(&mut a, r, f);
        }
    });
    let mut acc = a[0];
    let mut best = f64::INFINITY;
    for _ in 0..3 {
        let t = Instant::now();
        for _ in 0..LATN {
            acc = f(acc, r);
        }
        let d = t.elapsed().as_secs_f64();
        if d < best {
            best = d;
        }
    }
    black_box(&acc);
    Row {
        name,
        bits,
        lanes,
        tput_ns,
        const_ns,
        lat_cyc: best / LATN as f64 * hz,
    }
}

fn main() {
    let hz = clock_hz();
    println!("clock {:.3} GHz   (dependent vpaddq zmm chain)\n", hz / 1e9);

    let f162_soa = |o: u64| -> Vec<[__m512i; 3]> {
        (0..NB)
            .map(|i| {
                let s = o + i as u64 * 3;
                [v512(s, !0), v512(s + 1, !0), v512(s + 2, (1 << 34) - 1)]
            })
            .collect()
    };
    let f162_aos = |o: u64, dup: bool| -> Vec<f162::Aos4> {
        (0..NB)
            .map(|i| {
                let s = o + i as u64 * 2;
                let src: [u64; 8] = unsafe { std::mem::transmute(v512(s + 1, (1 << 34) - 1)) };
                let mut w = [0u64; 8];
                for k in 0..4 {
                    w[2 * k] = src[2 * k];
                    w[2 * k + 1] = if dup { src[2 * k] } else { 0 };
                }
                f162::Aos4 {
                    q01: v512(s, !0),
                    q2: unsafe { std::mem::transmute(w) },
                }
            })
            .collect()
    };
    let f128_soa = |o: u64| -> Vec<[__m512i; 2]> {
        (0..NB)
            .map(|i| {
                let s = o + i as u64 * 2;
                [v512(s, !0), v512(s + 1, !0)]
            })
            .collect()
    };
    let f128_aos = |o: u64| -> Vec<__m512i> { (0..NB).map(|i| v512(o + i as u64, !0)).collect() };

    let rows = vec![
        measure("f128 polyval binius aos4", 128, 4, f128_aos(3000), f128_aos(4000), hz, |x, y| unsafe {
            f128::polyval_binius_aos4(x, y)
        }),
        measure("f128 polyval soa8", 128, 8, f128_soa(5000), f128_soa(6000), hz, |x, y| unsafe {
            f128::polyval_soa8(x, y)
        }),
        measure("f128 ghash soa8", 128, 8, f128_soa(7000), f128_soa(8000), hz, |x, y| unsafe {
            f128::ghash_soa8(x, y)
        }),
        measure("f162 aos4 v0 (C port)", 162, 4, f162_aos(100, false), f162_aos(900, false), hz, |x, y| unsafe {
            f162::mul_aos4_v0(x, y)
        }),
        measure("f162 aos4 v1 (opt)", 162, 4, f162_aos(200, true), f162_aos(950, true), hz, |x, y| unsafe {
            f162::mul_aos4_v1(x, y)
        }),
        measure("f162 soa8", 162, 8, f162_soa(1), f162_soa(2000), hz, |x, y| unsafe {
            f162::mul_soa8(x, y)
        }),
    ];

    let mac_rows = {
        let a3 = f162_soa(11);
        let b3 = f162_soa(3111);
        let a2 = f128_soa(22);
        let b2 = f128_soa(4222);
        let muls = (NB * 8 * PASSES) as f64;
        let f162_ns = best_ns(muls, || {
            let mut acc = unsafe { [_mm512_setzero_si512(); 12] };
            for _ in 0..PASSES {
                for (x, y) in a3.iter().zip(b3.iter()) {
                    unsafe { f162::mac_soa8(&mut acc, *x, *y) };
                }
            }
            black_box(unsafe { f162::reduce_soa8(acc) });
        });
        let f128_ns = best_ns(muls, || {
            let mut acc = unsafe { [_mm512_setzero_si512(); 6] };
            for _ in 0..PASSES {
                for (x, y) in a2.iter().zip(b2.iter()) {
                    unsafe { f128::mac_soa8(&mut acc, *x, *y) };
                }
            }
            black_box(unsafe { f128::reduce_soa8(acc) });
        });
        [("f128 polyval mac soa8", 128u32, f128_ns), ("f162 mac soa8", 162, f162_ns)]
    };

    println!(
        "{:<26}{:>5}{:>7}{:>10}{:>9}{:>9}{:>10}{:>11}{:>9}",
        "kernel", "bits", "lanes", "ns/mul", "cyc/mul", "cyc/bit", "Gmul/s", "const rhs", "lat cyc"
    );
    println!("{}", "-".repeat(96));
    for r in &rows {
        let c = r.tput_ns * 1e-9 * hz;
        println!(
            "{:<26}{:>5}{:>7}{:>10.4}{:>9.2}{:>9.4}{:>10.3}{:>11.2}{:>9.1}",
            r.name,
            r.bits,
            r.lanes,
            r.tput_ns,
            c,
            c / r.bits as f64,
            1.0 / r.tput_ns,
            r.const_ns * 1e-9 * hz,
            r.lat_cyc
        );
    }

    println!("\nmultiply-accumulate, reduction deferred to end of dot product");
    println!(
        "{:<26}{:>5}{:>7}{:>10}{:>9}{:>9}{:>10}",
        "kernel", "bits", "lanes", "ns/mac", "cyc/mac", "cyc/bit", "Gmac/s"
    );
    println!("{}", "-".repeat(76));
    for (name, bits, ns) in mac_rows {
        let c = ns * 1e-9 * hz;
        println!(
            "{:<26}{:>5}{:>7}{:>10.4}{:>9.2}{:>9.4}{:>10.3}",
            name, bits, 8, ns, c, c / bits as f64, 1.0 / ns
        );
    }
}
