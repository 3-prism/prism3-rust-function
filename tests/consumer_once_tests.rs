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

// ============================================================================
// Custom ConsumerOnce Tests - Testing Default into_xxx() Implementation
// ============================================================================

#[cfg(test)]
mod custom_consumer_once_tests {
    use super::*;

    /// Custom consumer that increments a counter
    struct CustomConsumer {
        log: Arc<Mutex<Vec<i32>>>,
        multiplier: i32,
    }

    impl CustomConsumer {
        fn new(log: Arc<Mutex<Vec<i32>>>, multiplier: i32) -> Self {
            Self { log, multiplier }
        }
    }

    impl ConsumerOnce<i32> for CustomConsumer {
        fn accept(self, value: &i32) {
            self.log.lock().unwrap().push(*value * self.multiplier);
        }

        // 注意：我们不重写 into_box() 和 into_fn()，
        // 而是使用 trait 提供的默认实现
    }

    #[test]
    fn test_custom_consumer_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 3);
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![15]);
    }

    #[test]
    fn test_custom_consumer_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 2);
        let boxed = consumer.into_box();
        boxed.accept(&7);
        assert_eq!(*log.lock().unwrap(), vec![14]);
    }

    #[test]
    fn test_custom_consumer_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 4);
        let func = consumer.into_fn();
        func(&3);
        assert_eq!(*log.lock().unwrap(), vec![12]);
    }

    #[test]
    fn test_custom_consumer_into_box_chaining() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = CustomConsumer::new(l1, 2);
        let boxed = consumer.into_box();
        let chained = boxed.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 100);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 105]);
    }

    #[test]
    fn test_custom_consumer_with_generic_function() {
        let log = Arc::new(Mutex::new(Vec::new()));

        fn process_with_consumer<C>(consumer: C, value: &i32)
        where
            C: ConsumerOnce<i32>,
        {
            consumer.accept(value);
        }

        let consumer = CustomConsumer::new(log.clone(), 5);
        process_with_consumer(consumer, &6);
        assert_eq!(*log.lock().unwrap(), vec![30]);
    }

    #[test]
    fn test_custom_consumer_into_box_with_generic_function() {
        let log = Arc::new(Mutex::new(Vec::new()));

        fn process_with_box_consumer(consumer: BoxConsumerOnce<i32>, value: &i32) {
            consumer.accept(value);
        }

        let consumer = CustomConsumer::new(log.clone(), 3);
        let boxed = consumer.into_box();
        process_with_box_consumer(boxed, &8);
        assert_eq!(*log.lock().unwrap(), vec![24]);
    }

    #[test]
    fn test_multiple_custom_consumers_chained() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let consumer1 = CustomConsumer::new(l1, 2);
        let consumer2 = CustomConsumer::new(l2, 3);

        let chained = consumer1.into_box().and_then(consumer2.into_box());
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    /// Custom consumer with String type
    struct StringLogger {
        log: Arc<Mutex<Vec<String>>>,
        prefix: String,
    }

    impl StringLogger {
        fn new(log: Arc<Mutex<Vec<String>>>, prefix: impl Into<String>) -> Self {
            Self {
                log,
                prefix: prefix.into(),
            }
        }
    }

    impl ConsumerOnce<String> for StringLogger {
        fn accept(self, value: &String) {
            self.log
                .lock()
                .unwrap()
                .push(format!("{}{}", self.prefix, value));
        }
    }

    #[test]
    fn test_custom_string_consumer_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = StringLogger::new(log.clone(), "Log: ");
        let boxed = consumer.into_box();
        boxed.accept(&"Hello".to_string());
        assert_eq!(*log.lock().unwrap(), vec!["Log: Hello".to_string()]);
    }

    #[test]
    fn test_custom_string_consumer_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = StringLogger::new(log.clone(), "Info: ");
        let func = consumer.into_fn();
        func(&"World".to_string());
        assert_eq!(*log.lock().unwrap(), vec!["Info: World".to_string()]);
    }

    /// Custom consumer that counts how many times it was supposed to be called
    struct CountingConsumer {
        counter: Arc<Mutex<usize>>,
        value_log: Arc<Mutex<Vec<i32>>>,
    }

    impl CountingConsumer {
        fn new(counter: Arc<Mutex<usize>>, value_log: Arc<Mutex<Vec<i32>>>) -> Self {
            Self { counter, value_log }
        }
    }

    impl ConsumerOnce<i32> for CountingConsumer {
        fn accept(self, value: &i32) {
            *self.counter.lock().unwrap() += 1;
            self.value_log.lock().unwrap().push(*value);
        }
    }

    #[test]
    fn test_counting_consumer_into_box() {
        let counter = Arc::new(Mutex::new(0));
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingConsumer::new(counter.clone(), log.clone());
        let boxed = consumer.into_box();
        boxed.accept(&42);
        assert_eq!(*counter.lock().unwrap(), 1);
        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_counting_consumer_into_fn() {
        let counter = Arc::new(Mutex::new(0));
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingConsumer::new(counter.clone(), log.clone());
        let func = consumer.into_fn();
        func(&99);
        assert_eq!(*counter.lock().unwrap(), 1);
        assert_eq!(*log.lock().unwrap(), vec![99]);
    }
}

// ============================================================================
// BoxConditionalConsumerOnce Focused Tests
// ============================================================================

#[cfg(test)]
mod box_conditional_consumer_once_tests {
    use super::*;

    // Tests for accept() method

    #[test]
    fn test_accept_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&-5);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_accept_predicate_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_accept_predicate_boundary() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        // Test boundary case - predicate checks > 0, so 0 should be false
        conditional.accept(&0);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    // Tests for into_box() method

    #[test]
    fn test_into_box_predicate_true() {
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
    fn test_into_box_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let boxed = conditional.into_box();
        boxed.accept(&-5);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_into_box_predicate_boundary() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let boxed = conditional.into_box();
        boxed.accept(&0);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    // Tests for into_fn() method

    #[test]
    fn test_into_fn_predicate_true() {
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
    fn test_into_fn_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let func = conditional.into_fn();
        func(&-5);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_into_fn_predicate_boundary() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let func = conditional.into_fn();
        func(&0);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    // Additional tests for into_box() and into_fn() with complex predicates

    #[test]
    fn test_into_box_complex_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let conditional = consumer.when(|x: &i32| *x % 2 == 0);
        let boxed = conditional.into_box();
        boxed.accept(&4);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_box_complex_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let conditional = consumer.when(|x: &i32| *x % 2 == 0);
        let boxed = conditional.into_box();
        boxed.accept(&3);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_into_fn_complex_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let conditional = consumer.when(|x: &i32| *x % 2 == 0);
        let func = conditional.into_fn();
        func(&4);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_fn_complex_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let conditional = consumer.when(|x: &i32| *x % 2 == 0);
        let func = conditional.into_fn();
        func(&3);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }
}
