use num_bigint::{BigUint, ToBigUint};
use num_integer::Integer;
use num_traits::{One, ToPrimitive, Zero};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: aks <n>");
        std::process::exit(2);
    }
    let n = BigUint::parse_bytes(args[1].as_bytes(), 10).expect("invalid integer");
    let is_prime = aks_is_prime(&n);
    println!("{}", if is_prime { "prime" } else { "composite" });
}

/// Deterministic AKS primality test (educational, not optimized).
pub fn aks_is_prime(n: &BigUint) -> bool {
    // Handle small n
    if *n < 2u32.to_biguint().unwrap() {
        return false;
    }
    if *n == 2u32.to_biguint().unwrap() || *n == 3u32.to_biguint().unwrap() {
        return true;
    }
    if n.is_even() {
        return false;
    }

    // 1) Perfect power check
    if is_perfect_power(n) {
        return false;
    }

    // 2) Find r such that ord_r(n) > (log2 n)^2
    let log2n = log2_biguint_approx(n);
    let max_k = (log2n * log2n).ceil() as u64;

    let r = find_smallest_r(n, max_k);

    // 3) Check gcd(a, n) for 2..=r
    for a in 2u64..=r {
        let g = n.gcd(&a.to_biguint().unwrap());
        if g > One::one() && g < *n {
            return false;
        }
    }

    // 4) If n <= r, prime
    if let Some(n_u64) = n.to_u64() {
        if n_u64 <= r {
            return true;
        }
    } else {
        // n doesn't fit u64; definitely > r for our search bounds
    }

    // 5) Polynomial congruence check
    let phi_r = euler_phi_u64(r);
    let limit = ((phi_r as f64).sqrt() * log2n).floor() as u64;

    for a in 1u64..=limit.max(1) {
        if !aks_poly_congruence_holds(n, r, a) {
            return false;
        }
    }

    true
}

/// Approximate log2(n) as f64 using bit length.
fn log2_biguint_approx(n: &BigUint) -> f64 {
    // bit_length ~ floor(log2(n)) + 1
    let bits = n.bits();
    if bits == 0 {
        0.0
    } else {
        (bits - 1) as f64
    }
}

/// Perfect power: check if n = a^b for integers a>1, b>1.
fn is_perfect_power(n: &BigUint) -> bool {
    // For b up to floor(log2(n))
    let max_b = n.bits().saturating_sub(1) as u32;
    for b in 2..=max_b {
        if let Some(a) = integer_nth_root(n, b) {
            // verify a^b == n
            if a.pow(b) == *n {
                return true;
            }
        }
    }
    false
}

/// Integer b-th root by binary search; returns floor(root) as BigUint.
fn integer_nth_root(n: &BigUint, b: u32) -> Option<BigUint> {
    if *n < 2u32.to_biguint().unwrap() {
        return Some(n.clone());
    }
    // Upper bound: 2^(bits/b + 1)
    let bits = n.bits() as u32;
    let hi_bits = (bits / b) + 2;
    let mut lo = BigUint::one();
    let mut hi = BigUint::one() << hi_bits;

    while lo <= hi {
        let mid: BigUint = (&lo + &hi) >> 1;
        let p = mid.pow(b);
        if p == *n {
            return Some(mid);
        } else if p < *n {
            lo = &mid + 1u32;
        } else {
            if mid.is_zero() {
                break;
            }
            hi = &mid - 1u32;
        }
    }
    // return floor root
    Some(hi)
}

/// Find smallest r such that ord_r(n) > max_k, using brute force for r.
fn find_smallest_r(n: &BigUint, max_k: u64) -> u64 {
    // Practical note: AKS theory bounds r by poly(log n), but this brute search
    // is for educational use and moderate n only.
    let mut r = 2u64;
    loop {
        if n.gcd(&r.to_biguint().unwrap()) != One::one() {
            r += 1;
            continue;
        }
        let ord = multiplicative_order_mod_u64(n, r, max_k);
        if ord > max_k {
            return r;
        }
        r += 1;
    }
}

/// Compute multiplicative order of n mod r, but stop early once it exceeds max_k.
/// Returns order if <= max_k, else returns max_k+1.
fn multiplicative_order_mod_u64(n: &BigUint, r: u64, max_k: u64) -> u64 {
    let r_big = r.to_biguint().unwrap();
    let n_mod_r = (n % &r_big).to_u64().unwrap_or(0) % r;
    if n_mod_r == 0 {
        return 0;
    }

    let mut x = 1u64 % r;
    for k in 1..=max_k {
        x = mul_mod_u64(x, n_mod_r, r);
        if x == 1u64 % r {
            return k;
        }
    }
    max_k + 1
}

fn mul_mod_u64(a: u64, b: u64, m: u64) -> u64 {
    // safe for moderate m; uses u128
    ((a as u128 * b as u128) % (m as u128)) as u64
}

/// Euler's totient for u64 r (simple factorization).
fn euler_phi_u64(r: u64) -> u64 {
    let mut n = r;
    let mut result = r;
    let mut p = 2u64;
    while p * p <= n {
        if n % p == 0 {
            while n % p == 0 {
                n /= p;
            }
            result = result / p * (p - 1);
        }
        p += if p == 2 { 1 } else { 2 };
    }
    if n > 1 {
        result = result / n * (n - 1);
    }
    result
}

/// Check AKS polynomial congruence for a given 'a':
/// (x + a)^n ≡ x^n + a  (mod x^r - 1, n)
fn aks_poly_congruence_holds(n: &BigUint, r: u64, a: u64) -> bool {
    // Compute left = (x + a)^n mod (x^r - 1, n)
    let left = poly_pow_x_plus_a_mod(n, r, a);

    // Compute right = x^n + a mod (x^r - 1, n)
    // x^n mod (x^r - 1) => x^(n mod r)
    let n_mod_r = (n % r.to_biguint().unwrap()).to_u64().unwrap();
    let mut right = vec![BigUint::zero(); r as usize];
    right[n_mod_r as usize] = BigUint::one();
    right[0] = (right[0].clone() + a.to_biguint().unwrap()) % n;

    poly_eq_mod_n(&left, &right, n)
}

/// Compare polynomials coefficient-wise mod n.
fn poly_eq_mod_n(p: &[BigUint], q: &[BigUint], n: &BigUint) -> bool {
    if p.len() != q.len() {
        return false;
    }
    for i in 0..p.len() {
        if (&p[i] % n) != (&q[i] % n) {
            return false;
        }
    }
    true
}

/// Polynomial multiplication mod (x^r - 1, n).
/// Polynomials are represented as Vec<BigUint> of length r.
fn poly_mul_mod(p: &[BigUint], q: &[BigUint], n: &BigUint) -> Vec<BigUint> {
    let r = p.len();
    let mut out = vec![BigUint::zero(); r];
    for i in 0..r {
        if p[i].is_zero() {
            continue;
        }
        for j in 0..r {
            if q[j].is_zero() {
                continue;
            }
            let k = (i + j) % r;
            let term = (&p[i] * &q[j]) % n;
            out[k] = (&out[k] + &term) % n;
        }
    }
    out
}

/// Polynomial exponentiation: (x + a)^n mod (x^r - 1, n).
fn poly_pow_x_plus_a_mod(n: &BigUint, r: u64, a: u64) -> Vec<BigUint> {
    let r_usize = r as usize;

    // base polynomial: x + a
    let mut base = vec![BigUint::zero(); r_usize];
    base[0] = a.to_biguint().unwrap() % n;
    base[1 % r_usize] = BigUint::one();

    // result polynomial: 1
    let mut result = vec![BigUint::zero(); r_usize];
    result[0] = BigUint::one();

    // exponentiation by squaring
    let mut exp = n.clone();
    while !exp.is_zero() {
        if (&exp & BigUint::one()) == BigUint::one() {
            result = poly_mul_mod(&result, &base, n);
        }
    
        exp >>= 1;
        if !exp.is_zero() {
            base = poly_mul_mod(&base, &base, n);
        }
    }
    
    result
}
