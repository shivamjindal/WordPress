use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[error("invalid blog id: {0}")]
    InvalidBlogId(u64),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableScope {
    Global,
    Blog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDefinition {
    pub logical_name: &'static str,
    pub scope: TableScope,
}

pub const CORE_TABLES: [TableDefinition; 15] = [
    TableDefinition {
        logical_name: "posts",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "postmeta",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "comments",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "commentmeta",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "terms",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "term_taxonomy",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "term_relationships",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "termmeta",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "options",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "links",
        scope: TableScope::Blog,
    },
    TableDefinition {
        logical_name: "users",
        scope: TableScope::Global,
    },
    TableDefinition {
        logical_name: "usermeta",
        scope: TableScope::Global,
    },
    TableDefinition {
        logical_name: "blogs",
        scope: TableScope::Global,
    },
    TableDefinition {
        logical_name: "site",
        scope: TableScope::Global,
    },
    TableDefinition {
        logical_name: "sitemeta",
        scope: TableScope::Global,
    },
];

/// Resolves a logical table name to its concrete table name.
pub fn resolve_table_name(
    table_prefix: &str,
    table: &TableDefinition,
    blog_id: Option<u64>,
) -> Result<String, DatabaseError> {
    validate_table_prefix(table_prefix)?;
    let blog_prefix = match table.scope {
        TableScope::Global => table_prefix.to_string(),
        TableScope::Blog => match blog_id.unwrap_or(1) {
            0 => return Err(DatabaseError::InvalidBlogId(0)),
            1 => table_prefix.to_string(),
            id => format!("{table_prefix}{id}_"),
        },
    };
    Ok(format!("{blog_prefix}{}", table.logical_name))
}

pub fn all_core_table_names(
    table_prefix: &str,
    blog_id: Option<u64>,
) -> Result<Vec<String>, DatabaseError> {
    CORE_TABLES
        .iter()
        .map(|definition| resolve_table_name(table_prefix, definition, blog_id))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionRecord {
    pub name: String,
    pub value: String,
    pub autoload: bool,
}

#[derive(Debug, Default, Clone)]
pub struct OptionStore {
    values: HashMap<String, OptionRecord>,
    alloptions_cache: Option<HashMap<String, String>>,
}

impl OptionStore {
    pub fn set_option(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        autoload: bool,
    ) {
        let name = name.into();
        self.values.insert(
            name.clone(),
            OptionRecord {
                name,
                value: value.into(),
                autoload,
            },
        );
        self.alloptions_cache = None;
    }

    pub fn get_option(&self, name: &str) -> Option<String> {
        self.values.get(name).map(|record| record.value.clone())
    }

    pub fn delete_option(&mut self, name: &str) -> bool {
        let deleted = self.values.remove(name).is_some();
        if deleted {
            self.alloptions_cache = None;
        }
        deleted
    }

    /// Equivalent to WordPress `alloptions` cache behavior:
    /// returns autoloaded options and memoizes the snapshot.
    pub fn load_alloptions(&mut self) -> HashMap<String, String> {
        if let Some(cache) = &self.alloptions_cache {
            return cache.clone();
        }

        let autoloaded = self
            .values
            .iter()
            .filter(|(_, record)| record.autoload)
            .map(|(name, record)| (name.clone(), record.value.clone()))
            .collect::<HashMap<_, _>>();
        self.alloptions_cache = Some(autoloaded.clone());
        autoloaded
    }

    pub fn snapshot(&self) -> HashMap<String, String> {
        self.values
            .iter()
            .map(|(name, record)| (name.clone(), record.value.clone()))
            .collect()
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    expires_at: Option<SystemTime>,
}

#[derive(Debug, Default)]
pub struct ObjectCache {
    groups: HashMap<String, HashMap<String, CacheEntry>>,
}

impl ObjectCache {
    pub fn set(
        &mut self,
        key: impl Into<String>,
        group: impl Into<String>,
        value: Value,
        ttl: Option<Duration>,
    ) {
        let entry = CacheEntry {
            value,
            expires_at: ttl.map(|duration| SystemTime::now() + duration),
        };
        self.groups
            .entry(group.into())
            .or_default()
            .insert(key.into(), entry);
    }

    pub fn get(&mut self, key: &str, group: &str) -> Option<Value> {
        let now = SystemTime::now();
        let entries = self.groups.get_mut(group)?;
        if let Some(entry) = entries.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if now >= expires_at {
                    entries.remove(key);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn delete(&mut self, key: &str, group: &str) -> bool {
        self.groups
            .get_mut(group)
            .and_then(|entries| entries.remove(key))
            .is_some()
    }

    pub fn flush_group(&mut self, group: &str) -> bool {
        self.groups.remove(group).is_some()
    }

    pub fn flush_all(&mut self) {
        self.groups.clear();
    }
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

    #[test]
    fn resolves_multisite_blog_table_name() {
        let table = CORE_TABLES
            .iter()
            .find(|table| table.logical_name == "options")
            .expect("options table should exist");
        let name = resolve_table_name("wp_", table, Some(3)).expect("table name should resolve");
        assert_eq!(name, "wp_3_options");
    }

    #[test]
    fn resolves_global_table_name_with_base_prefix() {
        let table = CORE_TABLES
            .iter()
            .find(|table| table.logical_name == "users")
            .expect("users table should exist");
        let name = resolve_table_name("wp_", table, Some(7)).expect("table name should resolve");
        assert_eq!(name, "wp_users");
    }

    #[test]
    fn option_store_tracks_autoload_cache() {
        let mut store = OptionStore::default();
        store.set_option("blogname", "Example", true);
        store.set_option("transient_timeout", "12345", false);
        let alloptions = store.load_alloptions();
        assert_eq!(alloptions.get("blogname"), Some(&"Example".to_string()));
        assert!(!alloptions.contains_key("transient_timeout"));
    }

    #[test]
    fn object_cache_supports_groups() {
        let mut cache = ObjectCache::default();
        cache.set("post_1", "posts", Value::String("cached".to_string()), None);
        assert_eq!(
            cache.get("post_1", "posts"),
            Some(Value::String("cached".to_string()))
        );
        assert!(cache.delete("post_1", "posts"));
        assert_eq!(cache.get("post_1", "posts"), None);
    }
}
