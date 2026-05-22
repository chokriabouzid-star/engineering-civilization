/// حساب GCD — pure، لا side effects
pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// فحص أولية — pure
pub fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    let sqrt = (n as f64).sqrt() as u64;
    !(3..=sqrt).step_by(2).any(|i| n % i == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gcd() { assert_eq!(gcd(12, 8), 4); }
    #[test]
    fn test_prime() { assert!(is_prime(7)); assert!(!is_prime(9)); }
    #[test]
    fn test_gcd_zero() { assert_eq!(gcd(5, 0), 5); }
}
