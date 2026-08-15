use serde::{Deserialize, Serialize};
use std::time::Duration;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackoffType {
    Fixed,
    Linear,
    Exponential,
    ExponentialWithJitter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_type: BackoffType,
    pub initial_interval_secs: u64,
    pub max_interval_secs: u64,
    pub jitter_factor: f64, // e.g. 0.2 for 20%
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_type: BackoffType::ExponentialWithJitter,
            initial_interval_secs: 5,
            max_interval_secs: 300,
            jitter_factor: 0.2,
        }
    }
}

impl RetryPolicy {
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let base_delay = match self.backoff_type {
            BackoffType::Fixed => self.initial_interval_secs,
            BackoffType::Linear => self.initial_interval_secs * (attempt as u64),
            BackoffType::Exponential | BackoffType::ExponentialWithJitter => {
                let factor = 2u64.saturating_pow(attempt.saturating_sub(1));
                self.initial_interval_secs.saturating_mul(factor)
            }
        };

        let clamped = base_delay.min(self.max_interval_secs) as f64;

        if self.backoff_type == BackoffType::ExponentialWithJitter && self.jitter_factor > 0.0 {
            let mut rng = rand::thread_rng();
            let jitter_range = clamped * self.jitter_factor;
            let jitter: f64 = rng.gen_range(-jitter_range..=jitter_range);
            let final_secs = (clamped + jitter).max(0.1);
            Duration::from_millis((final_secs * 1000.0) as u64)
        } else {
            Duration::from_secs(clamped as u64)
        }
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_backoff() {
        let policy = RetryPolicy {
            max_attempts: 3,
            backoff_type: BackoffType::Fixed,
            initial_interval_secs: 5,
            max_interval_secs: 60,
            jitter_factor: 0.0,
        };
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(5));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(5));
        assert_eq!(policy.calculate_delay(3), Duration::from_secs(5));
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy {
            max_attempts: 5,
            backoff_type: BackoffType::Exponential,
            initial_interval_secs: 5,
            max_interval_secs: 50,
            jitter_factor: 0.0,
        };
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(5));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(10));
        assert_eq!(policy.calculate_delay(3), Duration::from_secs(20));
        assert_eq!(policy.calculate_delay(4), Duration::from_secs(40));
        assert_eq!(policy.calculate_delay(5), Duration::from_secs(50)); // capped
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy { max_attempts: 3, ..Default::default() };
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
    }
}
