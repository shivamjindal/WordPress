use std::collections::BTreeMap;

use serde_json::Value;

type ActionCallback = Box<dyn Fn(&[Value]) + Send + Sync + 'static>;
type FilterCallback = Box<dyn Fn(Value, &[Value]) -> Value + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HookId(u64);

struct ActionRegistration {
    id: HookId,
    accepted_args: usize,
    callback: ActionCallback,
}

struct FilterRegistration {
    id: HookId,
    accepted_args: usize,
    callback: FilterCallback,
}

/// Minimal action/filter dispatcher for Rust migration scaffolding.
pub struct HookDispatcher {
    actions: BTreeMap<String, BTreeMap<i32, Vec<ActionRegistration>>>,
    filters: BTreeMap<String, BTreeMap<i32, Vec<FilterRegistration>>>,
    current_stack: Vec<String>,
    dispatch_counts: BTreeMap<String, u64>,
    next_id: u64,
}

impl Default for HookDispatcher {
    fn default() -> Self {
        Self {
            actions: BTreeMap::new(),
            filters: BTreeMap::new(),
            current_stack: Vec::new(),
            dispatch_counts: BTreeMap::new(),
            next_id: 1,
        }
    }
}

impl HookDispatcher {
    fn dispatch_action_callbacks(&self, name: &str, args: &[Value]) {
        if let Some(by_priority) = self.actions.get(name) {
            for callbacks in by_priority.values() {
                for callback in callbacks {
                    let accepted = callback.accepted_args.min(args.len());
                    (callback.callback)(&args[..accepted]);
                }
            }
        }
    }

    fn dispatch_all_actions(&self, hook_name: &str, args: &[Value]) {
        if hook_name == "all" {
            return;
        }

        let mut all_args = Vec::with_capacity(args.len() + 1);
        all_args.push(Value::String(hook_name.to_string()));
        all_args.extend(args.iter().cloned());
        self.dispatch_action_callbacks("all", &all_args);
    }

    pub fn add_action(
        &mut self,
        name: impl Into<String>,
        priority: i32,
        callback: ActionCallback,
    ) -> HookId {
        self.add_action_with_accepted_args(name, priority, 1, callback)
    }

    pub fn add_action_with_accepted_args(
        &mut self,
        name: impl Into<String>,
        priority: i32,
        accepted_args: usize,
        callback: ActionCallback,
    ) -> HookId {
        let id = self.allocate_id();
        self.actions
            .entry(name.into())
            .or_default()
            .entry(priority)
            .or_default()
            .push(ActionRegistration {
                id,
                accepted_args,
                callback,
            });
        id
    }

    pub fn do_action(&mut self, name: &str, args: &[Value]) {
        self.current_stack.push(name.to_string());
        self.dispatch_all_actions(name, args);
        self.dispatch_action_callbacks(name, args);
        self.current_stack.pop();
        *self.dispatch_counts.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn add_filter(
        &mut self,
        name: impl Into<String>,
        priority: i32,
        callback: FilterCallback,
    ) -> HookId {
        self.add_filter_with_accepted_args(name, priority, 1, callback)
    }

    pub fn add_filter_with_accepted_args(
        &mut self,
        name: impl Into<String>,
        priority: i32,
        accepted_args: usize,
        callback: FilterCallback,
    ) -> HookId {
        let id = self.allocate_id();
        self.filters
            .entry(name.into())
            .or_default()
            .entry(priority)
            .or_default()
            .push(FilterRegistration {
                id,
                accepted_args,
                callback,
            });
        id
    }

    pub fn apply_filters(&mut self, name: &str, mut value: Value, args: &[Value]) -> Value {
        self.current_stack.push(name.to_string());
        let all_args = if name == "all" {
            None
        } else {
            let mut all_args = Vec::with_capacity(args.len() + 2);
            all_args.push(Value::String(name.to_string()));
            all_args.push(value.clone());
            all_args.extend(args.iter().cloned());
            Some(all_args)
        };
        if let Some(all_args) = all_args.as_ref() {
            self.dispatch_action_callbacks("all", all_args);
        }

        if let Some(by_priority) = self.filters.get(name) {
            for callbacks in by_priority.values() {
                for callback in callbacks {
                    let accepted = callback.accepted_args.saturating_sub(1).min(args.len());
                    value = (callback.callback)(value, &args[..accepted]);
                }
            }
        }

        self.current_stack.pop();
        *self.dispatch_counts.entry(name.to_string()).or_insert(0) += 1;
        value
    }

    pub fn remove_action(&mut self, name: &str, id: HookId) -> bool {
        remove_registration(&mut self.actions, name, id)
    }

    pub fn remove_filter(&mut self, name: &str, id: HookId) -> bool {
        remove_registration(&mut self.filters, name, id)
    }

    pub fn remove_all_actions(&mut self, name: &str) -> usize {
        remove_all_registrations(&mut self.actions, name)
    }

    pub fn remove_all_filters(&mut self, name: &str) -> usize {
        remove_all_registrations(&mut self.filters, name)
    }

    pub fn has_action(&self, name: &str) -> bool {
        self.actions
            .get(name)
            .is_some_and(|by_priority| by_priority.values().any(|callbacks| !callbacks.is_empty()))
    }

    pub fn has_filter(&self, name: &str) -> bool {
        self.filters
            .get(name)
            .is_some_and(|by_priority| by_priority.values().any(|callbacks| !callbacks.is_empty()))
    }

    pub fn current_hook(&self) -> Option<&str> {
        self.current_stack.last().map(String::as_str)
    }

    pub fn doing_hook(&self, name: &str) -> bool {
        self.current_stack.iter().any(|current| current == name)
    }

    pub fn did_hook(&self, name: &str) -> u64 {
        self.dispatch_counts.get(name).copied().unwrap_or(0)
    }

    fn allocate_id(&mut self) -> HookId {
        let id = HookId(self.next_id);
        self.next_id += 1;
        id
    }
}

trait HookRegistration {
    fn id(&self) -> HookId;
}

impl HookRegistration for ActionRegistration {
    fn id(&self) -> HookId {
        self.id
    }
}

impl HookRegistration for FilterRegistration {
    fn id(&self) -> HookId {
        self.id
    }
}

fn remove_registration<T: HookRegistration>(
    hooks: &mut BTreeMap<String, BTreeMap<i32, Vec<T>>>,
    name: &str,
    id: HookId,
) -> bool {
    let mut removed = false;
    if let Some(by_priority) = hooks.get_mut(name) {
        for callbacks in by_priority.values_mut() {
            let previous_len = callbacks.len();
            callbacks.retain(|registration| registration.id() != id);
            if callbacks.len() != previous_len {
                removed = true;
            }
        }
    }
    removed
}

fn remove_all_registrations<T>(
    hooks: &mut BTreeMap<String, BTreeMap<i32, Vec<T>>>,
    name: &str,
) -> usize {
    let Some(by_priority) = hooks.remove(name) else {
        return 0;
    };

    by_priority.values().map(Vec::len).sum()
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

    #[test]
    fn remove_filter_prevents_callback_execution() {
        let mut dispatcher = HookDispatcher::default();
        let hook_id = dispatcher.add_filter(
            "sample",
            10,
            Box::new(|value, _| {
                let current = value.as_str().unwrap_or_default();
                Value::String(format!("{current}-changed"))
            }),
        );
        assert!(dispatcher.remove_filter("sample", hook_id));

        let result = dispatcher.apply_filters("sample", Value::String("value".to_string()), &[]);
        assert_eq!(result, Value::String("value".to_string()));
    }

    #[test]
    fn action_callback_honors_accepted_args_limit() {
        let mut dispatcher = HookDispatcher::default();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let seen_ref = std::sync::Arc::clone(&seen);
        dispatcher.add_action_with_accepted_args(
            "sample",
            10,
            1,
            Box::new(move |args| {
                let values = args
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                *seen_ref.lock().expect("lock poisoned") = values;
            }),
        );

        dispatcher.do_action(
            "sample",
            &[
                Value::String("first".to_string()),
                Value::String("second".to_string()),
            ],
        );

        assert_eq!(
            *seen.lock().expect("lock poisoned"),
            vec!["first".to_string()]
        );
    }

    #[test]
    fn filter_callback_honors_accepted_args_limit() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_filter_with_accepted_args(
            "sample",
            10,
            2,
            Box::new(|value, args| {
                let current = value.as_str().unwrap_or_default();
                let joined = args
                    .iter()
                    .filter_map(|item| item.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                Value::String(format!("{current}|{joined}"))
            }),
        );

        let result = dispatcher.apply_filters(
            "sample",
            Value::String("base".to_string()),
            &[
                Value::String("first".to_string()),
                Value::String("second".to_string()),
            ],
        );

        assert_eq!(result, Value::String("base|first".to_string()));
    }

    #[test]
    fn filter_callback_default_accepted_args_passes_no_extra_args() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_filter_with_accepted_args(
            "sample",
            10,
            1,
            Box::new(|value, args| {
                let current = value.as_str().unwrap_or_default();
                Value::String(format!("{current}|{}", args.len()))
            }),
        );

        let result = dispatcher.apply_filters(
            "sample",
            Value::String("base".to_string()),
            &[
                Value::String("first".to_string()),
                Value::String("second".to_string()),
            ],
        );

        assert_eq!(result, Value::String("base|0".to_string()));
    }

    #[test]
    fn did_hook_counts_action_and_filter_dispatches() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_action("init", 10, Box::new(|_| {}));
        dispatcher.add_filter("the_title", 10, Box::new(|value, _| value));

        dispatcher.do_action("init", &[]);
        dispatcher.do_action("init", &[]);
        let _ = dispatcher.apply_filters("the_title", Value::String("hello".to_string()), &[]);

        assert_eq!(dispatcher.did_hook("init"), 2);
        assert_eq!(dispatcher.did_hook("the_title"), 1);
        assert_eq!(dispatcher.did_hook("missing"), 0);
    }

    #[test]
    fn action_default_accepted_args_passes_only_first_argument() {
        let mut dispatcher = HookDispatcher::default();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let seen_ref = std::sync::Arc::clone(&seen);

        dispatcher.add_action(
            "sample",
            10,
            Box::new(move |args| {
                let observed = args
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                *seen_ref.lock().expect("lock poisoned") = observed;
            }),
        );

        dispatcher.do_action(
            "sample",
            &[
                Value::String("first".to_string()),
                Value::String("second".to_string()),
            ],
        );

        assert_eq!(
            *seen.lock().expect("lock poisoned"),
            vec!["first".to_string()]
        );
    }

    #[test]
    fn filter_default_accepted_args_passes_no_extra_arguments() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_filter(
            "sample",
            10,
            Box::new(|value, args| {
                let current = value.as_str().unwrap_or_default();
                Value::String(format!("{current}|{}", args.len()))
            }),
        );

        let result = dispatcher.apply_filters(
            "sample",
            Value::String("base".to_string()),
            &[Value::String("extra".to_string())],
        );

        assert_eq!(result, Value::String("base|0".to_string()));
    }

    #[test]
    fn all_action_receives_hook_name_and_arguments() {
        let mut dispatcher = HookDispatcher::default();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let seen_ref = std::sync::Arc::clone(&seen);
        dispatcher.add_action_with_accepted_args(
            "all",
            10,
            99,
            Box::new(move |args| {
                let observed = args
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| value.to_string())
                    })
                    .collect::<Vec<_>>();
                *seen_ref.lock().expect("lock poisoned") = observed;
            }),
        );

        dispatcher.do_action(
            "init",
            &[
                Value::String("first".to_string()),
                Value::String("second".to_string()),
            ],
        );

        assert_eq!(
            *seen.lock().expect("lock poisoned"),
            vec![
                "init".to_string(),
                "first".to_string(),
                "second".to_string()
            ]
        );
    }

    #[test]
    fn all_action_receives_filter_name_and_values() {
        let mut dispatcher = HookDispatcher::default();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let seen_ref = std::sync::Arc::clone(&seen);
        dispatcher.add_action_with_accepted_args(
            "all",
            10,
            99,
            Box::new(move |args| {
                let observed = args
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| value.to_string())
                    })
                    .collect::<Vec<_>>();
                *seen_ref.lock().expect("lock poisoned") = observed;
            }),
        );

        let result = dispatcher.apply_filters(
            "the_title",
            Value::String("hello".to_string()),
            &[Value::String("extra".to_string())],
        );
        assert_eq!(result, Value::String("hello".to_string()));
        assert_eq!(
            *seen.lock().expect("lock poisoned"),
            vec![
                "the_title".to_string(),
                "hello".to_string(),
                "extra".to_string()
            ]
        );
    }

    #[test]
    fn remove_all_actions_clears_hook_callbacks() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_action("init", 10, Box::new(|_| {}));
        dispatcher.add_action("init", 20, Box::new(|_| {}));

        assert_eq!(dispatcher.remove_all_actions("init"), 2);
        assert!(!dispatcher.has_action("init"));
        assert_eq!(dispatcher.remove_all_actions("init"), 0);
    }

    #[test]
    fn remove_all_filters_clears_hook_callbacks() {
        let mut dispatcher = HookDispatcher::default();
        dispatcher.add_filter("the_title", 10, Box::new(|value, _| value));
        dispatcher.add_filter("the_title", 20, Box::new(|value, _| value));

        assert_eq!(dispatcher.remove_all_filters("the_title"), 2);
        assert!(!dispatcher.has_filter("the_title"));
        assert_eq!(dispatcher.remove_all_filters("the_title"), 0);
    }
}
