fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    for divisor in 2..n {
        if n % divisor == 0 { return false; }
    }
    true
}

fn unfinished() {
    // TODO: replace this placeholder
    println!("unfinished");
}
