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
    if prefix.is_empty() {
        return Err(DatabaseError::InvalidTablePrefix(prefix.to_string()));
    }

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

pub const CORE_TABLES: [TableDefinition; 19] = [
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
        logical_name: "blogmeta",
        scope: TableScope::Global,
    },
    TableDefinition {
        logical_name: "registration_log",
        scope: TableScope::Global,
    },
    TableDefinition {
        logical_name: "blog_versions",
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
    TableDefinition {
        logical_name: "signups",
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
    pub fn add_option(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        autoload: bool,
    ) -> bool {
        let name = name.into();
        if self.values.contains_key(&name) {
            return false;
        }

        self.values.insert(
            name.clone(),
            OptionRecord {
                name,
                value: value.into(),
                autoload,
            },
        );
        self.alloptions_cache = None;
        true
    }

    pub fn set_option(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        autoload: Option<bool>,
    ) -> bool {
        let name = name.into();
        let value = value.into();
        let resolved_autoload = autoload.unwrap_or_else(|| {
            self.values
                .get(&name)
                .map(|existing| existing.autoload)
                .unwrap_or(true)
        });
        if let Some(existing) = self.values.get(&name) {
            if existing.value == value && existing.autoload == resolved_autoload {
                return false;
            }
        }

        self.values.insert(
            name.clone(),
            OptionRecord {
                name,
                value,
                autoload: resolved_autoload,
            },
        );
        self.alloptions_cache = None;
        true
    }

    pub fn get_option(&self, name: &str) -> Option<String> {
        self.values.get(name).map(|record| record.value.clone())
    }

    pub fn get_option_with_default(&self, name: &str, default: Option<&str>) -> Option<String> {
        self.get_option(name)
            .or_else(|| default.map(|value| value.to_string()))
    }

    pub fn get_multiple(&self, names: &[String]) -> HashMap<String, Option<String>> {
        names
            .iter()
            .map(|name| (name.clone(), self.get_option(name)))
            .collect()
    }

    pub fn get_option_record(&self, name: &str) -> Option<OptionRecord> {
        self.values.get(name).cloned()
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

    pub fn set_multiple(
        &mut self,
        entries: &HashMap<String, Value>,
        group: &str,
        ttl: Option<Duration>,
    ) -> usize {
        for (key, value) in entries {
            self.set(key.clone(), group.to_string(), value.clone(), ttl);
        }
        entries.len()
    }

    pub fn add_multiple(
        &mut self,
        entries: &HashMap<String, Value>,
        group: &str,
        ttl: Option<Duration>,
    ) -> HashMap<String, bool> {
        entries
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    self.add(key.clone(), group.to_string(), value.clone(), ttl),
                )
            })
            .collect()
    }

    pub fn replace_multiple(
        &mut self,
        entries: &HashMap<String, Value>,
        group: &str,
        ttl: Option<Duration>,
    ) -> HashMap<String, bool> {
        entries
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    self.replace(key.clone(), group.to_string(), value.clone(), ttl),
                )
            })
            .collect()
    }

    pub fn add(
        &mut self,
        key: impl Into<String>,
        group: impl Into<String>,
        value: Value,
        ttl: Option<Duration>,
    ) -> bool {
        let key = key.into();
        let group = group.into();
        let now = SystemTime::now();
        let entries = self.groups.entry(group).or_default();

        if let Some(existing) = entries.get(&key) {
            if !is_cache_entry_expired(existing, now) {
                return false;
            }
            entries.remove(&key);
        }

        let entry = CacheEntry {
            value,
            expires_at: ttl.map(|duration| now + duration),
        };
        entries.insert(key, entry);
        true
    }

    pub fn replace(
        &mut self,
        key: impl Into<String>,
        group: impl Into<String>,
        value: Value,
        ttl: Option<Duration>,
    ) -> bool {
        let key = key.into();
        let group = group.into();
        let now = SystemTime::now();
        let entries = self.groups.entry(group).or_default();

        if let Some(existing) = entries.get(&key) {
            if is_cache_entry_expired(existing, now) {
                entries.remove(&key);
                return false;
            }
        } else {
            return false;
        }

        let entry = CacheEntry {
            value,
            expires_at: ttl.map(|duration| now + duration),
        };
        entries.insert(key, entry);
        true
    }

    pub fn get(&mut self, key: &str, group: &str) -> Option<Value> {
        let now = SystemTime::now();
        let entries = self.groups.get_mut(group)?;
        if let Some(entry) = entries.get(key) {
            if is_cache_entry_expired(entry, now) {
                entries.remove(key);
                return None;
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

    pub fn get_multiple(&mut self, keys: &[String], group: &str) -> HashMap<String, Option<Value>> {
        keys.iter()
            .map(|key| (key.clone(), self.get(key, group)))
            .collect()
    }

    pub fn delete_multiple(&mut self, keys: &[String], group: &str) -> usize {
        keys.iter().filter(|key| self.delete(key, group)).count()
    }

    pub fn incr(&mut self, key: &str, group: &str, offset: u64) -> Option<i64> {
        let now = SystemTime::now();
        let entries = self.groups.get_mut(group)?;
        let entry = entries.get_mut(key)?;
        if is_cache_entry_expired(entry, now) {
            entries.remove(key);
            return None;
        }

        let current = parse_cache_numeric_value(&entry.value)?;
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let updated = current.saturating_add(offset);
        entry.value = Value::from(updated);
        Some(updated)
    }

    pub fn decr(&mut self, key: &str, group: &str, offset: u64) -> Option<i64> {
        let now = SystemTime::now();
        let entries = self.groups.get_mut(group)?;
        let entry = entries.get_mut(key)?;
        if is_cache_entry_expired(entry, now) {
            entries.remove(key);
            return None;
        }

        let current = parse_cache_numeric_value(&entry.value)?;
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let updated = (current.saturating_sub(offset)).max(0);
        entry.value = Value::from(updated);
        Some(updated)
    }

    pub fn flush_group(&mut self, group: &str) -> bool {
        self.groups.remove(group).is_some()
    }

    pub fn flush_all(&mut self) {
        self.groups.clear();
    }
}

fn is_cache_entry_expired(entry: &CacheEntry, now: SystemTime) -> bool {
    entry.expires_at.is_some_and(|expires_at| now >= expires_at)
}

fn parse_cache_numeric_value(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok())),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSite {
    pub blog_id: u64,
    pub domain: String,
    pub path: String,
    pub is_public: bool,
}

#[derive(Debug, Default, Clone)]
pub struct MultisiteResolver {
    sites: Vec<NetworkSite>,
}

impl MultisiteResolver {
    pub fn register_site(&mut self, site: NetworkSite) {
        self.sites.push(site);
        self.sites
            .sort_by(|left, right| right.path.len().cmp(&left.path.len()));
    }

    pub fn resolve(&self, domain: &str, request_path: &str) -> Option<NetworkSite> {
        let normalized_path = if request_path.starts_with('/') {
            request_path.to_string()
        } else {
            format!("/{request_path}")
        };

        self.sites
            .iter()
            .find(|site| {
                site.domain.eq_ignore_ascii_case(domain)
                    && normalized_path.starts_with(site.path.as_str())
            })
            .cloned()
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
    fn rejects_empty_prefix() {
        assert!(validate_table_prefix("").is_err());
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
    fn all_core_table_names_include_multisite_and_global_prefixes() {
        let names = all_core_table_names("wp_", Some(3)).expect("table names should resolve");
        assert_eq!(names.len(), CORE_TABLES.len());
        assert!(names.contains(&"wp_3_posts".to_string()));
        assert!(names.contains(&"wp_3_options".to_string()));
        assert!(names.contains(&"wp_users".to_string()));
        assert!(names.contains(&"wp_blogmeta".to_string()));
        assert!(names.contains(&"wp_registration_log".to_string()));
        assert!(names.contains(&"wp_blog_versions".to_string()));
        assert!(names.contains(&"wp_signups".to_string()));
        assert!(names.contains(&"wp_sitemeta".to_string()));
    }

    #[test]
    fn all_core_table_names_reject_invalid_blog_id() {
        let result = all_core_table_names("wp_", Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn multisite_global_tables_do_not_include_blog_id_prefix() {
        let names = all_core_table_names("wp_", Some(42)).expect("table names should resolve");
        assert!(names.contains(&"wp_blogs".to_string()));
        assert!(names.contains(&"wp_blogmeta".to_string()));
        assert!(names.contains(&"wp_registration_log".to_string()));
        assert!(names.contains(&"wp_blog_versions".to_string()));
        assert!(names.contains(&"wp_signups".to_string()));
        assert!(!names.contains(&"wp_42_blogmeta".to_string()));
        assert!(!names.contains(&"wp_42_blog_versions".to_string()));
        assert!(!names.contains(&"wp_42_signups".to_string()));
    }

    #[test]
    fn option_store_tracks_autoload_cache() {
        let mut store = OptionStore::default();
        store.set_option("blogname", "Example", Some(true));
        store.set_option("transient_timeout", "12345", Some(false));
        let alloptions = store.load_alloptions();
        assert_eq!(alloptions.get("blogname"), Some(&"Example".to_string()));
        assert!(!alloptions.contains_key("transient_timeout"));
    }

    #[test]
    fn option_store_invalidates_alloptions_after_mutations() {
        let mut store = OptionStore::default();
        store.set_option("blogname", "WordPress", Some(true));
        let initial = store.load_alloptions();
        assert_eq!(initial.get("blogname"), Some(&"WordPress".to_string()));

        store.set_option("blogname", "WordPress Rust", Some(true));
        let updated = store.load_alloptions();
        assert_eq!(updated.get("blogname"), Some(&"WordPress Rust".to_string()));

        assert!(store.delete_option("blogname"));
        let after_delete = store.load_alloptions();
        assert!(!after_delete.contains_key("blogname"));
    }

    #[test]
    fn option_store_reports_noop_updates() {
        let mut store = OptionStore::default();
        assert!(store.set_option("blogname", "WordPress", Some(true)));
        assert!(!store.set_option("blogname", "WordPress", Some(true)));
        assert!(store.set_option("blogname", "WordPress Rust", Some(true)));
        assert!(store.set_option("blogname", "WordPress Rust", Some(false)));
    }

    #[test]
    fn option_store_preserves_autoload_when_unspecified() {
        let mut store = OptionStore::default();
        assert!(store.set_option("blogname", "WordPress", Some(false)));
        assert!(store.set_option("blogname", "WordPress 2", None));
        let record = store
            .get_option_record("blogname")
            .expect("option should exist");
        assert_eq!(record.value, "WordPress 2");
        assert!(!record.autoload);
    }

    #[test]
    fn option_store_add_option_only_writes_when_missing() {
        let mut store = OptionStore::default();
        assert!(store.add_option("blogname", "WordPress", true));
        assert!(!store.add_option("blogname", "Different", false));

        let option = store
            .get_option_record("blogname")
            .expect("option should exist");
        assert_eq!(option.value, "WordPress");
        assert!(option.autoload);
    }

    #[test]
    fn option_store_returns_default_for_missing_option() {
        let store = OptionStore::default();
        assert_eq!(
            store.get_option_with_default("missing", Some("fallback")),
            Some("fallback".to_string())
        );
        assert_eq!(store.get_option_with_default("missing", None), None);
    }

    #[test]
    fn option_store_get_multiple_reports_missing_entries() {
        let mut store = OptionStore::default();
        assert!(store.set_option("blogname", "WordPress", Some(true)));
        let names = vec!["blogname".to_string(), "missing".to_string()];

        let values = store.get_multiple(&names);
        assert_eq!(values.get("blogname"), Some(&Some("WordPress".to_string())));
        assert_eq!(values.get("missing"), Some(&None));
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

    #[test]
    fn object_cache_add_only_stores_when_key_missing() {
        let mut cache = ObjectCache::default();
        assert!(cache.add(
            "post_1",
            "posts",
            Value::String("initial".to_string()),
            None
        ));
        assert!(!cache.add(
            "post_1",
            "posts",
            Value::String("new-value".to_string()),
            None
        ));
        assert_eq!(
            cache.get("post_1", "posts"),
            Some(Value::String("initial".to_string()))
        );
    }

    #[test]
    fn object_cache_entries_expire_based_on_ttl() {
        let mut cache = ObjectCache::default();
        cache.set(
            "post_1",
            "posts",
            Value::String("ephemeral".to_string()),
            Some(Duration::from_secs(0)),
        );
        assert_eq!(cache.get("post_1", "posts"), None);
    }

    #[test]
    fn object_cache_get_multiple_reports_hits_and_misses() {
        let mut cache = ObjectCache::default();
        cache.set("first", "posts", Value::String("one".to_string()), None);
        cache.set("second", "posts", Value::String("two".to_string()), None);

        let keys = vec![
            "first".to_string(),
            "second".to_string(),
            "missing".to_string(),
        ];
        let result = cache.get_multiple(&keys, "posts");
        assert_eq!(
            result.get("first"),
            Some(&Some(Value::String("one".to_string())))
        );
        assert_eq!(
            result.get("second"),
            Some(&Some(Value::String("two".to_string())))
        );
        assert_eq!(result.get("missing"), Some(&None));
    }

    #[test]
    fn object_cache_set_multiple_stores_all_entries() {
        let mut cache = ObjectCache::default();
        let entries = HashMap::from([
            ("first".to_string(), Value::String("one".to_string())),
            ("second".to_string(), Value::String("two".to_string())),
        ]);

        assert_eq!(cache.set_multiple(&entries, "posts", None), 2);
        assert_eq!(
            cache.get("first", "posts"),
            Some(Value::String("one".to_string()))
        );
        assert_eq!(
            cache.get("second", "posts"),
            Some(Value::String("two".to_string()))
        );
    }

    #[test]
    fn object_cache_add_multiple_only_adds_missing_keys() {
        let mut cache = ObjectCache::default();
        cache.set("existing", "posts", Value::String("seed".to_string()), None);
        let entries = HashMap::from([
            ("existing".to_string(), Value::String("new".to_string())),
            ("fresh".to_string(), Value::String("value".to_string())),
        ]);

        let result = cache.add_multiple(&entries, "posts", None);
        assert_eq!(result.get("existing"), Some(&false));
        assert_eq!(result.get("fresh"), Some(&true));
        assert_eq!(
            cache.get("existing", "posts"),
            Some(Value::String("seed".to_string()))
        );
        assert_eq!(
            cache.get("fresh", "posts"),
            Some(Value::String("value".to_string()))
        );
    }

    #[test]
    fn object_cache_replace_multiple_only_replaces_existing_keys() {
        let mut cache = ObjectCache::default();
        cache.set("existing", "posts", Value::String("seed".to_string()), None);
        let entries = HashMap::from([
            ("existing".to_string(), Value::String("updated".to_string())),
            ("missing".to_string(), Value::String("value".to_string())),
        ]);

        let result = cache.replace_multiple(&entries, "posts", None);
        assert_eq!(result.get("existing"), Some(&true));
        assert_eq!(result.get("missing"), Some(&false));
        assert_eq!(
            cache.get("existing", "posts"),
            Some(Value::String("updated".to_string()))
        );
        assert_eq!(cache.get("missing", "posts"), None);
    }

    #[test]
    fn object_cache_delete_multiple_removes_existing_keys() {
        let mut cache = ObjectCache::default();
        cache.set("first", "posts", Value::String("one".to_string()), None);
        cache.set("second", "posts", Value::String("two".to_string()), None);

        let keys = vec![
            "first".to_string(),
            "missing".to_string(),
            "second".to_string(),
        ];
        assert_eq!(cache.delete_multiple(&keys, "posts"), 2);
        assert_eq!(cache.get("first", "posts"), None);
        assert_eq!(cache.get("second", "posts"), None);
    }

    #[test]
    fn object_cache_replace_requires_existing_key() {
        let mut cache = ObjectCache::default();
        assert!(!cache.replace("post_1", "posts", Value::String("value".to_string()), None));

        cache.set(
            "post_1",
            "posts",
            Value::String("initial".to_string()),
            None,
        );
        assert!(cache.replace(
            "post_1",
            "posts",
            Value::String("updated".to_string()),
            None
        ));
        assert_eq!(
            cache.get("post_1", "posts"),
            Some(Value::String("updated".to_string()))
        );
    }

    #[test]
    fn object_cache_add_succeeds_after_expired_entry() {
        let mut cache = ObjectCache::default();
        assert!(cache.add(
            "post_1",
            "posts",
            Value::String("expired".to_string()),
            Some(Duration::from_secs(0)),
        ));
        assert!(cache.add("post_1", "posts", Value::String("fresh".to_string()), None));
        assert_eq!(
            cache.get("post_1", "posts"),
            Some(Value::String("fresh".to_string()))
        );
    }

    #[test]
    fn object_cache_incr_and_decr_support_numeric_values() {
        let mut cache = ObjectCache::default();
        cache.set("counter", "stats", Value::from(10), None);

        assert_eq!(cache.incr("counter", "stats", 2), Some(12));
        assert_eq!(cache.decr("counter", "stats", 5), Some(7));
        assert_eq!(cache.get("counter", "stats"), Some(Value::from(7)));
    }

    #[test]
    fn object_cache_decr_floors_at_zero() {
        let mut cache = ObjectCache::default();
        cache.set("counter", "stats", Value::from(3), None);

        assert_eq!(cache.decr("counter", "stats", 10), Some(0));
        assert_eq!(cache.get("counter", "stats"), Some(Value::from(0)));
    }

    #[test]
    fn object_cache_incr_rejects_non_numeric_values() {
        let mut cache = ObjectCache::default();
        cache.set(
            "counter",
            "stats",
            Value::String("not-a-number".to_string()),
            None,
        );

        assert_eq!(cache.incr("counter", "stats", 1), None);
        assert_eq!(cache.decr("counter", "stats", 1), None);
    }

    #[test]
    fn object_cache_flush_group_clears_only_target_group() {
        let mut cache = ObjectCache::default();
        cache.set("post_1", "posts", Value::String("cached".to_string()), None);
        cache.set(
            "user_1",
            "users",
            Value::String("cached-user".to_string()),
            None,
        );

        assert!(cache.flush_group("posts"));
        assert_eq!(cache.get("post_1", "posts"), None);
        assert_eq!(
            cache.get("user_1", "users"),
            Some(Value::String("cached-user".to_string()))
        );
    }

    #[test]
    fn object_cache_flush_all_clears_every_group() {
        let mut cache = ObjectCache::default();
        cache.set("post_1", "posts", Value::String("cached".to_string()), None);
        cache.set(
            "user_1",
            "users",
            Value::String("cached-user".to_string()),
            None,
        );

        cache.flush_all();

        assert_eq!(cache.get("post_1", "posts"), None);
        assert_eq!(cache.get("user_1", "users"), None);
    }

    #[test]
    fn resolves_multisite_by_domain_and_path() {
        let mut resolver = MultisiteResolver::default();
        resolver.register_site(NetworkSite {
            blog_id: 1,
            domain: "example.com".to_string(),
            path: "/".to_string(),
            is_public: true,
        });
        resolver.register_site(NetworkSite {
            blog_id: 2,
            domain: "example.com".to_string(),
            path: "/blog/".to_string(),
            is_public: true,
        });

        let resolved = resolver
            .resolve("example.com", "/blog/hello-world")
            .expect("site should resolve");
        assert_eq!(resolved.blog_id, 2);
    }

    #[test]
    fn multisite_resolver_returns_none_for_unknown_domain() {
        let mut resolver = MultisiteResolver::default();
        resolver.register_site(NetworkSite {
            blog_id: 1,
            domain: "example.com".to_string(),
            path: "/".to_string(),
            is_public: true,
        });

        assert!(resolver.resolve("invalid.example", "/").is_none());
    }
}
