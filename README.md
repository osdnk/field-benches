# field-benches

AVX-512 multiplication benchmarks for two binary fields, aimed at sumcheck.
Requires AVX512F + AVX512VBMI2 + VPCLMULQDQ; no scalar fallback.

- **F(2^128)**, POLYVAL basis, modulus `x^128+x^127+x^126+x^121+1`, Montgomery
  form — the algorithm binius uses
  (`crates/field/src/arch/x86_64/pclmul/montgomery_mul.rs`, commit `47675e1`,
  Apache-2.0), ported faithfully as the baseline.
- **F(2^162)** = `F2[x]/Phi_243`, `Phi_243 = x^162+x^81+1`. `Phi_243` divides
  `x^243-1`, so `x^243 = 1` and `x^162 = x^81+1`: reduction is shifts and XORs
  only, no carry-less multiply and no Montgomery. Baseline here is *the original
  C kernel* — hand-written AVX-512 C intrinsics, one field element per 128-bit
  lane.

Two layouts are compared throughout. **Element-per-lane** puts one field element
in each 128-bit lane of a zmm register (what binius and the original C kernel
do). **Word-sliced** puts limb *k* of 8 consecutive elements in one zmm.

## Results

Tiger Lake i7-11850H, one pinned core, ~4.6 GHz, L1-resident. `ns/mul` is stable
to 4 digits across runs. *lanes* = field elements per 512-bit register.

| kernel | bits | lanes | ns/mul | cyc/mul |
|---|---|---|---|---|
| f128 polyval, binius layout | 128 | 4 | 1.081 | 5.0 |
| f128 polyval, word-sliced | 128 | 8 | **0.702** | 3.2 |
| f128 ghash `x^128+x^7+x^2+x+1`, word-sliced | 128 | 8 | 0.738 | 3.4 |
| f162, the original C kernel | 162 | 4 | 1.688 | 7.8 |
| f162, same layout, optimised | 162 | 4 | 1.516 | 7.0 |
| f162, word-sliced | 162 | 8 | **1.097** | 5.0 |

Multiply-accumulate, reduction deferred to the end of a dot product — binary
fields have no carries, so products XOR-accumulate unreduced indefinitely:

| kernel | bits | ns/multiply-accumulate |
|---|---|---|
| f128 polyval | 128 | 0.341 |
| f162 | 162 | 0.784 |

- Word-slicing is **1.54x** on both fields.
- Optimised F162 (1.097 ns) ties binius' current F128 (1.081 ns).
- Best against best, an F162 multiply costs 1.56x an F128 multiply, 1.24x per
  field bit.
- Deferring the reduction is 2.06x faster than reducing every multiply on F128,
  1.40x on F162. The gap between the fields widens to 1.82x per bit, because the
  cheap `x^243 = 1` reduction is exactly what gets amortised away, leaving the
  raw carry-less-multiply counts, 6 against 3.

## Why word-slicing wins

On this chip `vpclmulqdq` (carry-less multiply) on a zmm register takes 2 cycles
and issues only on **port 5** — and `vpshufd`, `vpslldq`, `vpermt2q`, `vpunpck*`
issue only on port 5 as well, 1 cycle each. Port 5 is therefore the scarce
resource, and every lane shuffle costs half a carry-less multiply.

Element-per-lane layouts spend port 5 shuffling 64-bit halves into position
before they can multiply. Word-sliced consumes operands where they already sit:
zero input shuffles, leaving only the transpose of the product words. Port-5
cycles per 8 multiplications: 24 -> 18 for F128, 52 -> 34 for F162. Both
optimised kernels run at 94-95% of their floor — the same instruction mix with
all data dependencies removed.

What remains of F162's cost is limb count, not the modulus: 3 limbs means 6
carry-less multiplies, i.e. 27 field bits per multiply, against 42.7 for 2-limb
F128.

Two caveats. Word-sliced is a different memory format, so improving binius this
way means a new packed type, not a patch to the existing one. And this is one
microarchitecture: Zen 4/5 have much better `vpclmulqdq` throughput, so the
port-5 argument does not transfer.

## Correctness

11 tests. Every SIMD kernel is checked lane-by-lane against a bit-serial scalar
reference on random inputs; `x` has multiplicative order exactly 243 in the F162
reference; and binius' `ONE = 0xc2000000000000000000000000000001` is the
identity under the reference Montgomery multiply.

## Run

```
cargo test --release
cargo run --release --bin probe   # per-instruction throughput and port probe (inline asm)
cargo run --release --bin bench   # the table above
cargo bench                       # criterion, agrees within 5-8%
```

## Source

```
src/f128.rs       polyval_binius_aos4 (binius port), polyval_soa8, ghash_soa8,
                  prods_soa8 / mac_soa8 / reduce_soa8 (deferred reduction)
src/f162.rs       mul_aos4_v0 (the original C kernel), mul_aos4_v1, mul_soa8,
                  prods_soa8 / mac_soa8 / reduce_soa8
src/reference.rs  bit-serial scalar references
src/bin/probe.rs  src/bin/bench.rs  benches/mul.rs
```

`aos4` = element-per-lane, `soa8` = word-sliced.

`polyval_binius_aos4` is a port of `simd_montgomery_multiply` from
[binius](https://github.com/IrreducibleOSS/binius) (Irreducible Inc.,
Apache-2.0); the F162 element-per-lane kernel is a port of C intrinsics supplied
by the repo author.
