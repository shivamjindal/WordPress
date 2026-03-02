use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CronEvent {
    pub hook: String,
    pub timestamp: u64,
    pub schedule: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CronScheduler {
    events: Vec<CronEvent>,
    lock_acquired_at: Option<SystemTime>,
}

impl CronScheduler {
    pub fn schedule_event(&mut self, event: CronEvent) {
        self.events.push(event);
        self.events.sort_by_key(|event| event.timestamp);
    }

    pub fn due_events(&mut self, now_timestamp: u64) -> Vec<CronEvent> {
        let split_index = self
            .events
            .iter()
            .position(|event| event.timestamp > now_timestamp)
            .unwrap_or(self.events.len());
        self.events.drain(..split_index).collect()
    }

    pub fn next_event_timestamp(&self) -> Option<u64> {
        self.events.first().map(|event| event.timestamp)
    }

    pub fn acquire_lock(&mut self, timeout: Duration) -> bool {
        if should_run_cron(self.lock_acquired_at, timeout) {
            self.lock_acquired_at = Some(SystemTime::now());
            true
        } else {
            false
        }
    }

    pub fn release_lock(&mut self) {
        self.lock_acquired_at = None;
    }

    pub fn lock_acquired_at(&self) -> Option<SystemTime> {
        self.lock_acquired_at
    }
}

pub fn parse_doing_wp_cron(query_string: &str) -> Option<String> {
    query_string
        .split('&')
        .filter_map(|entry| entry.split_once('='))
        .find(|(key, _)| key.trim() == "doing_wp_cron")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_run_when_unlocked() {
        assert!(should_run_cron(None, Duration::from_secs(60)));
    }

    #[test]
    fn scheduler_returns_due_events_in_order() {
        let mut scheduler = CronScheduler::default();
        scheduler.schedule_event(CronEvent {
            hook: "future_hook".to_string(),
            timestamp: 200,
            schedule: Some("hourly".to_string()),
            args: vec![],
        });
        scheduler.schedule_event(CronEvent {
            hook: "due_hook".to_string(),
            timestamp: 100,
            schedule: None,
            args: vec!["1".to_string()],
        });

        let due = scheduler.due_events(150);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].hook, "due_hook");
        assert_eq!(scheduler.next_event_timestamp(), Some(200));
    }

    #[test]
    fn scheduler_lock_respects_timeout() {
        let mut scheduler = CronScheduler::default();
        assert!(scheduler.acquire_lock(Duration::from_secs(60)));
        assert!(!scheduler.acquire_lock(Duration::from_secs(60)));
        scheduler.release_lock();
        assert!(scheduler.acquire_lock(Duration::from_secs(60)));
    }

    #[test]
    fn parses_doing_wp_cron_token() {
        let token = parse_doing_wp_cron("foo=1&doing_wp_cron=173847");
        assert_eq!(token, Some("173847".to_string()));
    }

    #[test]
    fn parse_doing_wp_cron_ignores_empty_value() {
        let token = parse_doing_wp_cron("foo=1&doing_wp_cron=&bar=2");
        assert_eq!(token, None);
    }

    #[test]
    fn should_run_cron_allows_when_lock_in_future() {
        let future_lock = SystemTime::now() + Duration::from_secs(30);
        assert!(should_run_cron(Some(future_lock), Duration::from_secs(60)));
    }
}
