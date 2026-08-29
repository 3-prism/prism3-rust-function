// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulConsumer types

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::ArcConsumer;
use qubit_function::ArcStatefulConsumer;
use qubit_function::BoxConsumer;
use qubit_function::BoxStatefulConsumer;
use qubit_function::Consumer;
use qubit_function::RcConsumer;
use qubit_function::RcStatefulConsumer;
use qubit_function::StatefulConsumer;

// ============================================================================
// BoxConsumer Tests
// ============================================================================

#[cfg(test)]
mod test_arc_consumer {
    use std::panic::AssertUnwindSafe;
    use std::panic::catch_unwind;

    use super::Arc;
    use super::ArcConsumer;
    use super::ArcStatefulConsumer;
    use super::Consumer;
    use super::Mutex;
    use super::Rc;
    use super::RefCell;
    use super::StatefulConsumer;

    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut clone = consumer.clone();

        consumer.accept(&5);
        clone.accept(&10);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10]);
    }

    /// Verifies that callback mutations survive a panic and that the shared
    /// consumer remains usable after unwinding.
    #[test]
    fn test_accept_after_callback_panic() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let callback_log = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |value: &i32| {
            callback_log.lock().expect("mutex should not be poisoned").push(*value);
            assert_ne!(*value, 1, "first callback should panic");
        });

        let result = catch_unwind(AssertUnwindSafe(|| consumer.accept(&1)));
        assert!(result.is_err(), "the first callback should panic");
        assert_eq!(
            *log.lock().expect("mutex should not be poisoned"),
            vec![1],
            "mutations completed before the panic should be preserved"
        );

        consumer.accept(&2);
        assert_eq!(
            *log.lock().expect("mutex should not be poisoned"),
            vec![1, 2],
            "the consumer should remain usable after unwinding"
        );
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = ArcStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        });
        let second = ArcStatefulConsumer::new(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });
        let mut chained = first.and_then(second);

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]);
    }

    #[test]
    fn test_thread_safety() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });

        let mut c1 = consumer.clone();
        let mut c2 = consumer.clone();

        let h1 = std::thread::spawn(move || {
            c1.accept(&1);
        });

        let h2 = std::thread::spawn(move || {
            c2.accept(&2);
        });

        h1.join().expect("thread should not panic");
        h2.join().expect("thread should not panic");

        let mut result = log.lock().expect("mutex should not be poisoned").clone();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_noop() {
        let noop = ArcConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new_with_name("test_consumer", move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_noop_stateful() {
        let mut noop = ArcStatefulConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_debug() {
        let consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcStatefulConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcStatefulConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcStatefulConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcStatefulConsumer(my_consumer)");
    }

    // ============================================================================
    // ArcConsumer ConsumerOnce Tests
    // ============================================================================

    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_accept_once_with_different_types() {
        // String
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |s: &String| {
            *l.lock().expect("mutex should not be poisoned") = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept(&text);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "Got: hello");

        // Vec
        let log = Arc::new(Mutex::new(0));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |v: &Vec<i32>| {
            *l.lock().expect("mutex should not be poisoned") = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept(&numbers);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), 3);

        // bool
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |b: &bool| {
            *l.lock().expect("mutex should not be poisoned") = if *b { "true" } else { "false" }.to_string();
        });
        let flag = true;
        consumer.accept(&flag);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "true");
    }

    #[test]
    fn test_consumer_once_with_state_modification() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            counter += 1;
            l.lock().expect("mutex should not be poisoned").push(*x + counter);
        });
        consumer.accept(&10);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![11]); // 10 + 1
    }

    #[test]
    fn test_consumer_once_consumes_self() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });

        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    /// Test that ArcConsumer can work with non-Send + non-Sync types
    ///
    /// This test verifies that the relaxed generic constraints (T: 'static
    /// instead of T: Send + Sync + 'static) allow ArcConsumer to be created
    /// for types that are not thread-safe, as long as we only pass
    /// references to them.
    #[test]
    fn test_with_non_send_sync_type() {
        // Rc<RefCell<i32>> is neither Send nor Sync
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        // This should compile now with relaxed constraints
        let consumer = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = *value.borrow();
            l.lock().expect("mutex should not be poisoned").push(val);
        });

        let value = Rc::new(RefCell::new(42));
        consumer.accept(&value);

        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![42]);
    }

    /// Test that ArcConsumer with non-Send type can be cloned and used
    #[test]
    fn test_clone_with_non_send_sync_type() {
        type NonSendType = Rc<RefCell<String>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = value.borrow().clone();
            l.lock().expect("mutex should not be poisoned").push(val);
        });

        let consumer2 = consumer.clone();

        let value1 = Rc::new(RefCell::new("hello".to_string()));
        let value2 = Rc::new(RefCell::new("world".to_string()));

        consumer.accept(&value1);
        consumer2.accept(&value2);

        let result = log.lock().expect("mutex should not be poisoned").clone();
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    /// Test that ArcConsumer with non-Send type can be chained
    #[test]
    fn test_and_then_with_non_send_sync_type() {
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let first = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = *value.borrow();
            l1.lock().expect("mutex should not be poisoned").push(val * 2);
        });

        let second = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = *value.borrow();
            l2.lock().expect("mutex should not be poisoned").push(val + 10);
        });

        let chained = first.and_then(second);

        let value = Rc::new(RefCell::new(5));
        chained.accept(&value);

        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]); // 5*2=10, 5+10=15
    }
}

// ============================================================================
// RcConsumer Tests
// ============================================================================
