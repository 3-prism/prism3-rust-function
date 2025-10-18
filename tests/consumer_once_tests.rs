/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # ConsumerOnce Tests
//!
//! Unit tests for the ConsumerOnce trait and its implementations.

use prism3_function::{BoxConsumerOnce, ConsumerOnce, FnConsumerOnceOps};
use std::sync::{Arc, Mutex};

// ============================================================================
// BoxConsumerOnce Tests
// ============================================================================

#[cfg(test)]
mod box_consumer_once_tests {
    use super::*;

    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_and_then_multiple() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let chained = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        })
        .and_then(move |x: &i32| {
            l3.lock().unwrap().push(*x - 1);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15, 4]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxConsumerOnce::<i32>::noop();
        noop.accept(&42);
        // Should not panic
    }

    // print and print_with methods have been removed

    #[test]
    fn test_if_then_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x + 1);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![6]);
    }

    #[test]
    fn test_if_then_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x + 1);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&-5);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_if_then_else_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x + 1);
        });
        let conditional = consumer.when(|x: &i32| *x > 0).or_else(move |x: &i32| {
            l2.lock().unwrap().push(*x - 1);
        });
        conditional.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![6]);
    }

    #[test]
    fn test_if_then_else_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x + 1);
        });
        let conditional = consumer.when(|x: &i32| *x > 0).or_else(move |x: &i32| {
            l2.lock().unwrap().push(*x - 1);
        });
        conditional.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![-6]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let boxed = consumer.into_box();
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let func = consumer.into_fn();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_into_fn_consumes_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let func = consumer.into_fn();
        // FnOnce can only be called once, so we call it once
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
        // Note: Cannot call func again because it's FnOnce
    }
}

// ============================================================================
// Closure Tests
// ============================================================================

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        closure.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = (move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_closure_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        let boxed = closure.into_box();
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_closure_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        let func = closure.into_fn();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_closure_multi_step_chaining() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let chained = (move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        })
        .and_then(move |x: &i32| {
            l3.lock().unwrap().push(*x / 2);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15, 2]);
    }
}

#[cfg(test)]
mod debug_display_tests {
    use super::*;

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
    fn test_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let boxed = conditional.into_box();
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_conditional_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let func = conditional.into_fn();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let chained = conditional.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }
}
