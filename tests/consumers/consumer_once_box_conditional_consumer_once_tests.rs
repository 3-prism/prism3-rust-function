// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::sync::Arc;
use std::sync::Mutex;

///
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
use qubit_function::BoxConsumerOnce;
///
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
use qubit_function::ConsumerOnce;

// ============================================================================
// BoxConsumerOnce Tests
// ============================================================================

#[cfg(test)]
mod box_conditional_consumer_once_tests {
    use super::Arc;
    use super::BoxConsumerOnce;
    use super::ConsumerOnce;
    use super::Mutex;

    // Tests for accept() method

    #[test]
    fn test_accept_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&-5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    #[test]
    fn test_accept_predicate_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_accept_predicate_boundary() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        // Test boundary case - predicate checks > 0, so 0 should be false
        conditional.accept(&0);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    // Tests for into_box() method

    // Tests for into_fn() method

    // Additional tests for into_box() and into_fn() with complex predicates

    // Additional comprehensive branch coverage tests for accept() method

    #[test]
    fn test_accept_with_always_true_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|_: &i32| true);
        conditional.accept(&42);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![42]);
    }

    #[test]
    fn test_accept_with_always_false_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|_: &i32| false);
        conditional.accept(&42);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    #[test]
    fn test_accept_with_complex_predicate_logic() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x * 10);
        });
        // Complex predicate: value is positive and even
        let conditional = consumer.when(|x: &i32| *x > 0 && *x % 2 == 0);
        conditional.accept(&4);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![40]);
    }

    #[test]
    fn test_accept_with_complex_predicate_logic_fails() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x * 10);
        });
        // Complex predicate: value is positive and even
        let conditional = consumer.when(|x: &i32| *x > 0 && *x % 2 == 0);
        // Test with odd number - fails the even check
        conditional.accept(&3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    #[test]
    fn test_accept_with_complex_predicate_logic_fails_negative() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x * 10);
        });
        // Complex predicate: value is positive and even
        let conditional = consumer.when(|x: &i32| *x > 0 && *x % 2 == 0);
        // Test with negative even number - fails the positive check
        conditional.accept(&-4);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    // Tests for and_then() method with conditional consumer

    #[test]
    fn test_and_then_predicate_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let conditional = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        })
        .when(|x: &i32| *x > 0);

        let chained = conditional.and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        });

        chained.accept(&5);
        // First consumer executes (5), second consumer executes (10)
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10]);
    }

    #[test]
    fn test_and_then_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let conditional = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        })
        .when(|x: &i32| *x > 0);

        let chained = conditional.and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        });

        chained.accept(&-5);
        // First consumer doesn't execute (predicate false), second consumer
        // still executes (-10)
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![-10]);
    }

    #[test]
    fn test_and_then_multiple_conditionals() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();

        let conditional1 = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        })
        .when(|x: &i32| *x > 0);

        let conditional2 = BoxConsumerOnce::new(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .when(|x: &i32| *x % 2 == 0);

        let chained = conditional1.and_then(conditional2).and_then(move |x: &i32| {
            l3.lock().expect("mutex should not be poisoned").push(*x + 100);
        });

        // Test with 6: positive (first passes), even (second passes), third
        // always executes
        chained.accept(&6);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![6, 12, 106]);
    }
}
// ============================================================================
// to_box() and to_fn() Tests - Closure Implementation
// ============================================================================
