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
    fn test_when_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_when_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    #[test]
    fn test_when_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, _y: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let conditional =
            consumer
                .when(|x: &i32, y: &i32| *x > *y)
                .or_else(move |_x: &i32, y: &i32| {
                    l2.lock().unwrap().push(*y);
                });

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_when_or_else_false_branch() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, _y: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let conditional =
            consumer
                .when(|x: &i32, y: &i32| *x > *y)
                .or_else(move |_x: &i32, y: &i32| {
                    l2.lock().unwrap().push(*y);
                });

        // Condition is false (3 is not > 5), so else branch should execute
        conditional.accept(&3, &5);
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
        let data = [1, 2, 3];
        let consumer = BoxBiConsumerOnce::new(move |_x: &i32, _y: &i32| {
            // data is moved into the closure
            println!("Data length: {}", data.len());
        });
        consumer.accept(&5, &3);
        // data is no longer available here
    }

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer =
            BoxBiConsumerOnce::new_with_name("test_consumer", move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let boxed = conditional.into_box();
        boxed.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let chained = conditional.and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
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

    #[test]
    fn test_closure_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let func = closure.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }
}

#[cfg(test)]
mod debug_display_tests {
    use super::*;

    #[test]
    fn test_debug() {
        let consumer = BoxBiConsumerOnce::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxBiConsumerOnce"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = BoxBiConsumerOnce::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxBiConsumerOnce"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = BoxBiConsumerOnce::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxBiConsumerOnce");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxBiConsumerOnce::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxBiConsumerOnce(my_consumer)");
    }

    #[test]
    fn test_name_methods() {
        let mut consumer = BoxBiConsumerOnce::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);

        consumer.set_name("test");
        assert_eq!(consumer.name(), Some("test"));
    }
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        let consumer = BoxBiConsumerOnce::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let boxed = consumer.into_box();
        boxed.accept(&10, &20);
    }

    #[test]
    fn test_box_into_fn() {
        let consumer = BoxBiConsumerOnce::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let func = consumer.into_fn();
        func(&10, &20);
    }

    #[test]
    fn test_when_or_else_conversion() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let result = Arc::new(Mutex::new(0));
        let result_clone1 = result.clone();
        let result_clone2 = result.clone();

        let consumer = BoxBiConsumerOnce::new(move |x: &i32, _y: &i32| {
            *result_clone1.lock().unwrap() = *x;
        })
        .when(|x: &i32, y: &i32| x > y)
        .or_else(move |_x: &i32, y: &i32| {
            *result_clone2.lock().unwrap() = *y;
        });
        consumer.accept(&5, &3);
        assert_eq!(*result.lock().unwrap(), 5);
    }
}
