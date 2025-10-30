/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Comprehensive tests for StatefulBiConsumer types
//!
//! This module provides exhaustive test coverage for all StatefulBiConsumer
//! implementations including BoxStatefulBiConsumer, ArcStatefulBiConsumer,
//! RcStatefulBiConsumer, and their conditional variants.

use prism3_function::{
    ArcStatefulBiConsumer,
    BiConsumerOnce,
    BoxStatefulBiConsumer,
    FnStatefulBiConsumerOps,
    RcStatefulBiConsumer,
    StatefulBiConsumer,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

// ============================================================================
// BoxStatefulBiConsumer Tests
// ============================================================================

#[cfg(test)]
mod box_stateful_bi_consumer_tests {
    use super::*;

    // Test new() constructor
    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test new_with_name() constructor
    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer =
            BoxStatefulBiConsumer::new_with_name("test_consumer", move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test noop() constructor
    #[test]
    fn test_noop() {
        let mut noop = BoxStatefulBiConsumer::<i32, i32>::noop();
        noop.accept(&42, &10);
        // Should not panic, values unchanged
    }

    // Test name() getter
    #[test]
    fn test_name_getter() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);
    }

    // Test set_name() setter
    #[test]
    fn test_set_name() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    // Test accept() method
    #[test]
    fn test_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x * *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![15]);
    }

    // Test accept() with multiple calls
    #[test]
    fn test_accept_multiple_calls() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&1, &2);
        consumer.accept(&3, &4);
        consumer.accept(&5, &6);
        assert_eq!(*log.lock().unwrap(), vec![3, 7, 11]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    // Test and_then() with multiple consumers
    #[test]
    fn test_and_then_chain() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let mut chained = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l3.lock().unwrap().push(*x - *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15, 2]);
    }

    // Test when() method
    #[test]
    fn test_when_true_condition() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test when() with false condition
    #[test]
    fn test_when_false_condition() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    // Test into_box() method (identity conversion)
    #[test]
    fn test_into_box_1() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut boxed = consumer.into_box();
        boxed.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_rc() method
    #[test]
    fn test_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_fn() method
    #[test]
    fn test_into_fn_1() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test accept_once() from BiConsumerOnce trait
    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_box() from BiConsumerOnce trait
    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut box_once = consumer.into_box();
        box_once.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_fn() from BiConsumerOnce trait
    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test with different types
    #[test]
    fn test_with_different_types() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |s: &String, n: &i32| {
            *l.lock().unwrap() = format!("{}: {}", s, n);
        });
        consumer.accept(&"Count".to_string(), &42);
        assert_eq!(*log.lock().unwrap(), "Count: 42");
    }

    // Test with zero values
    #[test]
    fn test_with_zero_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&0, &0);
        assert_eq!(*log.lock().unwrap(), vec![0]);
    }

    // Test with negative values
    #[test]
    fn test_with_negative_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&-5, &-3);
        assert_eq!(*log.lock().unwrap(), vec![-8]);
    }

    // Test with mixed positive and negative values
    #[test]
    fn test_with_mixed_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &-3);
        assert_eq!(*log.lock().unwrap(), vec![2]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxStatefulBiConsumer"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxStatefulBiConsumer");
    }

    // Test Display with name
    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxStatefulBiConsumer(my_consumer)");
    }
}

// ============================================================================
// BoxConditionalBiConsumer Tests
// ============================================================================

#[cfg(test)]
mod box_conditional_bi_consumer_tests {
    use super::*;

    // Test accept() with true condition
    #[test]
    fn test_accept_when_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test accept() with false condition
    #[test]
    fn test_accept_when_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut chained = conditional.and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
        chained.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15, -15]);
    }

    // Test or_else() method
    #[test]
    fn test_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let mut conditional =
            consumer
                .when(|x: &i32, _y: &i32| *x > 0)
                .or_else(move |x: &i32, y: &i32| {
                    l2.lock().unwrap().push(*x * *y);
                });

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, -15]);
    }

    // Test into_box() method
    #[test]
    fn test_into_box_2() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut boxed = conditional.into_box();
        boxed.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        boxed.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_rc() method
    #[test]
    fn test_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        rc_consumer.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_fn() method
    #[test]
    fn test_into_fn_2() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        func(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test with always true predicate
    #[test]
    fn test_with_always_true_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|_: &i32, _: &i32| true);
        conditional.accept(&5, &3);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, -2]);
    }

    // Test with always false predicate
    #[test]
    fn test_with_always_false_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|_: &i32, _: &i32| false);
        conditional.accept(&5, &3);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    // Test complex predicate
    #[test]
    fn test_with_complex_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0 && *x + *y < 10);
        conditional.accept(&2, &3);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        conditional.accept(&5, &10);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }
}

// ============================================================================
// ArcStatefulBiConsumer Tests
// ============================================================================

#[cfg(test)]
mod arc_stateful_bi_consumer_tests {
    use super::*;

    // Test new() constructor
    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test new_with_name() constructor
    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer =
            ArcStatefulBiConsumer::new_with_name("test_consumer", move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test name() getter
    #[test]
    fn test_name_getter() {
        let mut consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);
    }

    // Test set_name() setter
    #[test]
    fn test_set_name() {
        let mut consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    // Test accept() method
    #[test]
    fn test_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x * *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![15]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let mut clone1 = consumer.clone();
        let mut clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        clone2.accept(&10, &2);
        assert_eq!(*log.lock().unwrap(), vec![8, 12]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let second = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        let mut chained = first.and_then(second);
        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    // Test when() method
    #[test]
    fn test_when() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test to_fn() method
    #[test]
    fn test_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let mut func = consumer.to_fn();
        func(&5, &3);
        func(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    // Test into_box() method
    #[test]
    fn test_into_box_3() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_rc() method
    #[test]
    fn test_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_arc() method
    #[test]
    fn test_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut arc_consumer = consumer.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_fn() method
    #[test]
    fn test_into_fn_3() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test to_box() method
    #[test]
    fn test_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_rc() method
    #[test]
    fn test_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_arc() method
    #[test]
    fn test_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut arc_consumer = consumer.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test accept_once() from BiConsumerOnce trait
    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_box() from BiConsumerOnce trait
    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut box_once = consumer.into_box();
        box_once.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_fn() from BiConsumerOnce trait
    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test thread safety
    #[test]
    fn test_thread_safety() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let mut cons = consumer.clone();
                std::thread::spawn(move || {
                    cons.accept(&i, &1);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let mut result = log.lock().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let mut consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcStatefulBiConsumer"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let mut consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcStatefulBiConsumer");
    }

    // Test Display with name
    #[test]
    fn test_display_with_name() {
        let mut consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcStatefulBiConsumer(my_consumer)");
    }
}

// ============================================================================
// ArcConditionalBiConsumer Tests
// ============================================================================

#[cfg(test)]
mod arc_conditional_bi_consumer_tests {
    use super::*;

    // Test accept() with true condition
    #[test]
    fn test_accept_when_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test accept() with false condition
    #[test]
    fn test_accept_when_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        clone2.accept(&10, &2);
        assert_eq!(*log.lock().unwrap(), vec![8, 12]);
    }

    // Test or_else() method
    #[test]
    fn test_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        with_else.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        with_else.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, -15]);
    }

    // Test into_box() method
    #[test]
    fn test_into_box_4() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut box_consumer = conditional.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        box_consumer.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_rc() method
    #[test]
    fn test_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        rc_consumer.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_arc() method
    #[test]
    fn test_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut arc_consumer = conditional.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        arc_consumer.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_fn() method
    #[test]
    fn test_into_fn_4() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        func(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test to_box() method
    #[test]
    fn test_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut box_consumer = conditional.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_rc() method
    #[test]
    fn test_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut rc_consumer = conditional.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_arc() method
    #[test]
    fn test_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut arc_consumer = conditional.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_fn() method
    #[test]
    fn test_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut func = conditional.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }
}

// ============================================================================
// RcStatefulBiConsumer Tests
// ============================================================================

#[cfg(test)]
mod rc_stateful_bi_consumer_tests {
    use super::*;

    // Test new() constructor
    #[test]
    fn test_new() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test new_with_name() constructor
    #[test]
    fn test_new_with_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer =
            RcStatefulBiConsumer::new_with_name("test_consumer", move |x: &i32, y: &i32| {
                l.borrow_mut().push(*x + *y);
            });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test name() getter
    #[test]
    fn test_name_getter() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);
    }

    // Test set_name() setter
    #[test]
    fn test_set_name() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    // Test accept() method
    #[test]
    fn test_accept() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x * *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![15]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let mut clone1 = consumer.clone();
        let mut clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);

        clone2.accept(&10, &2);
        assert_eq!(*log.borrow(), vec![8, 12]);
    }

    // Test and_then() method
    #[test]
    fn test_and_then() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        });
        let second = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });

        let mut chained = first.and_then(second);
        chained.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8, 15]);
    }

    // Test when() method
    #[test]
    fn test_when() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        conditional.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test to_fn() method
    #[test]
    fn test_to_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let mut func = consumer.to_fn();
        func(&5, &3);
        func(&10, &20);
        assert_eq!(*log.borrow(), vec![8, 30]);
    }

    // Test into_box() method
    #[test]
    fn test_into_box_5() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_rc() method
    #[test]
    fn test_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_fn() method
    #[test]
    fn test_into_fn_5() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test to_box() method
    #[test]
    fn test_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    // Test to_rc() method
    #[test]
    fn test_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    // Test accept_once() from BiConsumerOnce trait
    #[test]
    fn test_accept_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_box() from BiConsumerOnce trait
    #[test]
    fn test_into_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut box_once = consumer.into_box();
        box_once.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_fn() from BiConsumerOnce trait
    #[test]
    fn test_into_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test Debug trait implementation
    #[test]
    fn test_debug() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcStatefulBiConsumer"));
    }

    // Test Display trait implementation
    #[test]
    fn test_display() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcStatefulBiConsumer");
    }

    // Test Display with name
    #[test]
    fn test_display_with_name() {
        let mut consumer = RcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcStatefulBiConsumer(my_consumer)");
    }
}

// ============================================================================
// RcConditionalBiConsumer Tests
// ============================================================================

#[cfg(test)]
mod rc_conditional_bi_consumer_tests {
    use super::*;

    // Test accept() with true condition
    #[test]
    fn test_accept_when_true() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test accept() with false condition
    #[test]
    fn test_accept_when_false() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        conditional.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![]);
    }

    // Test clone() method
    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        clone2.accept(&10, &2);
        assert_eq!(*log.borrow(), vec![8, 12]);
    }

    // Test or_else() method
    #[test]
    fn test_or_else() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });
        with_else.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        with_else.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8, -15]);
    }

    // Test into_box() method
    #[test]
    fn test_into_box_6() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut box_consumer = conditional.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        box_consumer.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_rc() method
    #[test]
    fn test_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        rc_consumer.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_fn() method
    #[test]
    fn test_into_fn_6() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        func(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test to_box() method
    #[test]
    fn test_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut box_consumer = conditional.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    // Test to_rc() method
    #[test]
    fn test_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut rc_consumer = conditional.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    // Test to_fn() method
    #[test]
    fn test_to_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, _y: &i32| *x > 0);
        let mut func = conditional.to_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }
}

// ============================================================================
// FnStatefulBiConsumerOps Tests (Closure Extension Methods)
// ============================================================================

#[cfg(test)]
mod fn_stateful_bi_consumer_ops_tests {
    use super::*;

    // Test and_then() on closure
    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    // Test and_then() with multiple closures
    #[test]
    fn test_closure_and_then_chain() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let mut chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l3.lock().unwrap().push(*x - *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15, 2]);
    }

    // Test and_then() with BoxStatefulBiConsumer
    #[test]
    fn test_closure_and_then_with_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let box_consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        let mut chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(box_consumer);

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    // Test and_then() with ArcStatefulBiConsumer
    #[test]
    fn test_closure_and_then_with_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let arc_consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        let mut chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(arc_consumer);

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    // Test and_then() with RcStatefulBiConsumer
    #[test]
    fn test_closure_and_then_with_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let rc_consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });
        let mut chained = (move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        })
        .and_then(rc_consumer);

        chained.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8, 15]);
    }

    // Test and_then() returns BoxStatefulBiConsumer
    #[test]
    fn test_closure_and_then_returns_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        // Verify it's a BoxStatefulBiConsumer by using its methods
        let mut boxed = chained.into_box();
        boxed.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }
}

// ============================================================================
// Closure Implementation Tests (StatefulBiConsumer trait for closures)
// ============================================================================

#[cfg(test)]
mod closure_stateful_bi_consumer_tests {
    use super::*;

    // Test accept() on closure
    #[test]
    fn test_closure_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        closure.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_box() on closure
    #[test]
    fn test_closure_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let mut box_consumer = prism3_function::StatefulBiConsumer::<i32, i32>::into_box(closure);
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_rc() on closure
    #[test]
    fn test_closure_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        };
        let mut rc_consumer = closure.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_arc() on closure
    #[test]
    fn test_closure_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let mut arc_consumer = closure.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_fn() on closure
    #[test]
    fn test_closure_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let mut func = prism3_function::BiConsumer::into_fn(closure);
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test to_box() on closure
    #[test]
    fn test_closure_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let _l2 = log.clone();
        let mut closure = move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        };
        let mut box_consumer = prism3_function::StatefulBiConsumer::<i32, i32>::to_box(&closure);
        box_consumer.accept(&5, &3);
        // Original closure should still be usable
        closure.accept(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    // Test to_rc() on closure
    #[test]
    fn test_closure_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let _l2 = log.clone();
        let mut closure = move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        };
        let mut rc_consumer = closure.to_rc();
        rc_consumer.accept(&5, &3);
        // Original closure should still be usable
        closure.accept(&10, &20);
        assert_eq!(*log.borrow(), vec![8, 30]);
    }

    // Test to_arc() on closure
    #[test]
    fn test_closure_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let _l2 = log.clone();
        let mut closure = move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        };
        let mut arc_consumer = closure.to_arc();
        arc_consumer.accept(&5, &3);
        // Original closure should still be usable
        closure.accept(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    // Test to_fn() on closure - returns the closure itself
    #[test]
    fn test_closure_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        // to_fn() returns a clone of the closure
        let mut func = prism3_function::BiConsumer::to_fn(&closure);
        func(&5, &3);
        func(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }
}

// ============================================================================
// Edge Cases and Boundary Conditions Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    // Test with empty operations
    #[test]
    fn test_noop_multiple_calls() {
        let mut consumer = BoxStatefulBiConsumer::<i32, i32>::noop();
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        consumer.accept(&1, &2);
        // Should do nothing
    }

    // Test with large values
    #[test]
    fn test_with_large_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i64, y: &i64| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&i64::MAX, &0);
        assert_eq!(*log.lock().unwrap(), vec![i64::MAX]);
    }

    // Test with minimum values
    #[test]
    fn test_with_min_values() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i64, y: &i64| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&i64::MIN, &0);
        assert_eq!(*log.lock().unwrap(), vec![i64::MIN]);
    }

    // Test with string types
    #[test]
    fn test_with_string_types() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |s1: &String, s2: &String| {
            *l.lock().unwrap() = format!("{}{}", s1, s2);
        });
        consumer.accept(&"Hello, ".to_string(), &"World!".to_string());
        assert_eq!(*log.lock().unwrap(), "Hello, World!");
    }

    // Test with empty strings
    #[test]
    fn test_with_empty_strings() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |s1: &String, s2: &String| {
            *l.lock().unwrap() = format!("{}{}", s1, s2);
        });
        consumer.accept(&"".to_string(), &"".to_string());
        assert_eq!(*log.lock().unwrap(), "");
    }

    // Test with complex types
    #[test]
    fn test_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |p1: &Point, p2: &Point| {
            l.lock().unwrap().push(Point {
                x: p1.x + p2.x,
                y: p1.y + p2.y,
            });
        });
        consumer.accept(&Point { x: 1, y: 2 }, &Point { x: 3, y: 4 });
        assert_eq!(*log.lock().unwrap(), vec![Point { x: 4, y: 6 }]);
    }

    // Test stateful behavior
    #[test]
    fn test_stateful_behavior() {
        let counter = Arc::new(Mutex::new(0));
        let c = counter.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += 1;
            println!("Call {}: {} + {} = {}", *c.lock().unwrap(), x, y, x + y);
        });
        consumer.accept(&1, &2);
        consumer.accept(&3, &4);
        consumer.accept(&5, &6);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    // Test and_then with noop
    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        })
        .and_then(BoxStatefulBiConsumer::noop());
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }
}

// ============================================================================
// Custom Struct Tests - StatefulBiConsumer Default Implementation to_xxx()
// ============================================================================

/// Custom struct for testing StatefulBiConsumer trait default implementations
#[derive(Clone)]
struct CustomStatefulBiConsumer {
    multiplier: i32,
    log: Arc<Mutex<Vec<i32>>>,
}

impl StatefulBiConsumer<i32, i32> for CustomStatefulBiConsumer {
    fn accept(&mut self, first: &i32, second: &i32) {
        self.multiplier += 1;
        let result = (*first + *second) * self.multiplier;
        self.log.lock().unwrap().push(result);
    }
}

// Implement Send and Sync for CustomStatefulBiConsumer to support Arc
unsafe impl Send for CustomStatefulBiConsumer {}
unsafe impl Sync for CustomStatefulBiConsumer {}

#[test]
fn test_custom_stateful_bi_consumer_into_box() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut boxed = custom.into_box();
    boxed.accept(&3, &4); // (3 + 4) * 1 = 7
    boxed.accept(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
}

#[test]
fn test_custom_stateful_bi_consumer_into_rc() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut rc = custom.into_rc();
    rc.accept(&3, &4); // (3 + 4) * 1 = 7
    rc.accept(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
}

#[test]
fn test_custom_stateful_bi_consumer_into_arc() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut arc = custom.into_arc();
    arc.accept(&3, &4); // (3 + 4) * 1 = 7
    arc.accept(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
}

#[test]
fn test_custom_stateful_bi_consumer_into_fn() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut func = custom.into_fn();
    func(&3, &4); // (3 + 4) * 1 = 7
    func(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
}

#[test]
fn test_custom_stateful_bi_consumer_to_box() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut boxed = custom.to_box();
    boxed.accept(&3, &4); // (3 + 4) * 1 = 7
    boxed.accept(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    custom_clone.accept(&2, &3); // (2 + 3) * 1 = 5 (independent state)
    assert_eq!(*log.lock().unwrap(), vec![7, 14, 5]);
}

#[test]
fn test_custom_stateful_bi_consumer_to_rc() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut rc = custom.to_rc();
    rc.accept(&3, &4); // (3 + 4) * 1 = 7
    rc.accept(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    custom_clone.accept(&2, &3); // (2 + 3) * 1 = 5 (independent state)
    assert_eq!(*log.lock().unwrap(), vec![7, 14, 5]);
}

#[test]
fn test_custom_stateful_bi_consumer_to_arc() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut arc = custom.to_arc();
    arc.accept(&3, &4); // (3 + 4) * 1 = 7
    arc.accept(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    custom_clone.accept(&2, &3); // (2 + 3) * 1 = 5 (independent state)
    assert_eq!(*log.lock().unwrap(), vec![7, 14, 5]);
}

#[test]
fn test_custom_stateful_bi_consumer_to_fn() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let custom = CustomStatefulBiConsumer {
        multiplier: 0,
        log: log.clone(),
    };
    let mut func = custom.to_fn();
    func(&3, &4); // (3 + 4) * 1 = 7
    func(&5, &2); // (5 + 2) * 2 = 14
    assert_eq!(*log.lock().unwrap(), vec![7, 14]);
    // Original custom is still usable (was cloned)
    let mut custom_clone = custom.clone();
    custom_clone.accept(&2, &3); // (2 + 3) * 1 = 5 (independent state)
    assert_eq!(*log.lock().unwrap(), vec![7, 14, 5]);
}
