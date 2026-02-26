use std::time::{Duration, SystemTime};

/// Determines whether cron should run based on a lock timestamp and timeout.
pub fn should_run_cron(lock_acquired_at: Option<SystemTime>, timeout: Duration) -> bool {
    match lock_acquired_at {
        None => true,
        Some(locked_at) => match SystemTime::now().duration_since(locked_at) {
            Ok(elapsed) => elapsed >= timeout,
            Err(_) => true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_run_when_unlocked() {
        assert!(should_run_cron(None, Duration::from_secs(60)));
    }
}
