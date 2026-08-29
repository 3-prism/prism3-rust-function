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
mod test_box_stateful_consumer_chaining {
    use super::Arc;
    use super::ArcStatefulConsumer;
    use super::BoxStatefulConsumer;
    use super::Mutex;
    use super::StatefulConsumer;

    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let mut chained = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });

        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]);
    }

    #[test]
    fn test_closure_and_then_with_box_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let second = BoxStatefulConsumer::new(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });

        let mut chained = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(second);

        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]);
    }

    #[test]
    fn test_closure_and_then_multiple() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();

        let first = move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        };
        let second = move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        };
        let third = BoxStatefulConsumer::new(move |x: &i32| {
            l3.lock().expect("mutex should not be poisoned").push(*x + 100);
        });

        let chained = BoxStatefulConsumer::new(first).and_then(second);
        let mut chained = chained.and_then(third);

        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10, 105]);
    }

    #[test]
    fn test_closure_and_then_with_arc_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let second = ArcStatefulConsumer::new(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 3);
        });

        let mut chained = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x + 1);
        })
        .and_then(second);

        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![6, 15]);
    }

    #[test]
    fn test_closure_and_then_with_arc_consumer_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let second = ArcStatefulConsumer::new(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x + 10);
        });

        // Clone second to preserve it
        let mut chained = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(second.clone());

        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15]);

        // Original second still usable
        let mut second_copy = second;
        second_copy.accept(&3);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![10, 15, 13]);
    }
}
