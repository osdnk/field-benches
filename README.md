# field-benches

AVX-512 multiplication benchmarks for two binary fields, aimed at sumcheck:

- **F(2^128)** in POLYVAL basis, `x^128 + x^127 + x^126 + x^121 + 1`, Montgomery form — the algorithm binius uses (`crates/field/src/arch/x86_64/pclmul/montgomery_mul.rs`, commit `47675e1`).
- **F(2^162)** = `F2[x]/Phi_243(x)`, `Phi_243 = x^162 + x^81 + 1`. Since `Phi_243 | x^243 - 1`, we have `x^243 = 1` and `x^162 = x^81 + 1`, which makes the reduction pure shifts and XORs — no clmul, no Montgomery.

Everything is 512-bit VPCLMULQDQ + AVX512-VBMI2. There is no scalar fallback.

## Results

Tiger Lake i7-11850H, one core pinned, ~4.6 GHz, all operands L1-resident.
`ns/mul` is the primary number (stable to 4 digits across runs); cycles are
derived from a dependent `vpaddq zmm` chain measured in the same process.

| kernel | bits | lanes | ns/mul | cyc/mul | cyc/bit | Gmul/s |
|---|---|---|---|---|---|---|
| f128 polyval, binius layout (4x128b) | 128 | 4 | 1.081 | 5.0 | 0.039 | 0.93 |
| **f128 polyval, word-sliced (8x128b)** | 128 | 8 | **0.702** | 3.2 | 0.025 | **1.43** |
| f128 ghash `x^128+x^7+x^2+x+1`, word-sliced | 128 | 8 | 0.738 | 3.4 | 0.027 | 1.36 |
| f162, direct port of the C intrinsics (4x162b) | 162 | 4 | 1.688 | 7.8 | 0.048 | 0.59 |
| f162, same layout, optimised | 162 | 4 | 1.516 | 7.0 | 0.044 | 0.66 |
| **f162, word-sliced (8x162b)** | 162 | 8 | **1.097** | 5.0 | 0.031 | **0.91** |

Multiply-accumulate, reduction deferred to the end of a dot product:

| kernel | bits | ns/mac | cyc/mac | cyc/bit | Gmac/s |
|---|---|---|---|---|---|
| f128 polyval | 128 | 0.341 | 1.6 | 0.012 | 2.93 |
| f162 | 162 | 0.784 | 3.6 | 0.022 | 1.28 |

Headlines:

- Word-slicing is worth **1.54x** on both fields, independent of the field.
- Optimised F162 (1.097 ns) lands on top of binius' current F128 (1.081 ns).
  Relative to what binius ships today, the extra 34 field bits are free.
- Best against best, **F162 costs 1.56x an F128 multiply, 1.24x per field bit**.
- With deferred reduction the gap widens to **2.30x per mac, 1.82x per bit** —
  F162's cheap reduction is exactly what gets amortised away, leaving the raw
  clmul count.

## Why word-slicing wins

Measured instruction costs on this chip (`cargo run --release --bin probe`),
1 cycle = 0.213 ns:

| op (zmm) | cycles | port |
|---|---|---|
| `vpclmulqdq` | 2.0 tput / 8 lat | p5 only |
| `vpshufd`, `vpslldq`, `vpermt2q`, `vpunpck*` | 1.0 | p5 only |
| `vpsllq`, `vpsrlq`, `vpshrdq`, `vpshldq` | 1.0 | p0 only |
| `vpxorq`, `vpternlogq`, masked xor | 0.5 | p0 + p5 |

Two mixed probes pin it down: 2 clmul + 6 `vpshufd` = 10 cycles (all p5, purely
additive), while 2 clmul + 6 `vpxorq` = 6 cycles. **Port 5 is the currency, and
every lane shuffle costs half a clmul.**

An array-of-structs layout (one field element per 128-bit lane, which is what
binius' `PackedBinaryPolyval4x128b` and the original F162 C code both use)
spends port 5 shuffling 64-bit halves into place before it can even start
multiplying. The word-sliced layout stores limb `k` of 8 consecutive elements in
one zmm, so `clmul<0x00>` / `clmul<0x11>` consume the operands where they already
sit — zero input shuffles. The only lane traffic left is transposing the product
words back, one `vpunpck` per product word.

Port-5 budget per 8 multiplications:

|  | clmul | lane ops | total p5 | per mul |
|---|---|---|---|---|
| f128 binius layout | 6 x 2 = 12 | 12 (`vpshufd`/`unpck`) | 24 | 3.0 |
| f128 word-sliced | 12 | 6 unpack | 18 | 2.25 |
| f162 C-port layout | 24 | 28 | 52 | 6.5 |
| f162 same layout, optimised | 24 | 16 | 40 | 5.0 |
| f162 word-sliced | 24 | 10 unpack | 34 | 4.25 |

(counted from the disassembly, `vpclmulqdq zmm` charged 2 cycles of p5)

Both optimised kernels run at 94-95% of what their own instruction mix allows:
a dependency-free synthetic block with exactly the F162 kernel's op mix takes
38 cycles (kernel: 40), and the F128 mix takes 24 (kernel: 25.6). Further gains
have to delete instructions, not reschedule them.

## Where the remaining F162 cost is

Not the reduction. `x^243 = 1` really is as cheap as it looks: after the
transpose, reducing 6 product words to 3 is 4 funnel shifts (`vpshrdq`/`vpshldq`)
plus 8 boolean ops, all on port 0, which is idle anyway. `H1` (bits 243..322)
folds by XOR alone, and `h11` needs no mask because the third limb is 34 bits so
`p5` is only 3 bits wide.

The cost is limb count. 162 bits needs 3 limbs, and 3-limb Karatsuba is 6 clmuls
— 27 field bits per clmul, against 42.7 for 2-limb F128. The 6 product words also
need 10 transposing unpacks against F128's 6. That is the whole 1.56x.

Worth knowing when picking a field: a 3-limb field gets 6 clmuls whatever its
degree, so degree 191 (`x^191 + x^9 + 1`, irreducible — checked — and fits in 3 limbs)
would deliver 191 bits at the same clmul cost, 32 bits/clmul. F162 buys
something else with those 29 bits: since 162 = 2·3^4 it has subfields at every
divisor — `F(2^2)`, `F(2^3)`, `F(2^6)`, `F(2^9)`, `F(2^18)`, `F(2^27)`,
`F(2^54)`, `F(2^81)` — where degree 191 (prime) has none at all. If you need the tower, F162 is the right shape; if you only
need >128-bit soundness, it is not the cheapest way to buy it.

## For sumcheck

Two different inner loops, two different numbers:

- **Folding** (`a[i] += r * (b[i] - a[i])`, one fixed challenge, result stored):
  needs a full reduced multiply. Use the `const rhs` column of the bench output;
  it runs 2-3% below the general case, since only the challenge-side Karatsuba prep
  hoists out.
- **Round evaluation** (sums of products): binary fields have no carries, so
  products XOR-accumulate unreduced for arbitrarily many terms. `mac_soa8`
  accumulates the 6 (F162) or 3 (F128) raw Karatsuba products and
  `reduce_soa8` runs once per dot product. That is 2.06x faster than reducing
  every multiply on F128 and 1.40x on F162.

Layout cost, since it is not free: 8 F162 elements occupy 3 zmm = 192 B for
162 B of payload (84% density, 24 B/element). F128 is exact at 16 B/element.

## Layout

```
src/f128.rs      polyval_binius_aos4  faithful port of binius' simd_montgomery_multiply
                 polyval_soa8         same field + same Montgomery reduction, word-sliced
                 ghash_soa8           x^128+x^7+x^2+x+1, word-sliced
                 prods_soa8 / mac_soa8 / reduce_soa8    deferred reduction
src/f162.rs      mul_aos4_v0          direct port of the original C intrinsics
                 mul_aos4_v1          same layout, ternlog + VBMI2 funnel shifts + masked xor
                 mul_soa8             word-sliced
                 prods_soa8 / mac_soa8 / reduce_soa8    deferred reduction
src/reference.rs bit-serial scalar reference for all three fields
src/bin/probe.rs instruction throughput / port probe (inline asm)
src/bin/bench.rs the table above
benches/mul.rs   criterion, agrees with bin/bench within 5-8% (mean vs min)
```

```
cargo test --release          # 11 correctness tests
cargo run  --release --bin probe
cargo run  --release --bin bench
cargo bench
```

`.cargo/config.toml` sets `-C target-cpu=native`. Needs `avx512f`,
`avx512vbmi2`, `vpclmulqdq`.

## Correctness

Every SIMD kernel is checked lane-by-lane against a bit-serial scalar reference
on 256 random inputs, plus two structural checks:

- `x` has multiplicative order exactly 243 in the F162 reference — nothing
  smaller hits 1 — which pins down both `Phi_243` and the reduction.
- binius' `BinaryField128bPolyval::ONE = 0xc2000000000000000000000000000001` is
  the identity under the reference Montgomery multiply, confirming the F128
  convention matches binius rather than merely being self-consistent.
- Deferred-reduction MACs are checked against the XOR of individually reduced
  products.

## Caveats

- One machine, one microarchitecture. Zen 4/5 have much better `vpclmulqdq`
  throughput and the port-5 argument does not transfer; the layout conclusion
  probably does, the ratios will not.
- The word-sliced kernels are a different memory format, not a drop-in for
  binius' `PackedField`. Making binius faster this way means a new packed type,
  not a patch to `PackedBinaryPolyval4x128b`.
- Cycle counts assume the frequency measured by a light-AVX-512 probe; a
  clmul-heavy kernel may clock slightly lower, so `cyc/mul` may be marginally
  overstated. `ns/mul` is not affected.
- Everything here is throughput on L1-resident data. Latency (`lat cyc` in the
  bench output) is 30-45 cycles per dependent step and the word-sliced kernels
  are *worse* on it than the AoS ones — they trade critical path for throughput.

## Attribution

`polyval_binius_aos4` is a port of `simd_montgomery_multiply` from
[binius](https://github.com/IrreducibleOSS/binius) (Irreducible Inc.,
Apache-2.0). The F162 AoS kernel is a port of C intrinsics supplied by the repo
author.
