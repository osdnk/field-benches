pub mod f128;
pub mod f162;
pub mod reference;

pub fn has_avx512() -> bool {
    is_x86_feature_detected!("avx512f")
        && is_x86_feature_detected!("avx512vbmi2")
        && is_x86_feature_detected!("vpclmulqdq")
}

#[cfg(target_arch = "x86_64")]
pub mod probe_syms {
    use crate::{f128, f162};
    use std::arch::x86_64::*;

    #[no_mangle]
    pub unsafe extern "C" fn k_f162_soa8(a: *const __m512i, b: *const __m512i, o: *mut __m512i) {
        let x = [*a, *a.add(1), *a.add(2)];
        let y = [*b, *b.add(1), *b.add(2)];
        let r = f162::mul_soa8(x, y);
        *o = r[0];
        *o.add(1) = r[1];
        *o.add(2) = r[2];
    }

    #[no_mangle]
    pub unsafe extern "C" fn k_f128_soa8(a: *const __m512i, b: *const __m512i, o: *mut __m512i) {
        let r = f128::polyval_soa8([*a, *a.add(1)], [*b, *b.add(1)]);
        *o = r[0];
        *o.add(1) = r[1];
    }

    #[no_mangle]
    pub unsafe extern "C" fn k_f162_mac_loop(
        a: *const __m512i,
        b: *const __m512i,
        n: usize,
        o: *mut __m512i,
    ) {
        let mut acc = [_mm512_setzero_si512(); 12];
        for i in 0..n {
            f162::mac_soa8(
                &mut acc,
                [*a.add(3 * i), *a.add(3 * i + 1), *a.add(3 * i + 2)],
                [*b.add(3 * i), *b.add(3 * i + 1), *b.add(3 * i + 2)],
            );
        }
        let r = f162::reduce_soa8(acc);
        *o = r[0];
        *o.add(1) = r[1];
        *o.add(2) = r[2];
    }

    #[no_mangle]
    pub unsafe extern "C" fn k_f128_binius_aos4(a: *const __m512i, b: *const __m512i, o: *mut __m512i) {
        *o = f128::polyval_binius_aos4(*a, *b);
    }
}
