use std::sync::{Arc, Mutex};

use serde_json::Value;
use wp_rs_hooks::HookDispatcher;

#[test]
fn action_callbacks_execute_in_priority_order() {
    let mut dispatcher = HookDispatcher::default();
    let events = Arc::new(Mutex::new(Vec::<String>::new()));

    let late_events = Arc::clone(&events);
    dispatcher.add_action(
        "plugins_loaded",
        20,
        Box::new(move |_| late_events.lock().expect("lock").push("late".to_string())),
    );

    let early_events = Arc::clone(&events);
    dispatcher.add_action(
        "plugins_loaded",
        10,
        Box::new(move |_| early_events.lock().expect("lock").push("early".to_string())),
    );

    dispatcher.do_action("plugins_loaded", &[]);
    assert_eq!(
        events.lock().expect("lock").clone(),
        vec!["early".to_string(), "late".to_string()]
    );
}

#[test]
fn removing_action_prevents_future_execution() {
    let mut dispatcher = HookDispatcher::default();
    let count = Arc::new(Mutex::new(0_u32));
    let count_ref = Arc::clone(&count);
    let callback_id = dispatcher.add_action(
        "init",
        10,
        Box::new(move |_| {
            *count_ref.lock().expect("lock") += 1;
        }),
    );

    assert!(dispatcher.has_action("init"));
    assert!(dispatcher.remove_action("init", callback_id));

    dispatcher.do_action("init", &[]);
    assert_eq!(*count.lock().expect("lock"), 0);
}

#[test]
fn filter_chain_supports_value_transforms() {
    let mut dispatcher = HookDispatcher::default();
    dispatcher.add_filter(
        "the_title",
        10,
        Box::new(|value, _| {
            let value = value.as_str().unwrap_or_default();
            Value::String(format!("[prefix] {value}"))
        }),
    );
    dispatcher.add_filter(
        "the_title",
        20,
        Box::new(|value, _| {
            let value = value.as_str().unwrap_or_default();
            Value::String(format!("{value} [suffix]"))
        }),
    );

    let filtered = dispatcher.apply_filters("the_title", Value::String("Hello".to_string()), &[]);
    assert_eq!(
        filtered,
        Value::String("[prefix] Hello [suffix]".to_string())
    );
}
