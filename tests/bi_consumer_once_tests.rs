/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Tests for BiConsumerOnce types

use prism3_function::{
    BiConsumerOnce,
    BoxBiConsumerOnce,
    FnBiConsumerOnceOps,
};
use std::sync::{
    Arc,
    Mutex,
};

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
        consumer.accept_once(&5, &3);
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

        chained.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxBiConsumerOnce::<i32, i32>::noop();
        noop.accept_once(&42, &10);
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

        conditional.accept_once(&5, &3);
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

        conditional.accept_once(&-5, &3);
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

        conditional.accept_once(&5, &3);
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
        conditional.accept_once(&3, &5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let box_consumer = closure.into_box_once();
        box_consumer.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let func = consumer.into_fn_once();
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
        consumer.accept_once(&5, &3);
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
        consumer.accept_once(&5, &3);
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&5, &3);
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
        let func = conditional.into_fn_once();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_box_false_branch() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let boxed = conditional.into_box_once();
        // Test false branch: should not execute when condition is not met
        boxed.accept_once(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    #[test]
    fn test_conditional_into_fn_false_branch() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let func = conditional.into_fn_once();
        // Test false branch: should not execute when condition is not met
        func(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
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
        chained.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_box_consumer_once_into_box_identity() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // into_box should return self for BoxBiConsumerOnce
        let boxed = consumer.into_box_once();
        boxed.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_box_consumer_once_into_fn_unwrap() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumerOnce::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // into_fn should unwrap the Box and return the inner function
        let func = consumer.into_fn_once();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
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
        closure.accept_once(&5, &3);
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

        chained.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_closure_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let func = closure.into_fn_once();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_closure_to_box() {
        // Create a cloneable closure using a function pointer
        fn add_values(x: &i32, y: &i32) {
            println!("Sum: {}", x + y);
        }

        // Function pointers implement Clone
        let closure: fn(&i32, &i32) = add_values;

        // Test to_box - should not consume the closure since it's cloneable
        let box_consumer = closure.to_box_once();
        box_consumer.accept_once(&5, &3);

        // Verify we can still use the closure again
        let box_consumer2 = closure.to_box_once();
        box_consumer2.accept_once(&10, &20);
    }

    #[test]
    fn test_closure_to_fn() {
        // Create a cloneable closure using a function pointer
        fn multiply_values(x: &i32, y: &i32) {
            println!("Product: {}", x * y);
        }

        // Function pointers implement Clone
        let closure: fn(&i32, &i32) = multiply_values;

        // Test to_fn - should not consume the closure since it's cloneable
        let func = closure.to_fn_once();
        func(&5, &3);

        // Verify we can still use the closure again
        let func2 = closure.to_fn_once();
        func2(&10, &20);
    }

    #[test]
    fn test_closure_to_box_with_state() {
        let log = Arc::new(Mutex::new(Vec::new()));

        // Create a cloneable struct that implements the closure behavior
        #[derive(Clone)]
        struct LoggingConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl LoggingConsumer {
            fn consume(&self, x: &i32, y: &i32) {
                self.log.lock().unwrap().push(*x + *y);
            }
        }

        let consumer = LoggingConsumer { log: log.clone() };
        let consumer_clone = consumer.clone();

        // Create a cloneable closure wrapper
        let closure = move |x: &i32, y: &i32| consumer.consume(x, y);

        // Test to_box
        let box_consumer = closure.to_box_once();
        box_consumer.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        // Use the cloned consumer
        let closure2 = move |x: &i32, y: &i32| consumer_clone.consume(x, y);
        let box_consumer2 = closure2.to_box_once();
        box_consumer2.accept_once(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    #[test]
    fn test_closure_to_fn_with_state() {
        let log = Arc::new(Mutex::new(Vec::new()));

        // Create a cloneable struct that implements the closure behavior
        #[derive(Clone)]
        struct LoggingConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl LoggingConsumer {
            fn consume(&self, x: &i32, y: &i32) {
                self.log.lock().unwrap().push(*x * *y);
            }
        }

        let consumer = LoggingConsumer { log: log.clone() };
        let consumer_clone = consumer.clone();

        // Create a cloneable closure wrapper
        let closure = move |x: &i32, y: &i32| consumer.consume(x, y);

        // Test to_fn
        let func = closure.to_fn_once();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![15]);

        // Use the cloned consumer
        let closure2 = move |x: &i32, y: &i32| consumer_clone.consume(x, y);
        let func2 = closure2.to_fn_once();
        func2(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![15, 200]);
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
        let boxed = consumer.into_box_once();
        boxed.accept_once(&10, &20);
    }

    #[test]
    fn test_box_into_fn() {
        let consumer = BoxBiConsumerOnce::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let func = consumer.into_fn_once();
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
        consumer.accept_once(&5, &3);
        assert_eq!(*result.lock().unwrap(), 5);
    }
}

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod default_implementation_tests {
    use super::*;

    /// Custom struct that implements BiConsumerOnce
    /// to test the default implementations of into_box, into_fn, to_box, to_fn
    struct CustomBiConsumerOnce {
        log: Arc<Mutex<Vec<i32>>>,
        multiplier: i32,
    }

    impl CustomBiConsumerOnce {
        fn new(log: Arc<Mutex<Vec<i32>>>, multiplier: i32) -> Self {
            Self { log, multiplier }
        }
    }

    impl BiConsumerOnce<i32, i32> for CustomBiConsumerOnce {
        fn accept_once(self, first: &i32, second: &i32) {
            self.log
                .lock()
                .unwrap()
                .push((*first + *second) * self.multiplier);
        }
    }

    impl Clone for CustomBiConsumerOnce {
        fn clone(&self) -> Self {
            Self {
                log: self.log.clone(),
                multiplier: self.multiplier,
            }
        }
    }

    #[test]
    fn test_custom_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 2);

        // Test default into_box implementation
        let boxed = consumer.into_box_once();
        boxed.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![16]); // (5 + 3) * 2 = 16
    }

    #[test]
    fn test_custom_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 3);

        // Test default into_fn implementation
        let func = consumer.into_fn_once();
        func(&4, &2);
        assert_eq!(*log.lock().unwrap(), vec![18]); // (4 + 2) * 3 = 18
    }

    #[test]
    fn test_custom_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 4);

        // Test default to_box implementation
        let boxed = consumer.to_box_once();
        boxed.accept_once(&3, &2);
        assert_eq!(*log.lock().unwrap(), vec![20]); // (3 + 2) * 4 = 20

        // Verify original consumer is still usable (not consumed)
        consumer.accept_once(&1, &1);
        assert_eq!(*log.lock().unwrap(), vec![20, 8]); // (1 + 1) * 4 = 8
    }

    #[test]
    fn test_custom_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 5);

        // Test default to_fn implementation
        let func = consumer.to_fn_once();
        func(&2, &3);
        assert_eq!(*log.lock().unwrap(), vec![25]); // (2 + 3) * 5 = 25

        // Verify original consumer is still usable (not consumed)
        consumer.accept_once(&1, &0);
        assert_eq!(*log.lock().unwrap(), vec![25, 5]); // (1 + 0) * 5 = 5
    }

    #[test]
    fn test_custom_consumer_composition() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer1 = CustomBiConsumerOnce::new(log.clone(), 2);

        // Test composing with another consumer using and_then
        let l2 = log.clone();
        let chained = consumer1.into_box_once().and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![16, 15]); // (5+3)*2=16, 5*3=15
    }

    #[test]
    fn test_custom_consumer_with_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 10);

        // Test using when with custom consumer
        let conditional = consumer.into_box_once().when(|x: &i32, y: &i32| *x > *y);

        conditional.accept_once(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![80]); // (5+3)*10=80

        // Test when condition is false
        let log2 = Arc::new(Mutex::new(Vec::new()));
        let consumer2 = CustomBiConsumerOnce::new(log2.clone(), 10);
        let conditional2 = consumer2.into_box_once().when(|x: &i32, y: &i32| *x > *y);

        conditional2.accept_once(&3, &5);
        assert_eq!(*log2.lock().unwrap(), vec![]); // Condition false, not executed
    }

    #[test]
    fn test_custom_consumer_multiple_clones() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 2);

        // Clone multiple times and use each clone
        let boxed1 = consumer.to_box_once();
        let boxed2 = consumer.to_box_once();
        let boxed3 = consumer.to_box_once();

        boxed1.accept_once(&1, &1);
        boxed2.accept_once(&2, &2);
        boxed3.accept_once(&3, &3);

        assert_eq!(*log.lock().unwrap(), vec![4, 8, 12]);
        // (1+1)*2=4, (2+2)*2=8, (3+3)*2=12
    }

    #[test]
    fn test_custom_consumer_fn_multiple_uses() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomBiConsumerOnce::new(log.clone(), 3);

        // Get multiple functions from the same consumer
        let fn1 = consumer.to_fn_once();
        let fn2 = consumer.to_fn_once();

        fn1(&2, &2);
        fn2(&3, &3);

        assert_eq!(*log.lock().unwrap(), vec![12, 18]);
        // (2+2)*3=12, (3+3)*3=18
    }
}
