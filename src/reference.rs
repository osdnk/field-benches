pub type F162 = [u64; 3];

fn bit(v: &[u64], i: usize) -> bool {
    (v[i / 64] >> (i % 64)) & 1 == 1
}

fn flip(v: &mut [u64], i: usize) {
    v[i / 64] ^= 1u64 << (i % 64);
}

fn xor_shifted(acc: &mut [u64], src: &[u64], sh: usize) {
    for i in 0..src.len() * 64 {
        if bit(src, i) {
            flip(acc, i + sh);
        }
    }
}

pub fn mul162(a: F162, b: F162) -> F162 {
    let mut p = [0u64; 6];
    for i in 0..162 {
        if bit(&a, i) {
            xor_shifted(&mut p, &b, i);
        }
    }
    for j in (162..324).rev() {
        if bit(&p, j) {
            flip(&mut p, j);
            flip(&mut p, j - 162);
            flip(&mut p, j - 81);
        }
    }
    [p[0], p[1], p[2]]
}

pub fn canonical162(a: F162) -> F162 {
    [a[0], a[1], a[2] & ((1u64 << 34) - 1)]
}

pub fn mul_polyval(a: u128, b: u128) -> u128 {
    let av = [a as u64, (a >> 64) as u64];
    let bv = [b as u64, (b >> 64) as u64];
    let mut p = [0u64; 4];
    for i in 0..128 {
        if bit(&av, i) {
            xor_shifted(&mut p, &bv, i);
        }
    }
    for i in 0..128 {
        if bit(&p, i) {
            flip(&mut p, i);
            flip(&mut p, i + 121);
            flip(&mut p, i + 126);
            flip(&mut p, i + 127);
            flip(&mut p, i + 128);
        }
    }
    (p[2] as u128) | ((p[3] as u128) << 64)
}

pub fn mul_ghash(a: u128, b: u128) -> u128 {
    let av = [a as u64, (a >> 64) as u64];
    let bv = [b as u64, (b >> 64) as u64];
    let mut p = [0u64; 4];
    for i in 0..128 {
        if bit(&av, i) {
            xor_shifted(&mut p, &bv, i);
        }
    }
    for j in (128..256).rev() {
        if bit(&p, j) {
            flip(&mut p, j);
            flip(&mut p, j - 128);
            flip(&mut p, j - 121);
            flip(&mut p, j - 126);
            flip(&mut p, j - 127);
        }
    }
    (p[0] as u128) | ((p[1] as u128) << 64)
}
