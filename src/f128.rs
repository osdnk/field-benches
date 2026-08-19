use std::arch::x86_64::*;

#[inline(always)]
unsafe fn x3(a: __m512i, b: __m512i, c: __m512i) -> __m512i {
    _mm512_ternarylogic_epi64::<0x96>(a, b, c)
}

#[inline(always)]
pub unsafe fn polyval_binius_aos4(h: __m512i, y: __m512i) -> __m512i {
    let h0 = h;
    let h1 = _mm512_shuffle_epi32::<0x0E>(h);
    let h2 = _mm512_xor_si512(h0, h1);
    let y0 = y;
    let y1 = _mm512_shuffle_epi32::<0x0E>(y);
    let y2 = _mm512_xor_si512(y0, y1);
    let t0 = _mm512_clmulepi64_epi128::<0x00>(y0, h0);
    let t1 = _mm512_clmulepi64_epi128::<0x11>(y, h);
    let t2 = _mm512_clmulepi64_epi128::<0x00>(y2, h2);
    let t2 = _mm512_xor_si512(t2, _mm512_xor_si512(t0, t1));
    let v0 = t0;
    let v1 = _mm512_xor_si512(_mm512_shuffle_epi32::<0x0E>(t0), t2);
    let v2 = _mm512_xor_si512(t1, _mm512_shuffle_epi32::<0x0E>(t2));
    let v3 = _mm512_shuffle_epi32::<0x0E>(t1);

    let v2 = _mm512_xor_si512(
        _mm512_xor_si512(v2, v0),
        x3(
            _mm512_srli_epi64::<1>(v0),
            _mm512_srli_epi64::<2>(v0),
            _mm512_srli_epi64::<7>(v0),
        ),
    );
    let v1 = _mm512_xor_si512(
        v1,
        x3(
            _mm512_slli_epi64::<63>(v0),
            _mm512_slli_epi64::<62>(v0),
            _mm512_slli_epi64::<57>(v0),
        ),
    );
    let v3 = _mm512_xor_si512(
        _mm512_xor_si512(v3, v1),
        x3(
            _mm512_srli_epi64::<1>(v1),
            _mm512_srli_epi64::<2>(v1),
            _mm512_srli_epi64::<7>(v1),
        ),
    );
    let v2 = _mm512_xor_si512(
        v2,
        x3(
            _mm512_slli_epi64::<63>(v1),
            _mm512_slli_epi64::<62>(v1),
            _mm512_slli_epi64::<57>(v1),
        ),
    );
    _mm512_unpacklo_epi64(v2, v3)
}

#[inline(always)]
unsafe fn idx_lo() -> __m512i {
    _mm512_setr_epi64(0, 8, 2, 10, 4, 12, 6, 14)
}

#[inline(always)]
unsafe fn idx_hi() -> __m512i {
    _mm512_setr_epi64(1, 9, 3, 11, 5, 13, 7, 15)
}

#[inline(always)]
pub unsafe fn polyval_soa8(a: [__m512i; 2], b: [__m512i; 2]) -> [__m512i; 2] {
    let xa = _mm512_xor_si512(a[0], a[1]);
    let xb = _mm512_xor_si512(b[0], b[1]);

    let m0e = _mm512_clmulepi64_epi128::<0x00>(a[0], b[0]);
    let m0o = _mm512_clmulepi64_epi128::<0x11>(a[0], b[0]);
    let m1e = _mm512_clmulepi64_epi128::<0x00>(a[1], b[1]);
    let m1o = _mm512_clmulepi64_epi128::<0x11>(a[1], b[1]);
    let ke = _mm512_clmulepi64_epi128::<0x00>(xa, xb);
    let ko = _mm512_clmulepi64_epi128::<0x11>(xa, xb);

    let de = x3(ke, m0e, m1e);
    let d_o = x3(ko, m0o, m1o);

    let lo = idx_lo();
    let hi = idx_hi();
    let v0 = _mm512_permutex2var_epi64(m0e, lo, m0o);
    let v1 = _mm512_xor_si512(
        _mm512_permutex2var_epi64(m0e, hi, m0o),
        _mm512_permutex2var_epi64(de, lo, d_o),
    );
    let v2 = _mm512_xor_si512(
        _mm512_permutex2var_epi64(de, hi, d_o),
        _mm512_permutex2var_epi64(m1e, lo, m1o),
    );
    let v3 = _mm512_permutex2var_epi64(m1e, hi, m1o);

    let v2 = _mm512_xor_si512(
        _mm512_xor_si512(v2, v0),
        x3(
            _mm512_srli_epi64::<1>(v0),
            _mm512_srli_epi64::<2>(v0),
            _mm512_srli_epi64::<7>(v0),
        ),
    );
    let v1 = _mm512_xor_si512(
        v1,
        x3(
            _mm512_slli_epi64::<63>(v0),
            _mm512_slli_epi64::<62>(v0),
            _mm512_slli_epi64::<57>(v0),
        ),
    );
    let v3 = _mm512_xor_si512(
        _mm512_xor_si512(v3, v1),
        x3(
            _mm512_srli_epi64::<1>(v1),
            _mm512_srli_epi64::<2>(v1),
            _mm512_srli_epi64::<7>(v1),
        ),
    );
    let v2 = _mm512_xor_si512(
        v2,
        x3(
            _mm512_slli_epi64::<63>(v1),
            _mm512_slli_epi64::<62>(v1),
            _mm512_slli_epi64::<57>(v1),
        ),
    );
    [v2, v3]
}

#[inline(always)]
pub unsafe fn ghash_soa8(a: [__m512i; 2], b: [__m512i; 2]) -> [__m512i; 2] {
    let xa = _mm512_xor_si512(a[0], a[1]);
    let xb = _mm512_xor_si512(b[0], b[1]);

    let m0e = _mm512_clmulepi64_epi128::<0x00>(a[0], b[0]);
    let m0o = _mm512_clmulepi64_epi128::<0x11>(a[0], b[0]);
    let m1e = _mm512_clmulepi64_epi128::<0x00>(a[1], b[1]);
    let m1o = _mm512_clmulepi64_epi128::<0x11>(a[1], b[1]);
    let ke = _mm512_clmulepi64_epi128::<0x00>(xa, xb);
    let ko = _mm512_clmulepi64_epi128::<0x11>(xa, xb);

    let de = x3(ke, m0e, m1e);
    let d_o = x3(ko, m0o, m1o);

    let lo = idx_lo();
    let hi = idx_hi();
    let p0 = _mm512_permutex2var_epi64(m0e, lo, m0o);
    let p1 = _mm512_xor_si512(
        _mm512_permutex2var_epi64(m0e, hi, m0o),
        _mm512_permutex2var_epi64(de, lo, d_o),
    );
    let p2 = _mm512_xor_si512(
        _mm512_permutex2var_epi64(de, hi, d_o),
        _mm512_permutex2var_epi64(m1e, lo, m1o),
    );
    let p3 = _mm512_permutex2var_epi64(m1e, hi, m1o);

    let f0 = x3(
        _mm512_slli_epi64::<1>(p2),
        _mm512_slli_epi64::<2>(p2),
        _mm512_slli_epi64::<7>(p2),
    );
    let f0 = _mm512_xor_si512(f0, p2);
    let f1 = x3(
        _mm512_shldi_epi64::<1>(p3, p2),
        _mm512_shldi_epi64::<2>(p3, p2),
        _mm512_shldi_epi64::<7>(p3, p2),
    );
    let f1 = _mm512_xor_si512(f1, p3);
    let f2 = x3(
        _mm512_srli_epi64::<63>(p3),
        _mm512_srli_epi64::<62>(p3),
        _mm512_srli_epi64::<57>(p3),
    );

    let g0 = x3(
        _mm512_slli_epi64::<1>(f2),
        _mm512_slli_epi64::<2>(f2),
        _mm512_slli_epi64::<7>(f2),
    );
    let g0 = _mm512_xor_si512(g0, f2);

    [x3(p0, f0, g0), _mm512_xor_si512(p1, f1)]
}

#[inline(always)]
pub unsafe fn prods_soa8(a: [__m512i; 2], b: [__m512i; 2]) -> [__m512i; 6] {
    let xa = _mm512_xor_si512(a[0], a[1]);
    let xb = _mm512_xor_si512(b[0], b[1]);
    [
        _mm512_clmulepi64_epi128::<0x00>(a[0], b[0]),
        _mm512_clmulepi64_epi128::<0x11>(a[0], b[0]),
        _mm512_clmulepi64_epi128::<0x00>(a[1], b[1]),
        _mm512_clmulepi64_epi128::<0x11>(a[1], b[1]),
        _mm512_clmulepi64_epi128::<0x00>(xa, xb),
        _mm512_clmulepi64_epi128::<0x11>(xa, xb),
    ]
}

#[inline(always)]
pub unsafe fn mac_soa8(acc: &mut [__m512i; 6], a: [__m512i; 2], b: [__m512i; 2]) {
    let p = prods_soa8(a, b);
    for i in 0..6 {
        acc[i] = _mm512_xor_si512(acc[i], p[i]);
    }
}

#[inline(always)]
pub unsafe fn reduce_soa8(p: [__m512i; 6]) -> [__m512i; 2] {
    let (m0e, m0o, m1e, m1o, ke, ko) = (p[0], p[1], p[2], p[3], p[4], p[5]);
    let de = x3(ke, m0e, m1e);
    let d_o = x3(ko, m0o, m1o);
    let lo = idx_lo();
    let hi = idx_hi();
    let v0 = _mm512_permutex2var_epi64(m0e, lo, m0o);
    let v1 = _mm512_xor_si512(
        _mm512_permutex2var_epi64(m0e, hi, m0o),
        _mm512_permutex2var_epi64(de, lo, d_o),
    );
    let v2 = _mm512_xor_si512(
        _mm512_permutex2var_epi64(de, hi, d_o),
        _mm512_permutex2var_epi64(m1e, lo, m1o),
    );
    let v3 = _mm512_permutex2var_epi64(m1e, hi, m1o);

    let v2 = _mm512_xor_si512(
        _mm512_xor_si512(v2, v0),
        x3(
            _mm512_srli_epi64::<1>(v0),
            _mm512_srli_epi64::<2>(v0),
            _mm512_srli_epi64::<7>(v0),
        ),
    );
    let v1 = _mm512_xor_si512(
        v1,
        x3(
            _mm512_slli_epi64::<63>(v0),
            _mm512_slli_epi64::<62>(v0),
            _mm512_slli_epi64::<57>(v0),
        ),
    );
    let v3 = _mm512_xor_si512(
        _mm512_xor_si512(v3, v1),
        x3(
            _mm512_srli_epi64::<1>(v1),
            _mm512_srli_epi64::<2>(v1),
            _mm512_srli_epi64::<7>(v1),
        ),
    );
    let v2 = _mm512_xor_si512(
        v2,
        x3(
            _mm512_slli_epi64::<63>(v1),
            _mm512_slli_epi64::<62>(v1),
            _mm512_slli_epi64::<57>(v1),
        ),
    );
    [v2, v3]
}
