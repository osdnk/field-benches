use bin_fields::{f128, f162, reference};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::arch::x86_64::*;

fn to_arr(x: __m512i) -> [u64; 8] {
    unsafe { std::mem::transmute(x) }
}

fn from_arr(x: [u64; 8]) -> __m512i {
    unsafe { std::mem::transmute(x) }
}

fn rand162(r: &mut ChaCha8Rng) -> [u64; 3] {
    [r.r#gen(), r.r#gen(), r.r#gen::<u64>() & ((1u64 << 34) - 1)]
}

#[test]
fn funnel_shift_semantics() {
    unsafe {
        let a = 0x0123_4567_89ab_cdefu64;
        let b = 0xfedc_ba98_7654_3210u64;
        let va = _mm512_set1_epi64(a as i64);
        let vb = _mm512_set1_epi64(b as i64);
        assert_eq!(
            to_arr(_mm512_shrdi_epi64::<17>(va, vb))[0],
            (a >> 17) | (b << 47)
        );
        assert_eq!(
            to_arr(_mm512_shldi_epi64::<17>(va, vb))[0],
            (a << 17) | (b >> 47)
        );
    }
}

#[test]
fn cyclotomic_order_243() {
    let x: [u64; 3] = [2, 0, 0];
    let mut acc: [u64; 3] = [1, 0, 0];
    let mut hit_one = Vec::new();
    for k in 1..=243 {
        acc = reference::mul162(acc, x);
        if acc == [1, 0, 0] {
            hit_one.push(k);
        }
    }
    assert_eq!(hit_one, vec![243]);
}

#[test]
fn polyval_montgomery_one() {
    let one = 0xc200_0000_0000_0000_0000_0000_0000_0001u128;
    let mut r = ChaCha8Rng::seed_from_u64(7);
    for _ in 0..64 {
        let a: u128 = r.r#gen();
        assert_eq!(reference::mul_polyval(a, one), a);
    }
}

#[test]
fn f162_soa8() {
    let mut r = ChaCha8Rng::seed_from_u64(3);
    for _ in 0..256 {
        let a: Vec<[u64; 3]> = (0..8).map(|_| rand162(&mut r)).collect();
        let b: Vec<[u64; 3]> = (0..8).map(|_| rand162(&mut r)).collect();
        let mut aw = [[0u64; 8]; 3];
        let mut bw = [[0u64; 8]; 3];
        for i in 0..8 {
            for w in 0..3 {
                aw[w][i] = a[i][w];
                bw[w][i] = b[i][w];
            }
        }
        let va = [from_arr(aw[0]), from_arr(aw[1]), from_arr(aw[2])];
        let vb = [from_arr(bw[0]), from_arr(bw[1]), from_arr(bw[2])];
        let out = unsafe { f162::mul_soa8(va, vb) };
        let o = [to_arr(out[0]), to_arr(out[1]), to_arr(out[2])];
        for i in 0..8 {
            let want = reference::mul162(a[i], b[i]);
            assert_eq!([o[0][i], o[1][i], o[2][i]], want, "elem {i}");
        }
    }
}

#[test]
fn f128_polyval_aos4() {
    let mut r = ChaCha8Rng::seed_from_u64(4);
    for _ in 0..256 {
        let a: Vec<u128> = (0..4).map(|_| r.r#gen()).collect();
        let b: Vec<u128> = (0..4).map(|_| r.r#gen()).collect();
        let mut av = [0u64; 8];
        let mut bv = [0u64; 8];
        for i in 0..4 {
            av[2 * i] = a[i] as u64;
            av[2 * i + 1] = (a[i] >> 64) as u64;
            bv[2 * i] = b[i] as u64;
            bv[2 * i + 1] = (b[i] >> 64) as u64;
        }
        let out = unsafe { f128::polyval_binius_aos4(from_arr(av), from_arr(bv)) };
        let o = to_arr(out);
        for i in 0..4 {
            let want = reference::mul_polyval(a[i], b[i]);
            let got = (o[2 * i] as u128) | ((o[2 * i + 1] as u128) << 64);
            assert_eq!(got, want, "lane {i}");
        }
    }
}

#[test]
fn f128_polyval_soa8() {
    let mut r = ChaCha8Rng::seed_from_u64(5);
    for _ in 0..256 {
        let a: Vec<u128> = (0..8).map(|_| r.r#gen()).collect();
        let b: Vec<u128> = (0..8).map(|_| r.r#gen()).collect();
        let mut aw = [[0u64; 8]; 2];
        let mut bw = [[0u64; 8]; 2];
        for i in 0..8 {
            aw[0][i] = a[i] as u64;
            aw[1][i] = (a[i] >> 64) as u64;
            bw[0][i] = b[i] as u64;
            bw[1][i] = (b[i] >> 64) as u64;
        }
        let out = unsafe {
            f128::polyval_soa8(
                [from_arr(aw[0]), from_arr(aw[1])],
                [from_arr(bw[0]), from_arr(bw[1])],
            )
        };
        let o = [to_arr(out[0]), to_arr(out[1])];
        for i in 0..8 {
            let want = reference::mul_polyval(a[i], b[i]);
            let got = (o[0][i] as u128) | ((o[1][i] as u128) << 64);
            assert_eq!(got, want, "elem {i}");
        }
    }
}

#[test]
fn f128_ghash_soa8() {
    let mut r = ChaCha8Rng::seed_from_u64(6);
    for _ in 0..256 {
        let a: Vec<u128> = (0..8).map(|_| r.r#gen()).collect();
        let b: Vec<u128> = (0..8).map(|_| r.r#gen()).collect();
        let mut aw = [[0u64; 8]; 2];
        let mut bw = [[0u64; 8]; 2];
        for i in 0..8 {
            aw[0][i] = a[i] as u64;
            aw[1][i] = (a[i] >> 64) as u64;
            bw[0][i] = b[i] as u64;
            bw[1][i] = (b[i] >> 64) as u64;
        }
        let out = unsafe {
            f128::ghash_soa8(
                [from_arr(aw[0]), from_arr(aw[1])],
                [from_arr(bw[0]), from_arr(bw[1])],
            )
        };
        let o = [to_arr(out[0]), to_arr(out[1])];
        for i in 0..8 {
            let want = reference::mul_ghash(a[i], b[i]);
            let got = (o[0][i] as u128) | ((o[1][i] as u128) << 64);
            assert_eq!(got, want, "elem {i}");
        }
    }
}

#[test]
fn f162_mac_deferred() {
    let mut r = ChaCha8Rng::seed_from_u64(11);
    for _ in 0..64 {
        let k = 5;
        let mut acc = unsafe { [_mm512_setzero_si512(); 12] };
        let mut want = vec![[0u64; 3]; 8];
        for _ in 0..k {
            let a: Vec<[u64; 3]> = (0..8).map(|_| rand162(&mut r)).collect();
            let b: Vec<[u64; 3]> = (0..8).map(|_| rand162(&mut r)).collect();
            let mut aw = [[0u64; 8]; 3];
            let mut bw = [[0u64; 8]; 3];
            for i in 0..8 {
                for w in 0..3 {
                    aw[w][i] = a[i][w];
                    bw[w][i] = b[i][w];
                }
            }
            unsafe {
                f162::mac_soa8(
                    &mut acc,
                    [from_arr(aw[0]), from_arr(aw[1]), from_arr(aw[2])],
                    [from_arr(bw[0]), from_arr(bw[1]), from_arr(bw[2])],
                )
            };
            for i in 0..8 {
                let p = reference::mul162(a[i], b[i]);
                for w in 0..3 {
                    want[i][w] ^= p[w];
                }
            }
        }
        let out = unsafe { f162::reduce_soa8(acc) };
        let o = [to_arr(out[0]), to_arr(out[1]), to_arr(out[2])];
        for i in 0..8 {
            assert_eq!([o[0][i], o[1][i], o[2][i]], want[i], "elem {i}");
        }
    }
}

#[test]
fn f128_mac_deferred() {
    let mut r = ChaCha8Rng::seed_from_u64(12);
    for _ in 0..64 {
        let mut acc = unsafe { [_mm512_setzero_si512(); 6] };
        let mut want = vec![0u128; 8];
        for _ in 0..5 {
            let a: Vec<u128> = (0..8).map(|_| r.r#gen()).collect();
            let b: Vec<u128> = (0..8).map(|_| r.r#gen()).collect();
            let mut aw = [[0u64; 8]; 2];
            let mut bw = [[0u64; 8]; 2];
            for i in 0..8 {
                aw[0][i] = a[i] as u64;
                aw[1][i] = (a[i] >> 64) as u64;
                bw[0][i] = b[i] as u64;
                bw[1][i] = (b[i] >> 64) as u64;
            }
            unsafe {
                f128::mac_soa8(
                    &mut acc,
                    [from_arr(aw[0]), from_arr(aw[1])],
                    [from_arr(bw[0]), from_arr(bw[1])],
                )
            };
            for i in 0..8 {
                want[i] ^= reference::mul_polyval(a[i], b[i]);
            }
        }
        let out = unsafe { f128::reduce_soa8(acc) };
        let o = [to_arr(out[0]), to_arr(out[1])];
        for i in 0..8 {
            assert_eq!((o[0][i] as u128) | ((o[1][i] as u128) << 64), want[i], "elem {i}");
        }
    }
}
