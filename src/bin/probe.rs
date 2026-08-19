#![allow(unused_assignments)]
use std::arch::asm;
use std::arch::x86_64::*;
use std::time::Instant;

const UNROLL: u64 = 1;

macro_rules! probe {
    ($name:ident, [$($body:literal),+ $(,)?]) => {
        #[inline(never)]
        unsafe fn $name(iters: u64, a: __m512i, b: __m512i) {
            let mut n = iters;
            asm!(
                "mov rax, 0xaa",
                "kmovq k1, rax",
                "2:",
                $($body,)+
                "sub {n}, 1",
                "jnz 2b",
                n = inout(reg) n,
                in("zmm16") a,
                in("zmm17") b,
                out("zmm0") _, out("zmm1") _, out("zmm2") _, out("zmm3") _,
                out("zmm4") _, out("zmm5") _, out("zmm6") _, out("zmm7") _,
                out("rax") _, out("k1") _,
                options(nostack)
            );
        }
    };
}

probe!(clmul_zmm_t, [
    "vpclmulqdq zmm0, zmm16, zmm17, 0", "vpclmulqdq zmm1, zmm16, zmm17, 0",
    "vpclmulqdq zmm2, zmm16, zmm17, 0", "vpclmulqdq zmm3, zmm16, zmm17, 0",
    "vpclmulqdq zmm4, zmm16, zmm17, 0", "vpclmulqdq zmm5, zmm16, zmm17, 0",
    "vpclmulqdq zmm6, zmm16, zmm17, 0", "vpclmulqdq zmm7, zmm16, zmm17, 0"]);

probe!(clmul_zmm_l, [
    "vpclmulqdq zmm0, zmm0, zmm17, 0", "vpclmulqdq zmm0, zmm0, zmm17, 0",
    "vpclmulqdq zmm0, zmm0, zmm17, 0", "vpclmulqdq zmm0, zmm0, zmm17, 0",
    "vpclmulqdq zmm0, zmm0, zmm17, 0", "vpclmulqdq zmm0, zmm0, zmm17, 0",
    "vpclmulqdq zmm0, zmm0, zmm17, 0", "vpclmulqdq zmm0, zmm0, zmm17, 0"]);

probe!(clmul_ymm_t, [
    "vpclmulqdq ymm0, ymm16, ymm17, 0", "vpclmulqdq ymm1, ymm16, ymm17, 0",
    "vpclmulqdq ymm2, ymm16, ymm17, 0", "vpclmulqdq ymm3, ymm16, ymm17, 0",
    "vpclmulqdq ymm4, ymm16, ymm17, 0", "vpclmulqdq ymm5, ymm16, ymm17, 0",
    "vpclmulqdq ymm6, ymm16, ymm17, 0", "vpclmulqdq ymm7, ymm16, ymm17, 0"]);

probe!(clmul_xmm_t, [
    "vpclmulqdq xmm0, xmm16, xmm17, 0", "vpclmulqdq xmm1, xmm16, xmm17, 0",
    "vpclmulqdq xmm2, xmm16, xmm17, 0", "vpclmulqdq xmm3, xmm16, xmm17, 0",
    "vpclmulqdq xmm4, xmm16, xmm17, 0", "vpclmulqdq xmm5, xmm16, xmm17, 0",
    "vpclmulqdq xmm6, xmm16, xmm17, 0", "vpclmulqdq xmm7, xmm16, xmm17, 0"]);

probe!(shufd_zmm_t, [
    "vpshufd zmm0, zmm16, 0x4e", "vpshufd zmm1, zmm16, 0x4e",
    "vpshufd zmm2, zmm16, 0x4e", "vpshufd zmm3, zmm16, 0x4e",
    "vpshufd zmm4, zmm16, 0x4e", "vpshufd zmm5, zmm16, 0x4e",
    "vpshufd zmm6, zmm16, 0x4e", "vpshufd zmm7, zmm16, 0x4e"]);

probe!(xor_zmm_t, [
    "vpxorq zmm0, zmm16, zmm17", "vpxorq zmm1, zmm16, zmm17",
    "vpxorq zmm2, zmm16, zmm17", "vpxorq zmm3, zmm16, zmm17",
    "vpxorq zmm4, zmm16, zmm17", "vpxorq zmm5, zmm16, zmm17",
    "vpxorq zmm6, zmm16, zmm17", "vpxorq zmm7, zmm16, zmm17"]);

probe!(sllq_zmm_t, [
    "vpsllq zmm0, zmm16, 17", "vpsllq zmm1, zmm16, 17",
    "vpsllq zmm2, zmm16, 17", "vpsllq zmm3, zmm16, 17",
    "vpsllq zmm4, zmm16, 17", "vpsllq zmm5, zmm16, 17",
    "vpsllq zmm6, zmm16, 17", "vpsllq zmm7, zmm16, 17"]);

probe!(slldq_zmm_t, [
    "vpslldq zmm0, zmm16, 8", "vpslldq zmm1, zmm16, 8",
    "vpslldq zmm2, zmm16, 8", "vpslldq zmm3, zmm16, 8",
    "vpslldq zmm4, zmm16, 8", "vpslldq zmm5, zmm16, 8",
    "vpslldq zmm6, zmm16, 8", "vpslldq zmm7, zmm16, 8"]);

probe!(shrdq_zmm_t, [
    "vpshrdq zmm0, zmm16, zmm17, 34", "vpshrdq zmm1, zmm16, zmm17, 34",
    "vpshrdq zmm2, zmm16, zmm17, 34", "vpshrdq zmm3, zmm16, zmm17, 34",
    "vpshrdq zmm4, zmm16, zmm17, 34", "vpshrdq zmm5, zmm16, zmm17, 34",
    "vpshrdq zmm6, zmm16, zmm17, 34", "vpshrdq zmm7, zmm16, zmm17, 34"]);

probe!(ternlog_zmm_t, [
    "vpternlogq zmm0, zmm16, zmm17, 0x96", "vpternlogq zmm1, zmm16, zmm17, 0x96",
    "vpternlogq zmm2, zmm16, zmm17, 0x96", "vpternlogq zmm3, zmm16, zmm17, 0x96",
    "vpternlogq zmm4, zmm16, zmm17, 0x96", "vpternlogq zmm5, zmm16, zmm17, 0x96",
    "vpternlogq zmm6, zmm16, zmm17, 0x96", "vpternlogq zmm7, zmm16, zmm17, 0x96"]);

probe!(maskxor_zmm_t, [
    "vpxorq zmm0 {{k1}}, zmm16, zmm17", "vpxorq zmm1 {{k1}}, zmm16, zmm17",
    "vpxorq zmm2 {{k1}}, zmm16, zmm17", "vpxorq zmm3 {{k1}}, zmm16, zmm17",
    "vpxorq zmm4 {{k1}}, zmm16, zmm17", "vpxorq zmm5 {{k1}}, zmm16, zmm17",
    "vpxorq zmm6 {{k1}}, zmm16, zmm17", "vpxorq zmm7 {{k1}}, zmm16, zmm17"]);

probe!(permt2q_zmm_t, [
    "vpermt2q zmm0, zmm16, zmm17", "vpermt2q zmm1, zmm16, zmm17",
    "vpermt2q zmm2, zmm16, zmm17", "vpermt2q zmm3, zmm16, zmm17",
    "vpermt2q zmm4, zmm16, zmm17", "vpermt2q zmm5, zmm16, zmm17",
    "vpermt2q zmm6, zmm16, zmm17", "vpermt2q zmm7, zmm16, zmm17"]);

probe!(addq_zmm_l, [
    "vpaddq zmm0, zmm0, zmm17", "vpaddq zmm0, zmm0, zmm17",
    "vpaddq zmm0, zmm0, zmm17", "vpaddq zmm0, zmm0, zmm17",
    "vpaddq zmm0, zmm0, zmm17", "vpaddq zmm0, zmm0, zmm17",
    "vpaddq zmm0, zmm0, zmm17", "vpaddq zmm0, zmm0, zmm17"]);

probe!(clmul_xor_mix_t, [
    "vpclmulqdq zmm0, zmm16, zmm17, 0", "vpxorq zmm1, zmm16, zmm17",
    "vpxorq zmm2, zmm16, zmm17", "vpxorq zmm3, zmm16, zmm17",
    "vpclmulqdq zmm4, zmm16, zmm17, 0", "vpxorq zmm5, zmm16, zmm17",
    "vpxorq zmm6, zmm16, zmm17", "vpxorq zmm7, zmm16, zmm17"]);

probe!(clmul_shuf_mix_t, [
    "vpclmulqdq zmm0, zmm16, zmm17, 0", "vpshufd zmm1, zmm16, 0x4e",
    "vpshufd zmm2, zmm16, 0x4e", "vpshufd zmm3, zmm16, 0x4e",
    "vpclmulqdq zmm4, zmm16, zmm17, 0", "vpshufd zmm5, zmm16, 0x4e",
    "vpshufd zmm6, zmm16, 0x4e", "vpshufd zmm7, zmm16, 0x4e"]);


probe!(unpck_zmm_t, [
    "vpunpcklqdq zmm0, zmm16, zmm17", "vpunpckhqdq zmm1, zmm16, zmm17",
    "vpunpcklqdq zmm2, zmm16, zmm17", "vpunpckhqdq zmm3, zmm16, zmm17",
    "vpunpcklqdq zmm4, zmm16, zmm17", "vpunpckhqdq zmm5, zmm16, zmm17",
    "vpunpcklqdq zmm6, zmm16, zmm17", "vpunpckhqdq zmm7, zmm16, zmm17"]);

probe!(mix_f162_soa8, [
    "vpclmulqdq zmm0, zmm16, zmm17, 0", "vpclmulqdq zmm1, zmm16, zmm17, 17",
    "vpclmulqdq zmm2, zmm16, zmm17, 0", "vpclmulqdq zmm3, zmm16, zmm17, 17",
    "vpclmulqdq zmm4, zmm16, zmm17, 0", "vpclmulqdq zmm5, zmm16, zmm17, 17",
    "vpclmulqdq zmm6, zmm16, zmm17, 0", "vpclmulqdq zmm7, zmm16, zmm17, 17",
    "vpclmulqdq zmm0, zmm16, zmm17, 0", "vpclmulqdq zmm1, zmm16, zmm17, 17",
    "vpclmulqdq zmm2, zmm16, zmm17, 0", "vpclmulqdq zmm3, zmm16, zmm17, 17",
    "vpunpcklqdq zmm4, zmm16, zmm17", "vpunpckhqdq zmm5, zmm16, zmm17",
    "vpunpcklqdq zmm6, zmm16, zmm17", "vpunpckhqdq zmm7, zmm16, zmm17",
    "vpunpcklqdq zmm0, zmm16, zmm17", "vpunpckhqdq zmm1, zmm16, zmm17",
    "vpunpcklqdq zmm2, zmm16, zmm17", "vpunpckhqdq zmm3, zmm16, zmm17",
    "vpunpcklqdq zmm4, zmm16, zmm17", "vpunpckhqdq zmm5, zmm16, zmm17",
    "vpxorq zmm0, zmm16, zmm17", "vpxorq zmm1, zmm16, zmm17",
    "vpxorq zmm2, zmm16, zmm17", "vpxorq zmm3, zmm16, zmm17",
    "vpxorq zmm4, zmm16, zmm17", "vpxorq zmm5, zmm16, zmm17",
    "vpxorq zmm6, zmm16, zmm17", "vpxorq zmm7, zmm16, zmm17",
    "vpxorq zmm0, zmm16, zmm17", "vpxorq zmm1, zmm16, zmm17",
    "vpxorq zmm2, zmm16, zmm17", "vpxorq zmm3, zmm16, zmm17",
    "vpxorq zmm4, zmm16, zmm17", "vpxorq zmm5, zmm16, zmm17",
    "vpternlogq zmm0, zmm16, zmm17, 0x96", "vpternlogq zmm1, zmm16, zmm17, 0x96",
    "vpternlogq zmm2, zmm16, zmm17, 0x96", "vpternlogq zmm3, zmm16, zmm17, 0x96",
    "vpternlogq zmm4, zmm16, zmm17, 0x96", "vpternlogq zmm5, zmm16, zmm17, 0x96",
    "vpternlogq zmm6, zmm16, zmm17, 0x96", "vpternlogq zmm7, zmm16, zmm17, 0x96",
    "vpternlogq zmm0, zmm16, zmm17, 0x96", "vpandq zmm1, zmm16, zmm17",
    "vpshrdq zmm2, zmm16, zmm17, 34", "vpshrdq zmm3, zmm16, zmm17, 51",
    "vpshrdq zmm4, zmm16, zmm17, 51", "vpshldq zmm5, zmm16, zmm17, 17",
    "vpsrlq zmm6, zmm16, 34", "vpsllq zmm7, zmm16, 17"]);

probe!(mix_f128_soa8, [
    "vpclmulqdq zmm0, zmm16, zmm17, 0", "vpclmulqdq zmm1, zmm16, zmm17, 17",
    "vpclmulqdq zmm2, zmm16, zmm17, 0", "vpclmulqdq zmm3, zmm16, zmm17, 17",
    "vpclmulqdq zmm4, zmm16, zmm17, 0", "vpclmulqdq zmm5, zmm16, zmm17, 17",
    "vpunpcklqdq zmm6, zmm16, zmm17", "vpunpckhqdq zmm7, zmm16, zmm17",
    "vpunpcklqdq zmm0, zmm16, zmm17", "vpunpckhqdq zmm1, zmm16, zmm17",
    "vpunpcklqdq zmm2, zmm16, zmm17", "vpunpckhqdq zmm3, zmm16, zmm17",
    "vpxorq zmm4, zmm16, zmm17", "vpxorq zmm5, zmm16, zmm17",
    "vpternlogq zmm0, zmm16, zmm17, 0x96", "vpternlogq zmm1, zmm16, zmm17, 0x96",
    "vpternlogq zmm2, zmm16, zmm17, 0x96", "vpternlogq zmm3, zmm16, zmm17, 0x96",
    "vpternlogq zmm4, zmm16, zmm17, 0x96", "vpternlogq zmm5, zmm16, zmm17, 0x96",
    "vpternlogq zmm6, zmm16, zmm17, 0x96", "vpternlogq zmm7, zmm16, zmm17, 0x96",
    "vpternlogq zmm0, zmm16, zmm17, 0x96", "vpternlogq zmm1, zmm16, zmm17, 0x96",
    "vpsrlq zmm2, zmm16, 1", "vpsrlq zmm3, zmm16, 2",
    "vpsrlq zmm4, zmm16, 7", "vpsllq zmm5, zmm16, 63",
    "vpsllq zmm6, zmm16, 62", "vpsllq zmm7, zmm16, 57",
    "vpsrlq zmm0, zmm16, 1", "vpsrlq zmm1, zmm16, 2",
    "vpsrlq zmm2, zmm16, 7", "vpsllq zmm3, zmm16, 63",
    "vpsllq zmm4, zmm16, 62", "vpsllq zmm5, zmm16, 57"]);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let which = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let iters: u64 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000_000);

    let a = unsafe { _mm512_set1_epi64(0x0123_4567_89ab_cdefu64 as i64) };
    let b = unsafe { _mm512_set1_epi64(0xfedc_ba98_7654_3210u64 as i64) };

    let table: Vec<(&str, unsafe fn(u64, __m512i, __m512i))> = vec![
        ("clmul_zmm_t", clmul_zmm_t),
        ("clmul_zmm_l", clmul_zmm_l),
        ("clmul_ymm_t", clmul_ymm_t),
        ("clmul_xmm_t", clmul_xmm_t),
        ("shufd_zmm_t", shufd_zmm_t),
        ("xor_zmm_t", xor_zmm_t),
        ("sllq_zmm_t", sllq_zmm_t),
        ("slldq_zmm_t", slldq_zmm_t),
        ("shrdq_zmm_t", shrdq_zmm_t),
        ("ternlog_zmm_t", ternlog_zmm_t),
        ("maskxor_zmm_t", maskxor_zmm_t),
        ("permt2q_zmm_t", permt2q_zmm_t),
        ("addq_zmm_l", addq_zmm_l),
        ("clmul_xor_mix_t", clmul_xor_mix_t),
        ("clmul_shuf_mix_t", clmul_shuf_mix_t),
        ("unpck_zmm_t", unpck_zmm_t),
        ("mix_f162_soa8", mix_f162_soa8),
        ("mix_f128_soa8", mix_f128_soa8),
    ];

    if which == "list" {
        for (n, _) in &table {
            println!("{n}");
        }
        return;
    }

    for (name, f) in &table {
        if which != "all" && which != *name {
            continue;
        }
        unsafe { f(iters / 20, a, b) };
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let t = Instant::now();
            unsafe { f(iters, a, b) };
            let d = t.elapsed().as_secs_f64();
            if d < best {
                best = d;
            }
        }
        let ops = (iters * UNROLL) as f64;
        println!("{:<18} {:>8.3} ns/op   ops={}", name, best * 1e9 / ops, ops as u64);
    }
}
