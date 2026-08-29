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
mod debug_display_tests {
    use super::Arc;
    use super::BoxConsumerOnce;
    use super::ConsumerOnce;
    use super::Mutex;

    #[test]
    fn test_debug() {
        let consumer = BoxConsumerOnce::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxConsumerOnce"));
    }

    #[test]
    fn test_display() {
        let consumer = BoxConsumerOnce::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxConsumerOnce");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxConsumerOnce::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxConsumerOnce(my_consumer)");
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxConsumerOnce::new(|_x: &i32| {});
        assert_eq!(consumer.name(), None);
        consumer.set_name("test");
        assert_eq!(consumer.name(), Some("test"));
    }

    #[test]
    fn test_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let chained = conditional.and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10]);
    }
}

// ============================================================================
// Custom ConsumerOnce Tests - Testing Default into_xxx() Implementation
// ============================================================================
