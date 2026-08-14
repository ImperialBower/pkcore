//! Module 1 — Modular arithmetic and finite groups.
//!
//! Implement modular exponentiation and use it to *verify* the structure of
//! our toy group: that G generates a subgroup of prime order Q inside the
//! multiplicative group mod the safe prime P = 2Q + 1.
//!
//! Mantra: **elements live mod P, exponents live mod Q.**

use crate::exercise;

/// (a * b) mod m without overflow. Hint: widen to u128.
pub fn mod_mul(a: u64, b: u64, m: u64) -> u64 {
    exercise!("m1: multiply in u128, reduce mod m", {
        ((a as u128 * b as u128) % m as u128) as u64
    })
}

/// a^e mod m by square-and-multiply. This is the workhorse of everything —
/// every encryption, proof, and verification below is made of this.
pub fn mod_exp(mut a: u64, mut e: u64, m: u64) -> u64 {
    exercise!("m1: square-and-multiply; loop over bits of e", {
        let mut acc: u64 = 1 % m;
        a %= m;
        while e > 0 {
            if e & 1 == 1 {
                acc = mod_mul(acc, a, m);
            }
            a = mod_mul(a, a, m);
            e >>= 1;
        }
        acc
    })
}

/// The multiplicative order of `a` mod p: the smallest k >= 1 with a^k = 1.
/// Brute force is fine at this size — that's the point of small numbers.
pub fn element_order(a: u64, p: u64) -> u64 {
    exercise!("m1: multiply a into an accumulator until it hits 1", {
        let mut acc = a % p;
        let mut k = 1;
        while acc != 1 {
            acc = mod_mul(acc, a, p);
            k += 1;
        }
        k
    })
}

/// All Q distinct powers of g: the subgroup itself, materialized. (Only
/// possible because the group is tiny — this is your inspection window.)
pub fn subgroup_elements(g: u64, q: u64, p: u64) -> Vec<u64> {
    exercise!("m1: collect g^0 .. g^(q-1) mod p", {
        let mut v = Vec::with_capacity(q as usize);
        let mut acc = 1u64;
        for _ in 0..q {
            v.push(acc);
            acc = mod_mul(acc, g, p);
        }
        v
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G, P, Q};

    #[test]
    fn m1_mod_exp_basics() {
        assert_eq!(mod_exp(2, 10, 1000), 24); // 1024 mod 1000
        assert_eq!(mod_exp(3, 0, 7), 1);
        assert_eq!(mod_exp(0, 5, 7), 0);
        // Fermat: a^(p-1) = 1 mod p for prime p, a not divisible by p.
        assert_eq!(mod_exp(2, P - 1, P), 1);
        assert_eq!(mod_exp(123, P - 1, P), 1);
    }

    #[test]
    fn m1_g_generates_prime_order_subgroup() {
        // G has order exactly Q: G^Q = 1 and no smaller power hits 1.
        assert_eq!(mod_exp(G, Q, P), 1);
        assert_eq!(element_order(G, P), Q);
    }

    #[test]
    fn m1_subgroup_is_all_distinct_and_closed() {
        let sub = subgroup_elements(G, Q, P);
        assert_eq!(sub.len(), Q as usize);
        let mut sorted = sub.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), Q as usize, "all Q powers are distinct");
        // Closure: product of two subgroup elements is in the subgroup.
        let x = sub[17];
        let y = sub[101];
        assert!(sorted.binary_search(&mod_mul(x, y, P)).is_ok());
    }

    #[test]
    fn m1_lagrange_every_subgroup_element_has_order_dividing_q() {
        // Q is prime, so every non-identity element of the subgroup has order
        // exactly Q — i.e. EVERY such element is itself a generator.
        for &e in subgroup_elements(G, Q, P).iter().skip(1).take(20) {
            assert_eq!(element_order(e, P), Q);
        }
    }
}
