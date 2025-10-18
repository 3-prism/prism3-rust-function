/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Tests for BiConsumerOnce types

use prism3_function::{BiConsumerOnce, BoxBiConsumerOnce, FnBiConsumerOnceOps};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod box_bi_consumer_once_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxBiConsumerOnce::<i32, i32>::noop();
        noop.accept(&42, &10);
        // Should not panic
    }

    #[test]
    fn test_print() {
        let print = BoxBiConsumerOnce::<i32, i32>::print();
        print.accept(&42, &10); // Should print: (42, 10)
    }

    #[test]
    fn test_print_with() {
        let print = BoxBiConsumerOnce::<i32, i32>::print_with("Values: ");
        print.accept(&42, &10); // Should print: Values: 42, 10
    }

    #[test]
    fn test_if_then_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let conditional = BoxBiConsumerOnce::if_then(
            |x: &i32, y: &i32| *x > 0 && *y > 0,
            move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            },
        );

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_if_then_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let conditional = BoxBiConsumerOnce::if_then(
            |x: &i32, y: &i32| *x > 0 && *y > 0,
            move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            },
        );

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    #[test]
    fn test_if_then_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let conditional = BoxBiConsumerOnce::if_then_else(
            |x: &i32, y: &i32| *x > *y,
            move |x: &i32, _y: &i32| {
                l1.lock().unwrap().push(*x);
            },
            move |_x: &i32, y: &i32| {
                l2.lock().unwrap().push(*y);
            },
        );

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let box_consumer = closure.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxBiConsumerOnce::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_moved_value() {
        let data = vec![1, 2, 3];
        let consumer = BoxBiConsumerOnce::new(move |_x: &i32, _y: &i32| {
            // data is moved into the closure
            println!("Data length: {}", data.len());
        });
        consumer.accept(&5, &3);
        // data is no longer available here
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        closure.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }
}
