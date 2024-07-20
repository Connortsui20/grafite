//! Utility and helper functions for hashing and prime number generation.

use rand::prelude::*;

/// Generates a random 64-bit number.
pub fn gen_random() -> u64 {
    rand::thread_rng().gen()
}

/// Checks if a number is prime.
pub fn is_prime(n: u64) -> bool {
    match n {
        0 | 1 => false,
        2 => true,
        _ if n % 2 == 0 => false,
        _ => !(1..)
            .map(|x| 2 * x + 1)
            .take_while(|&x| x * x <= n)
            .any(|factor| n % factor == 0),
    }
}

/// Generates a random number until it generates a prime, and then returns that prime number.
pub fn gen_prime() -> u64 {
    let mut rng = rand::thread_rng();

    loop {
        let attempt = rng.gen::<u16>() as u64;

        if is_prime(attempt) {
            return attempt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        let primes = [2, 3, 5, 7, 11, 13, 17, 19];

        assert!(primes.iter().copied().all(is_prime));
    }
}
