use std::collections::HashMap;

/// Parses an HTTP Cookie header into a key/value map.
pub fn parse_cookie_header(cookie_header: &str) -> HashMap<String, String> {
    cookie_header
        .split(';')
        .filter_map(|pair| {
            let mut pieces = pair.trim().splitn(2, '=');
            let key = pieces.next()?.trim();
            let value = pieces.next().unwrap_or_default().trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

/// Placeholder nonce verification hook.
///
/// This function intentionally enforces strict presence checks today.
/// Detailed nonce parity with WordPress is implemented in later phases.
pub fn verify_nonce_presence(nonce: Option<&str>) -> bool {
    nonce.map(str::trim).is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cookie_header_pairs() {
        let cookies = parse_cookie_header("foo=bar; test=value");
        assert_eq!(cookies.get("foo"), Some(&"bar".to_string()));
        assert_eq!(cookies.get("test"), Some(&"value".to_string()));
    }

    #[test]
    fn requires_nonce_content() {
        assert!(!verify_nonce_presence(None));
        assert!(!verify_nonce_presence(Some("   ")));
        assert!(verify_nonce_presence(Some("abc123")));
    }
}
