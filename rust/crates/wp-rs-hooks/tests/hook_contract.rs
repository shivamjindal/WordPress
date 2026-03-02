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

#[test]
fn callbacks_can_limit_accepted_args() {
    let mut dispatcher = HookDispatcher::default();
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen_ref = Arc::clone(&seen);
    dispatcher.add_action_with_accepted_args(
        "init",
        10,
        1,
        Box::new(move |args| {
            let observed = args
                .iter()
                .filter_map(|value| value.as_str())
                .map(str::to_string)
                .collect::<Vec<_>>();
            *seen_ref.lock().expect("lock") = observed;
        }),
    );
    dispatcher.do_action(
        "init",
        &[
            Value::String("first".to_string()),
            Value::String("second".to_string()),
        ],
    );
    assert_eq!(*seen.lock().expect("lock"), vec!["first".to_string()]);

    dispatcher.add_filter_with_accepted_args(
        "sample_filter",
        10,
        1,
        Box::new(|value, args| {
            let value = value.as_str().unwrap_or_default();
            let first = args.first().and_then(Value::as_str).unwrap_or_default();
            Value::String(format!("{value}|{first}"))
        }),
    );
    let filtered = dispatcher.apply_filters(
        "sample_filter",
        Value::String("body".to_string()),
        &[
            Value::String("alpha".to_string()),
            Value::String("beta".to_string()),
        ],
    );
    assert_eq!(filtered, Value::String("body|alpha".to_string()));
}
