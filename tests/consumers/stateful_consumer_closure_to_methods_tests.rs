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
mod test_closure_to_methods {
    use super::Arc;
    use super::BoxStatefulConsumer;
    use super::Mutex;
    use super::StatefulConsumer;

    // Note: closures must implement Clone to use to_xxx methods
    // We need to use cloneable closures or wrapper types

    // ============================================================================
    // BoxConsumer ConsumerOnce Tests
    // ============================================================================

    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
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
    fn test_consumer_once_with_state_modification() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
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
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });

        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }
}

// ============================================================================
// ConsumerOnce Implementation Tests
// ============================================================================
