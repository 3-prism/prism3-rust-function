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
mod consumer_once_trait_tests {
    use super::Arc;
    use super::ArcConsumer;
    use super::BoxConsumer;
    use super::Consumer;
    use super::Mutex;
    use super::Rc;
    use super::RcConsumer;
    use super::RefCell;

    #[test]
    fn test_box_consumer_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });

        consumer.accept(&100);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![100]);
    }

    #[test]
    fn test_arc_consumer_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x * 3);
        });

        consumer.accept(&7);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![21]);
    }

    #[test]
    fn test_rc_consumer_accept_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x - 5);
        });

        consumer.accept(&15);
        assert_eq!(*log.borrow(), vec![10]);
    }
}

// ============================================================================
// BoxStatefulConsumer and_then Tests
// ============================================================================
