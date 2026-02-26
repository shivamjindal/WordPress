/// Parses the action from admin-ajax/admin-post style query payloads.
pub fn extract_action(query_pairs: &[(&str, &str)]) -> Option<String> {
    query_pairs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("action"))
        .and_then(|(_, value)| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_action_if_present() {
        let action = extract_action(&[("foo", "bar"), ("action", "heartbeat")]);
        assert_eq!(action, Some("heartbeat".to_string()));
    }
}
