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
mod test_box_consumer {
    use super::Arc;
    use super::BoxConsumer;
    use super::BoxStatefulConsumer;
    use super::Consumer;
    use super::Mutex;
    use super::StatefulConsumer;

    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |s: &String| {
            *l.lock().expect("mutex should not be poisoned") = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept(&text);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "Got: hello");

        // Vec
        let log = Arc::new(Mutex::new(0));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |v: &Vec<i32>| {
            *l.lock().expect("mutex should not be poisoned") = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept(&numbers);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), 3);

        // bool
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |b: &bool| {
            *l.lock().expect("mutex should not be poisoned") = if *b { "true" } else { "false" }.to_string();
        });
        let flag = true;
        consumer.accept(&flag);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), "true");
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });

        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]); // 5*2=10, 5+10=15
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + 1);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l3.lock().expect("mutex should not be poisoned").push(*x - 5);
        });

        let value = 10;
        consumer.accept(&value);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![11, 20, 5]); // 10+1=11, 10*2=20, 10-5=5
    }

    #[test]
    fn test_and_then_with_box_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let c1 = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        });
        let c2 = BoxStatefulConsumer::new(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });
        let mut combined = c1.and_then(c2);

        let value = 5;
        combined.accept(&value);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxConsumer::<i32>::noop();
        let value = 42;
        noop.accept(&value);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new_with_name("test_consumer", move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    // print and print_with methods have been removed

    #[test]
    fn test_if_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0);

        let positive = 5;
        conditional.accept(&positive);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);

        let negative = -5;
        conditional.accept(&negative);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]); // Unchanged
    }

    #[test]
    fn test_if_then_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0).or_else(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(-*x);
        });

        let positive = 5;
        conditional.accept(&positive);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);

        let negative = -5;
        conditional.accept(&negative);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 5]); // -(-5) = 5
    }

    #[test]
    fn test_debug() {
        let consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxStatefulConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxStatefulConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxStatefulConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxStatefulConsumer(my_consumer)");
    }
}

// ============================================================================
// ArcConsumer Tests
// ============================================================================
