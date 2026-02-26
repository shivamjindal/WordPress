/// Represents a normalized front-end request used by content pipeline adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontRequest {
    pub path: String,
    pub query_string: String,
}

impl FrontRequest {
    pub fn from_parts(path: &str, query_string: &str) -> Self {
        let normalized_path = if path.trim().is_empty() {
            "/"
        } else {
            path.trim()
        };
        Self {
            path: normalized_path.to_string(),
            query_string: query_string.trim().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_defaults_to_root() {
        let request = FrontRequest::from_parts("", "");
        assert_eq!(request.path, "/");
    }
}
