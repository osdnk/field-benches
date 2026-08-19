use bin_fields::{f128, f162};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::arch::x86_64::*;
use std::hint::black_box;

const NB: usize = 48;

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

fn soa3(o: u64) -> Vec<[__m512i; 3]> {
    (0..NB)
        .map(|i| {
            let s = o + i as u64 * 3;
            [v512(s, !0), v512(s + 1, !0), v512(s + 2, (1 << 34) - 1)]
        })
        .collect()
}

fn soa2(o: u64) -> Vec<[__m512i; 2]> {
    (0..NB)
        .map(|i| {
            let s = o + i as u64 * 2;
            [v512(s, !0), v512(s + 1, !0)]
        })
        .collect()
}

fn aos4(o: u64, dup: bool) -> Vec<f162::Aos4> {
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
}

macro_rules! tput {
    ($g:expr, $name:literal, $lanes:expr, $a:expr, $b:expr, $f:expr) => {{
        let mut a = $a;
        let b = $b;
        $g.throughput(Throughput::Elements((NB * $lanes) as u64));
        $g.bench_function($name, |bn| {
            bn.iter(|| {
                for (x, y) in a.iter_mut().zip(b.iter()) {
                    *x = $f(*x, *y);
                }
                black_box(a.as_ptr());
            })
        });
    }};
}

fn bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("mul");
    tput!(g, "f128/polyval_binius_aos4", 4, (0..NB).map(|i| v512(3000 + i as u64, !0)).collect::<Vec<_>>(), (0..NB).map(|i| v512(4000 + i as u64, !0)).collect::<Vec<_>>(), |x, y| unsafe { f128::polyval_binius_aos4(x, y) });
    tput!(g, "f128/polyval_soa8", 8, soa2(5000), soa2(6000), |x, y| unsafe { f128::polyval_soa8(x, y) });
    tput!(g, "f128/ghash_soa8", 8, soa2(7000), soa2(8000), |x, y| unsafe { f128::ghash_soa8(x, y) });
    tput!(g, "f162/aos4_v0", 4, aos4(100, false), aos4(900, false), |x, y| unsafe { f162::mul_aos4_v0(x, y) });
    tput!(g, "f162/aos4_v1", 4, aos4(200, true), aos4(950, true), |x, y| unsafe { f162::mul_aos4_v1(x, y) });
    tput!(g, "f162/soa8", 8, soa3(1), soa3(2000), |x, y| unsafe { f162::mul_soa8(x, y) });
    g.finish();

    let mut g = c.benchmark_group("mac");
    let a3 = soa3(11);
    let b3 = soa3(3111);
    g.throughput(Throughput::Elements((NB * 8) as u64));
    g.bench_function("f162/mac_soa8", |bn| {
        bn.iter(|| {
            let mut acc = unsafe { [_mm512_setzero_si512(); 12] };
            for (x, y) in a3.iter().zip(b3.iter()) {
                unsafe { f162::mac_soa8(&mut acc, *x, *y) };
            }
            black_box(unsafe { f162::reduce_soa8(acc) })
        })
    });
    let a2 = soa2(22);
    let b2 = soa2(4222);
    g.bench_function("f128/mac_soa8", |bn| {
        bn.iter(|| {
            let mut acc = unsafe { [_mm512_setzero_si512(); 6] };
            for (x, y) in a2.iter().zip(b2.iter()) {
                unsafe { f128::mac_soa8(&mut acc, *x, *y) };
            }
            black_box(unsafe { f128::reduce_soa8(acc) })
        })
    });
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
