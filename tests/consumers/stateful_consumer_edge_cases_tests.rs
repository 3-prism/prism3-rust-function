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
mod test_edge_cases {
    use super::Arc;
    use super::ArcStatefulConsumer;
    use super::BoxConsumer;
    use super::BoxStatefulConsumer;
    use super::Consumer;
    use super::Mutex;
    use super::Rc;
    use super::RcStatefulConsumer;
    use super::RefCell;
    use super::StatefulConsumer;

    #[test]
    fn test_noop_with_name() {
        let mut consumer = BoxConsumer::<i32>::noop();
        consumer.set_name("noop_consumer");
        assert_eq!(consumer.name(), Some("noop_consumer"));
        consumer.accept(&5); // Should do nothing
    }

    // print and print_with methods have been removed

    #[test]
    fn test_if_then_with_always_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| true);
        conditional.accept(&5);
        conditional.accept(&10);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10]);
    }

    #[test]
    fn test_if_then_with_always_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| false);
        conditional.accept(&5);
        conditional.accept(&10);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), Vec::<i32>::new());
    }

    #[test]
    fn test_if_then_else_all_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| true).or_else(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 100);
        });
        conditional.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_if_then_else_all_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| false).or_else(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 100);
        });
        conditional.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![500]);
    }

    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        })
        .and_then(BoxStatefulConsumer::noop());
        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_complex_chain() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let l4 = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        })
        .and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        })
        .and_then(BoxStatefulConsumer::noop())
        .and_then(move |x: &i32| {
            l3.lock().expect("mutex should not be poisoned").push(*x + 10);
        })
        .and_then(move |x: &i32| {
            l4.lock().expect("mutex should not be poisoned").push(*x - 5);
        });
        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10, 15, 0]);
    }

    #[test]
    fn test_box_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut chained = conditional.and_then(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10]);
        chained.accept(&-5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10, -10]);
    }

    #[test]
    fn test_arc_when() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
        conditional.accept(&-5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
    }

    #[test]
    fn test_arc_conditional_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
        clone2.accept(&10);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, 10]);
    }

    #[test]
    fn test_arc_conditional_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l1.lock().expect("mutex should not be poisoned").push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32| {
            l2.lock().expect("mutex should not be poisoned").push(*x * 2);
        });
        with_else.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5]);
        with_else.accept(&-5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![5, -10]);
    }

    #[test]
    fn test_arc_conditional_debug() {
        let consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        let conditional = consumer.when(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("ArcConditionalStatefulConsumer"));
        assert!(debug_str.contains("consumer"));
        assert!(debug_str.contains("predicate"));
    }

    #[test]
    fn test_arc_conditional_display() {
        let consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        let conditional = consumer.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("ArcConditionalStatefulConsumer"));
    }

    #[test]
    fn test_rc_when() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
        conditional.accept(&-5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_rc_conditional_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
        clone2.accept(&10);
        assert_eq!(*log.borrow(), vec![5, 10]);
    }

    #[test]
    fn test_rc_conditional_or_else() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l1.borrow_mut().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32| {
            l2.borrow_mut().push(*x * 2);
        });
        with_else.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
        with_else.accept(&-5);
        assert_eq!(*log.borrow(), vec![5, -10]);
    }

    #[test]
    fn test_rc_conditional_debug() {
        let consumer = RcStatefulConsumer::new(|_x: &i32| {});
        let conditional = consumer.when(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("RcConditionalStatefulConsumer"));
        assert!(debug_str.contains("consumer"));
        assert!(debug_str.contains("predicate"));
    }

    #[test]
    fn test_rc_conditional_display() {
        let consumer = RcStatefulConsumer::new(|_x: &i32| {});
        let conditional = consumer.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("RcConditionalStatefulConsumer"));
    }
}
// ============================================================================
// Closure StatefulConsumer Tests
// ============================================================================
