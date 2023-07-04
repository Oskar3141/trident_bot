pub fn binomial_coefficient(n: u128, k: u128) -> u128 {  
    if n < k { return 0; }
    if n == k { return 1; }
    if k == 0 { return 1; }
    if k == 1 { return n; }
    if k > n / 2 {
        binomial_coefficient(n, n - k - 1) * (n - k + 1) / k
    } else {
        binomial_coefficient(n, k - 1) * (n - k + 1) / k
    }
}

pub fn bernoullis_scheme(n: u128, k: u128, p: f64) -> f64 {
    binomial_coefficient(n, k) as f64 * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32)
}