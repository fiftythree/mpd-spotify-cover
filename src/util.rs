use std::fmt::Display;
use datetime::{Duration, Instant};

pub fn offset_time_by_expiration(expires_in: i64) -> i64 {
    (Instant::now() + Duration::of(expires_in)).seconds()
}

pub fn either_contains(first: &str, second: &str) -> bool {
    first.contains(second) || second.contains(first)
}