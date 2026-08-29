// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Tests for BiConsumer types
use qubit_function::ArcBiConsumer;
/// Tests for BiConsumer types
use qubit_function::BiConsumer;
/// Tests for BiConsumer types
use qubit_function::BoxBiConsumer;
/// Tests for BiConsumer types
use qubit_function::RcBiConsumer;

#[cfg(test)]
mod box_conditional_bi_consumer_tests {
    use std::sync::Mutex;

    use super::Arc;
    use super::BiConsumer;
    use super::BoxBiConsumer;

    #[test]
    fn test_box_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        });

        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let chained = conditional.and_then(move |x: &i32, y: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15]);

        chained.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, 15, -15]);
    }

    #[test]
    fn test_box_conditional_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + *y);
        });

        let conditional = consumer
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(move |x: &i32, y: &i32| {
                l2.lock().expect("mutex should not be poisoned").push(*x * *y);
            });

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, -15]);
    }

    #[test]
    fn test_box_conditional_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x + *y);
        });

        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8]);
    }
}
