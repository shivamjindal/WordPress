use std::collections::BTreeMap;

use serde_json::Value;

type ActionCallback = Box<dyn Fn(&[Value]) + Send + Sync + 'static>;
type FilterCallback = Box<dyn Fn(Value, &[Value]) -> Value + Send + Sync + 'static>;

/// Minimal action/filter dispatcher for Rust migration scaffolding.
#[derive(Default)]
pub struct HookDispatcher {
    actions: BTreeMap<String, BTreeMap<i32, Vec<ActionCallback>>>,
    filters: BTreeMap<String, BTreeMap<i32, Vec<FilterCallback>>>,
}

impl HookDispatcher {
    pub fn add_action(&mut self, name: impl Into<String>, priority: i32, callback: ActionCallback) {
        self.actions
            .entry(name.into())
            .or_default()
            .entry(priority)
            .or_default()
            .push(callback);
    }

    pub fn do_action(&self, name: &str, args: &[Value]) {
        if let Some(by_priority) = self.actions.get(name) {
            for callbacks in by_priority.values() {
                for callback in callbacks {
                    callback(args);
                }
            }
        }
    }

    pub fn add_filter(&mut self, name: impl Into<String>, priority: i32, callback: FilterCallback) {
        self.filters
            .entry(name.into())
            .or_default()
            .entry(priority)
            .or_default()
            .push(callback);
    }

    pub fn apply_filters(&self, name: &str, mut value: Value, args: &[Value]) -> Value {
        if let Some(by_priority) = self.filters.get(name) {
            for callbacks in by_priority.values() {
                for callback in callbacks {
                    value = callback(value, args);
                }
            }
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_apply_in_priority_order() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_filter(
            "sample",
            20,
            Box::new(|value, _| {
                let current = value.as_str().unwrap_or_default();
                Value::String(format!("{current}-late"))
            }),
        );
        dispatcher.add_filter(
            "sample",
            10,
            Box::new(|value, _| {
                let current = value.as_str().unwrap_or_default();
                Value::String(format!("{current}-early"))
            }),
        );

        let result = dispatcher.apply_filters("sample", Value::String("value".to_string()), &[]);
        assert_eq!(result, Value::String("value-early-late".to_string()));
    }
}
