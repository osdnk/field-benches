use bin_fields::crossfield::*;
use bin_fields::scalar::{B128, F162};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn setup(l: usize, seed: u64) -> (Vec<B128>, Vec<B128>, Vec<B128>, Transcript) {
    let mut r = ChaCha8Rng::seed_from_u64(seed);
    let pi0: Vec<B128> = (0..1 << l).map(|_| B128(r.r#gen())).collect();
    let r_lo: Vec<B128> = (0..LOG_PACK).map(|_| B128(r.r#gen())).collect();
    let r_hi: Vec<B128> = (0..l).map(|_| B128(r.r#gen())).collect();
    let ch = Transcript {
        r_prime: (0..LOG_PACK)
            .map(|_| F162([r.r#gen(), r.r#gen(), r.r#gen::<u64>() & ((1 << 34) - 1)]))
            .collect(),
        r_pp: (0..l)
            .map(|_| F162([r.r#gen(), r.r#gen(), r.r#gen::<u64>() & ((1 << 34) - 1)]))
            .collect(),
    };
    (pi0, r_lo, r_hi, ch)
}

fn true_claim(pi0: &[B128], r_lo: &[B128], r_hi: &[B128]) -> B128 {
    let eq_lo = eq_expand_b128(r_lo);
    let eq_hi = eq_expand_b128(r_hi);
    let mut s = B128::ZERO;
    for (y, &p) in pi0.iter().enumerate() {
        for i in 0..PACK {
            if p.bit(i) {
                s = s + eq_lo[i] * eq_hi[y];
            }
        }
    }
    s
}

fn true_pi1_eval(pi0: &[B128], r_pp: &[F162]) -> F162 {
    let eq = eq_expand_f162(r_pp);
    pi0.iter()
        .zip(&eq)
        .fold(F162::ZERO, |a, (&p, &e)| a + F162::from_b128(p) * e)
}

#[test]
fn switch_end_to_end() {
    for l in [1usize, 2, 5, 8] {
        let (pi0, r_lo, r_hi, ch) = setup(l, 100 + l as u64);
        let claim = true_claim(&pi0, &r_lo, &r_hi);
        let proof = prove(&pi0, &r_lo, &r_hi, &ch);
        let z = verify(&proof, claim, &r_lo, &r_hi, &ch).expect("verify");
        assert_eq!(z, true_pi1_eval(&pi0, &ch.r_pp), "l={l}: opened wrong value");
    }
}

#[test]
fn switch_rejects_wrong_claim() {
    let l = 6;
    let (pi0, r_lo, r_hi, ch) = setup(l, 7);
    let claim = true_claim(&pi0, &r_lo, &r_hi);
    let proof = prove(&pi0, &r_lo, &r_hi, &ch);
    assert!(verify(&proof, claim + B128::ONE, &r_lo, &r_hi, &ch).is_err());
}

#[test]
fn switch_rejects_tampered_v() {
    let l = 6;
    let (pi0, r_lo, r_hi, ch) = setup(l, 9);
    let claim = true_claim(&pi0, &r_lo, &r_hi);
    let mut proof = prove(&pi0, &r_lo, &r_hi, &ch);
    proof.v[3] = proof.v[3] + B128(0x1234);
    assert!(verify(&proof, claim, &r_lo, &r_hi, &ch).is_err());
}

#[test]
fn switch_rejects_tampered_round() {
    let l = 6;
    let (pi0, r_lo, r_hi, ch) = setup(l, 11);
    let claim = true_claim(&pi0, &r_lo, &r_hi);
    let mut proof = prove(&pi0, &r_lo, &r_hi, &ch);
    proof.rounds[2][0] += F162::ONE;
    assert!(verify(&proof, claim, &r_lo, &r_hi, &ch).is_err());
}

#[test]
fn transparent_coeff_matches_definition() {
    let l = 5;
    let (_, _, r_hi, ch) = setup(l, 42);
    let batch = eq_expand_f162(&ch.r_prime);
    let tab = psi_table(&batch);
    let eq_hi = eq_expand_b128(&r_hi);
    let eq_pp = eq_expand_f162(&ch.r_pp);
    let want = eq_hi
        .iter()
        .zip(&eq_pp)
        .fold(F162::ZERO, |a, (&h, &v)| a + v * psi(&tab, h));
    assert_eq!(transparent_coeff(&r_hi, &ch.r_pp, &batch), want);
}

#[test]
fn stepwise_matches_batch() {
    let l = 7;
    let (pi0, r_lo, r_hi, ch) = setup(l, 314);
    let claim = true_claim(&pi0, &r_lo, &r_hi);
    let batch = eq_expand_f162(&ch.r_prime);

    let (v, eq_hi) = SwitchProver::partial_evals_and_eq(&pi0, &r_hi);
    let mut pv = SwitchProver::new(&pi0, &eq_hi, &batch);
    let mut vf = SwitchVerifier::start(&v, claim, &r_lo, &batch).expect("start");
    for round in 0..l {
        let m = pv.msg();
        let r = ch.r_pp[round];
        vf.round(m, r);
        pv.fold(r);
    }
    let opened = eval_pi1(&pi0, &ch.r_pp);
    assert_eq!(pv.final_eval(), opened);
    vf.finish(&r_hi, &ch.r_pp, &batch, opened).expect("finish");
}
