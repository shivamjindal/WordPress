use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub table_prefix: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: String::new(),
            database: "wordpress".to_string(),
            table_prefix: "wp_".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("invalid table prefix: {0}")]
    InvalidTablePrefix(String),
}

/// Validates a WordPress table prefix using core-compatible constraints.
pub fn validate_table_prefix(prefix: &str) -> Result<(), DatabaseError> {
    if !prefix
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(DatabaseError::InvalidTablePrefix(prefix.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_prefix() {
        assert!(validate_table_prefix("wp_").is_ok());
    }

    #[test]
    fn rejects_invalid_prefix() {
        assert!(validate_table_prefix("wp-prefix").is_err());
    }
}
