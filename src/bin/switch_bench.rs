use bin_fields::crossfield::*;
use bin_fields::scalar::{B128, F162};
use std::time::Instant;

fn xs(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

fn main() {
    let l: usize = std::env::args()
        .nth(1)
        .and_then(|x| x.parse().ok())
        .unwrap_or(18);
    let mut s = 0x1234_5678_9abc_def1u64;
    let pi0: Vec<B128> = (0..1usize << l)
        .map(|_| B128((xs(&mut s) as u128) | ((xs(&mut s) as u128) << 64)))
        .collect();
    let r_lo: Vec<B128> = (0..LOG_PACK)
        .map(|_| B128((xs(&mut s) as u128) | ((xs(&mut s) as u128) << 64)))
        .collect();
    let r_hi: Vec<B128> = (0..l)
        .map(|_| B128((xs(&mut s) as u128) | ((xs(&mut s) as u128) << 64)))
        .collect();
    let ch = Transcript {
        r_prime: (0..LOG_PACK)
            .map(|_| F162([xs(&mut s), xs(&mut s), xs(&mut s) & ((1 << 34) - 1)]))
            .collect(),
        r_pp: (0..l)
            .map(|_| F162([xs(&mut s), xs(&mut s), xs(&mut s) & ((1 << 34) - 1)]))
            .collect(),
    };

    let t = Instant::now();
    let eq_hi = eq_expand_b128(&r_hi);
    println!("eq_expand_b128        {:>9.2?}", t.elapsed());

    let t = Instant::now();
    let v = partial_evals(&pi0, &eq_hi);
    println!("partial evals (v)     {:>9.2?}", t.elapsed());

    let t = Instant::now();
    let batch = eq_expand_f162(&ch.r_prime);
    let tab = psi_table(&batch);
    let a: Vec<F162> = eq_hi.iter().map(|&e| psi(&tab, e)).collect();
    println!("transparent poly A    {:>9.2?}", t.elapsed());

    let t = Instant::now();
    let p1: Vec<F162> = pi0.iter().map(|&x| F162::from_b128(x)).collect();
    println!("lift pi0 -> pi1       {:>9.2?}", t.elapsed());

    let t = Instant::now();
    let mut ap = bin_fields::sumcheck::Poly::from_scalars(&a);
    let mut pp = bin_fields::sumcheck::Poly::from_scalars(&p1);
    let mut half = 1usize << l;
    for round in 0..l {
        half /= 2;
        std::hint::black_box(bin_fields::sumcheck::round(&mut ap, &mut pp, half, ch.r_pp[round]));
    }
    println!("sumcheck ({l} rounds)   {:>9.2?}", t.elapsed());

    let t = Instant::now();
    let d = transparent_coeff(&r_hi, &ch.r_pp, &batch);
    println!("verifier coeff D      {:>9.2?}", t.elapsed());
    std::hint::black_box((v, d));

    let t = Instant::now();
    let claim = {
        let eq_lo = eq_expand_b128(&r_lo);
        let mut acc = B128::ZERO;
        for (y, &p) in pi0.iter().enumerate() {
            for i in 0..PACK {
                if p.bit(i) {
                    acc = acc + eq_lo[i] * eq_hi[y];
                }
            }
        }
        acc
    };
    println!("(ref claim)           {:>9.2?}", t.elapsed());
    let proof = prove(&pi0, &r_lo, &r_hi, &ch);
    let t = Instant::now();
    let ok = verify(&proof, claim, &r_lo, &r_hi, &ch);
    println!("verify total          {:>9.2?}  {:?}", t.elapsed(), ok.is_ok());
    println!(
        "proof: v = {} B128 = {} B, rounds = {} x 2 x F162 = {} B",
        proof.v.len(),
        proof.v.len() * 16,
        proof.rounds.len(),
        proof.rounds.len() * 2 * 21
    );
}
