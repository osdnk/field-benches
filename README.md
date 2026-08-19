# field-benches

AVX-512 multiplication benchmarks for two binary fields, aimed at sumcheck. Requires AVX512F + AVX512VBMI2 + VPCLMULQDQ; no scalar fallback.

- **F(2^128)**, POLYVAL basis, modulus x^128+x^127+x^126+x^121+1, Montgomery form — the algorithm the archived binius used (`crates/field/src/arch/x86_64/pclmul/montgomery_mul.rs`, commit 47675e1, Apache-2.0), ported faithfully as the baseline. Its successor binius64 has since moved to the GHASH row below, without Montgomery.
- **F(2^162)** = F2[x]/Phi_243, Phi_243 = x^162+x^81+1. Phi_243 divides x^243-1, so x^243 = 1 and x^162 = x^81+1: reduction is shifts and XORs only — no carry-less multiply (clmul, the `vpclmulqdq` instruction), no Montgomery.

## Results

Tiger Lake i7-11850H, one pinned core, ~4.6 GHz, L1-resident; ns/mul stable to 4 digits across runs. Lanes = field elements per 512-bit register. Word-sliced = limb k of 8 consecutive elements packed in one zmm register; the alternative, element-per-lane, keeps one element per 128-bit lane.

| kernel | bits | lanes | ns/mul | cyc/mul |
|---|---|---|---|---|
| f128 polyval, binius layout | 128 | 4 | 1.081 | 5.0 |
| f128 polyval, word-sliced | 128 | 8 | **0.702** | 3.2 |
| f128 ghash (x^128+x^7+x^2+x+1), word-sliced | 128 | 8 | 0.738 | 3.4 |
| f162, word-sliced | 162 | 8 | **1.097** | 5.1 |

Multiply-accumulate (mac), reduction deferred to the end of a dot product (binary fields have no carries, so products XOR-accumulate unreduced indefinitely):

| kernel | bits | ns/mac |
|---|---|---|
| f128 polyval | 128 | 0.341 |
| f162 | 162 | 0.784 |

- Word-slicing beats binius' layout by **1.54×** — same field, same Montgomery reduction, only the packing differs.
- Best vs best: an F162 multiply costs 1.56× an F128 multiply — 1.24× per field bit.
- Deferred reduction is 2.06× faster than reducing every mul on F128, 1.40× on F162; the F162/F128 gap widens to 1.82× per bit because the cheap x^243 = 1 reduction is what gets amortised away, leaving raw clmul counts 6 vs 3.

## Run

```
cargo test --release
cargo run --release --bin probe   # instruction throughput / port probe (inline asm)
cargo run --release --bin bench   # the table above
cargo bench                       # criterion; agrees within 5–8%
```

## Source

```
src/f128.rs       polyval_binius_aos4 (binius port), polyval_soa8, ghash_soa8, prods_soa8/mac_soa8/reduce_soa8
src/f162.rs       mul_soa8, prods_soa8/mac_soa8/reduce_soa8
src/reference.rs  bit-serial scalar references (tests check every kernel against them)
src/bin/probe.rs, src/bin/bench.rs, benches/mul.rs
```

`polyval_binius_aos4` is a port of `simd_montgomery_multiply` from [binius](https://github.com/IrreducibleOSS/binius) (Irreducible Inc., Apache-2.0).
