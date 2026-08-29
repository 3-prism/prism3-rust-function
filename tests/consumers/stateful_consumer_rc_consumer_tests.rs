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
mod test_rc_consumer {
    use std::panic::AssertUnwindSafe;
    use std::panic::catch_unwind;

    use super::Consumer;
    use super::Rc;
    use super::RcConsumer;
    use super::RcStatefulConsumer;
    use super::RefCell;
    use super::StatefulConsumer;

    #[test]
    fn test_new() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut clone = consumer.clone();

        consumer.accept(&5);
        clone.accept(&10);
        assert_eq!(*log.borrow(), vec![5, 10]);
    }

    /// Verifies that synchronous re-entry panics after preserving prior
    /// mutations, and that the shared consumer remains usable after unwinding.
    #[test]
    fn test_accept_reentrant_call_panics_and_recovers() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let callback_log = log.clone();
        let shared_consumer = Rc::new(RefCell::new(None::<RcStatefulConsumer<i32>>));
        let callback_consumer = shared_consumer.clone();
        let mut consumer = RcStatefulConsumer::new(move |value: &i32| {
            callback_log.borrow_mut().push(*value);
            if *value == 1 {
                let mut reentrant = callback_consumer
                    .borrow()
                    .as_ref()
                    .expect("shared consumer should be initialized")
                    .clone();
                reentrant.accept(&2);
            }
        });
        *shared_consumer.borrow_mut() = Some(consumer.clone());

        let result = catch_unwind(AssertUnwindSafe(|| consumer.accept(&1)));
        assert!(result.is_err(), "synchronous re-entry should panic");
        assert_eq!(
            *log.borrow(),
            vec![1],
            "mutations completed before the panic should be preserved"
        );

        consumer.accept(&3);
        assert_eq!(
            *log.borrow(),
            vec![1, 3],
            "the consumer should remain usable after unwinding"
        );

        assert!(
            shared_consumer.borrow_mut().take().is_some(),
            "shared consumer should be present during cleanup"
        );
    }

    #[test]
    fn test_and_then() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = RcStatefulConsumer::new(move |x: &i32| {
            l1.borrow_mut().push(*x * 2);
        });
        let second = RcStatefulConsumer::new(move |x: &i32| {
            l2.borrow_mut().push(*x + 10);
        });
        let mut chained = first.and_then(second);

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.borrow(), vec![10, 15]);
    }

    #[test]
    fn test_noop() {
        let noop = RcConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_new_with_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new_with_name("test_consumer", move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_noop_stateful() {
        let mut noop = RcStatefulConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_debug() {
        let consumer = RcStatefulConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcStatefulConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = RcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcStatefulConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = RcStatefulConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcStatefulConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = RcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcStatefulConsumer(my_consumer)");
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================
