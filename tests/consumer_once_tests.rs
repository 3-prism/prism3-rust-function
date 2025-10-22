/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
/// Copyright (c) 2025.
/// 3-Prism Co. Ltd.
///
/// All rights reserved.
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
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
        consumer.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        consumer.accept_once(&5);
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
        chained.accept_once(&5);
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
        chained.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15, 4]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxConsumerOnce::<i32>::noop();
        noop.accept_once(&42);
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
        conditional.accept_once(&5);
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
        conditional.accept_once(&-5);
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
        conditional.accept_once(&5);
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
        conditional.accept_once(&-5);
        assert_eq!(*log.lock().unwrap(), vec![-6]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let boxed = consumer.into_box_once();
        boxed.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let func = consumer.into_fn_once();
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
        let func = consumer.into_fn_once();
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
        closure.accept_once(&5);
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
        chained.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_closure_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        let boxed = closure.into_box_once();
        boxed.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_closure_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        let func = closure.into_fn_once();
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
        chained.accept_once(&5);
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&5);
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
        let func = conditional.into_fn_once();
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
        chained.accept_once(&5);
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
        fn accept_once(self, value: &i32) {
            self.log.lock().unwrap().push(*value * self.multiplier);
        }

        // Note: We do not override into_box_once() and into_fn_once(),
        // but use the default implementations provided by the trait
    }

    #[test]
    fn test_custom_consumer_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 3);
        consumer.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![15]);
    }

    #[test]
    fn test_custom_consumer_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 2);
        let boxed = consumer.into_box_once();
        boxed.accept_once(&7);
        assert_eq!(*log.lock().unwrap(), vec![14]);
    }

    #[test]
    fn test_custom_consumer_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 4);
        let func = consumer.into_fn_once();
        func(&3);
        assert_eq!(*log.lock().unwrap(), vec![12]);
    }

    #[test]
    fn test_custom_consumer_into_box_chaining() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = CustomConsumer::new(l1, 2);
        let boxed = consumer.into_box_once();
        let chained = boxed.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 100);
        });
        chained.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 105]);
    }

    #[test]
    fn test_custom_consumer_with_generic_function() {
        let log = Arc::new(Mutex::new(Vec::new()));

        fn process_with_consumer<C>(consumer: C, value: &i32)
        where
            C: ConsumerOnce<i32>,
        {
            consumer.accept_once(value);
        }

        let consumer = CustomConsumer::new(log.clone(), 5);
        process_with_consumer(consumer, &6);
        assert_eq!(*log.lock().unwrap(), vec![30]);
    }

    #[test]
    fn test_custom_consumer_into_box_with_generic_function() {
        let log = Arc::new(Mutex::new(Vec::new()));

        fn process_with_box_consumer(consumer: BoxConsumerOnce<i32>, value: &i32) {
            consumer.accept_once(value);
        }

        let consumer = CustomConsumer::new(log.clone(), 3);
        let boxed = consumer.into_box_once();
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

        let chained = consumer1
            .into_box_once()
            .and_then(consumer2.into_box_once());
        chained.accept_once(&5);
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
        fn accept_once(self, value: &String) {
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
        let boxed = consumer.into_box_once();
        boxed.accept_once(&"Hello".to_string());
        assert_eq!(*log.lock().unwrap(), vec!["Log: Hello".to_string()]);
    }

    #[test]
    fn test_custom_string_consumer_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = StringLogger::new(log.clone(), "Info: ");
        let func = consumer.into_fn_once();
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
        fn accept_once(self, value: &i32) {
            *self.counter.lock().unwrap() += 1;
            self.value_log.lock().unwrap().push(*value);
        }
    }

    #[test]
    fn test_counting_consumer_into_box() {
        let counter = Arc::new(Mutex::new(0));
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingConsumer::new(counter.clone(), log.clone());
        let boxed = consumer.into_box_once();
        boxed.accept_once(&42);
        assert_eq!(*counter.lock().unwrap(), 1);
        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_counting_consumer_into_fn() {
        let counter = Arc::new(Mutex::new(0));
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingConsumer::new(counter.clone(), log.clone());
        let func = consumer.into_fn_once();
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
        conditional.accept_once(&-5);
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
        conditional.accept_once(&5);
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
        conditional.accept_once(&0);
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&5);
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&-5);
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&0);
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
        let func = conditional.into_fn_once();
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
        let func = conditional.into_fn_once();
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
        let func = conditional.into_fn_once();
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&4);
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
        let boxed = conditional.into_box_once();
        boxed.accept_once(&3);
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
        let func = conditional.into_fn_once();
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
        let func = conditional.into_fn_once();
        func(&3);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    // Additional comprehensive branch coverage tests for accept() method

    #[test]
    fn test_accept_with_always_true_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|_: &i32| true);
        conditional.accept_once(&42);
        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_accept_with_always_false_predicate() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|_: &i32| false);
        conditional.accept_once(&42);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_accept_with_complex_predicate_logic() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 10);
        });
        // Complex predicate: value is positive and even
        let conditional = consumer.when(|x: &i32| *x > 0 && *x % 2 == 0);
        conditional.accept_once(&4);
        assert_eq!(*log.lock().unwrap(), vec![40]);
    }

    #[test]
    fn test_accept_with_complex_predicate_logic_fails() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 10);
        });
        // Complex predicate: value is positive and even
        let conditional = consumer.when(|x: &i32| *x > 0 && *x % 2 == 0);
        // Test with odd number - fails the even check
        conditional.accept_once(&3);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_accept_with_complex_predicate_logic_fails_negative() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumerOnce::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 10);
        });
        // Complex predicate: value is positive and even
        let conditional = consumer.when(|x: &i32| *x > 0 && *x % 2 == 0);
        // Test with negative even number - fails the positive check
        conditional.accept_once(&-4);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    // Tests for and_then() method with conditional consumer

    #[test]
    fn test_and_then_predicate_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let conditional = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        })
        .when(|x: &i32| *x > 0);

        let chained = conditional.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        });

        chained.accept_once(&5);
        // First consumer executes (5), second consumer executes (10)
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_and_then_predicate_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let conditional = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        })
        .when(|x: &i32| *x > 0);

        let chained = conditional.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        });

        chained.accept_once(&-5);
        // First consumer doesn't execute (predicate false), second consumer still executes (-10)
        assert_eq!(*log.lock().unwrap(), vec![-10]);
    }

    #[test]
    fn test_and_then_multiple_conditionals() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();

        let conditional1 = BoxConsumerOnce::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        })
        .when(|x: &i32| *x > 0);

        let conditional2 = BoxConsumerOnce::new(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        })
        .when(|x: &i32| *x % 2 == 0);

        let chained = conditional1
            .and_then(conditional2)
            .and_then(move |x: &i32| {
                l3.lock().unwrap().push(*x + 100);
            });

        // Test with 6: positive (first passes), even (second passes), third always executes
        chained.accept_once(&6);
        assert_eq!(*log.lock().unwrap(), vec![6, 12, 106]);
    }
}

// ============================================================================
// to_box() and to_fn() Tests - Custom Consumer with Clone
// ============================================================================

#[cfg(test)]
mod custom_consumer_to_methods_tests {
    use super::*;

    /// Cloneable custom consumer for testing to_xxx() methods
    #[derive(Clone)]
    struct CloneableConsumer {
        log: Arc<Mutex<Vec<i32>>>,
        multiplier: i32,
    }

    impl CloneableConsumer {
        fn new(log: Arc<Mutex<Vec<i32>>>, multiplier: i32) -> Self {
            Self { log, multiplier }
        }
    }

    impl ConsumerOnce<i32> for CloneableConsumer {
        fn accept_once(self, value: &i32) {
            self.log.lock().unwrap().push(*value * self.multiplier);
        }
    }

    // Tests for to_box() method

    #[test]
    fn test_custom_consumer_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableConsumer::new(log.clone(), 2);
        let boxed = consumer.to_box_once();
        boxed.accept_once(&7);
        assert_eq!(*log.lock().unwrap(), vec![14]);
    }

    #[test]
    fn test_custom_consumer_to_box_multiple_times() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableConsumer::new(log.clone(), 3);

        // to_box() should allow being called multiple times since the consumer is Clone
        let boxed1 = consumer.to_box_once();
        let boxed2 = consumer.to_box_once();

        boxed1.accept_once(&5);
        boxed2.accept_once(&6);
        assert_eq!(*log.lock().unwrap(), vec![15, 18]);
    }

    #[test]
    fn test_custom_consumer_to_box_original_still_usable() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableConsumer::new(log.clone(), 2);

        let boxed = consumer.to_box_once();
        boxed.accept_once(&5);

        // Original consumer should still be usable since to_box() borrows, not consumes
        consumer.accept_once(&10);
        assert_eq!(*log.lock().unwrap(), vec![10, 20]);
    }

    // Tests for to_fn() method

    #[test]
    fn test_custom_consumer_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableConsumer::new(log.clone(), 4);
        let func = consumer.to_fn_once();
        func(&3);
        assert_eq!(*log.lock().unwrap(), vec![12]);
    }

    #[test]
    fn test_custom_consumer_to_fn_original_still_usable() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableConsumer::new(log.clone(), 2);

        let func = consumer.to_fn_once();
        func(&5);

        // Original consumer should still be usable since to_fn() borrows, not consumes
        consumer.accept_once(&10);
        assert_eq!(*log.lock().unwrap(), vec![10, 20]);
    }

    #[test]
    fn test_custom_consumer_to_box_chaining() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = CloneableConsumer::new(l1, 2);
        let boxed = consumer.to_box_once();
        let chained = boxed.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 100);
        });
        chained.accept_once(&5);
        // First consumer: 5 * 2 = 10
        // Second consumer receives original value: 5 + 100 = 105
        assert_eq!(*log.lock().unwrap(), vec![10, 105]);
    }
}

// ============================================================================
// to_box() and to_fn() Tests - Closure Implementation
// ============================================================================

#[cfg(test)]
mod closure_to_methods_tests {
    use super::*;

    /// A cloneable closure wrapper for testing to_xxx() methods on closures
    #[test]
    fn test_closure_to_box_requires_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let _l = log.clone();

        // Create a cloneable closure by wrapping in a struct
        #[derive(Clone)]
        struct ClonableClosure {
            log: Arc<Mutex<Vec<i32>>>,
            multiplier: i32,
        }

        impl ConsumerOnce<i32> for ClonableClosure {
            fn accept_once(self, value: &i32) {
                self.log.lock().unwrap().push(*value * self.multiplier);
            }
        }

        let log_clone = log.clone();
        let consumer = ClonableClosure {
            log: log_clone,
            multiplier: 2,
        };
        let boxed = consumer.to_box_once();
        boxed.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    /// Test that raw closures can be used with into_box() but not to_box()
    #[test]
    fn test_closure_into_box_works_without_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };

        // into_box() works directly on closures
        let boxed = closure.into_box_once();
        boxed.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_closure_into_fn_works_without_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };

        // into_fn() works directly on closures
        let func = closure.into_fn_once();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    /// Test closures with captured state to verify to_box() concept
    #[test]
    fn test_closure_state_capture_in_to_box_wrapper() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        // Note: Raw closures can't be cloned, but we can demonstrate the principle
        // by wrapping them in a struct that implements Clone
        #[derive(Clone)]
        struct ClosureWrapper<F: Clone + Fn(&i32)> {
            f: F,
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl<F: Clone + Fn(&i32)> ConsumerOnce<i32> for ClosureWrapper<F> {
            fn accept_once(self, value: &i32) {
                (self.f)(value);
                self.log.lock().unwrap().push(*value);
            }
        }

        let wrapper = ClosureWrapper {
            f: |x: &i32| println!("Value: {}", x),
            log: l,
        };

        let boxed = wrapper.to_box_once();
        boxed.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    /// Test that we cannot call to_box() on non-cloneable closures
    #[test]
    #[ignore] // This test is meant to demonstrate that the code doesn't compile
    fn test_closure_to_box_does_not_compile() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };

        // This will NOT compile because closure doesn't implement Clone:
        // let boxed = closure.to_box_once();  // Compile error!
        // Workaround: Use into_box() instead
        let boxed = closure.into_box_once();
        boxed.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }
}

// ============================================================================
// to_box() and to_fn() Tests - Closure Implementation
// ============================================================================

#[cfg(test)]
mod closure_to_xxx_methods_tests {
    use super::*;

    /// Test that closures implementing FnOnce can use into_box()
    #[test]
    fn test_closure_fnonce_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 3);
        };

        let boxed = closure.into_box_once();
        boxed.accept_once(&7);
        assert_eq!(*log.lock().unwrap(), vec![21]);
    }

    /// Test that closures implementing FnOnce can use into_fn()
    #[test]
    fn test_closure_fnonce_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x + 100);
        };

        let func = closure.into_fn_once();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![105]);
    }

    /// Test closure with and_then() through FnConsumerOnceOps
    #[test]
    fn test_closure_fnonce_ops_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let chained = (move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 50);
        });

        chained.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 55]);
    }

    /// Test closure with multiple and_then() chains through FnConsumerOnceOps
    #[test]
    fn test_closure_fnonce_ops_multiple_and_then() {
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
            l3.lock().unwrap().push(*x * 3);
        });

        chained.accept_once(&5);
        // First: 5 * 2 = 10
        // Second: 5 + 10 = 15 (operates on original value, not on result of first)
        // Third: 5 * 3 = 15 (operates on original value, not on result of second)
        assert_eq!(*log.lock().unwrap(), vec![10, 15, 15]);
    }

    /// Test closure with conditional when() method
    #[test]
    fn test_closure_into_box_with_when() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };

        let boxed = closure.into_box_once();
        let conditional = boxed.when(|x: &i32| *x > 0);
        conditional.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    /// Test closure with conditional when() returning false
    #[test]
    fn test_closure_into_box_with_when_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };

        let boxed = closure.into_box_once();
        let conditional = boxed.when(|x: &i32| *x > 0);
        conditional.accept_once(&-5);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    /// Test closure with or_else() branch
    #[test]
    fn test_closure_into_box_with_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let closure = move |x: &i32| {
            l1.lock().unwrap().push(*x);
        };

        let boxed = closure.into_box_once();
        let conditional = boxed.when(|x: &i32| *x > 0).or_else(move |x: &i32| {
            l2.lock().unwrap().push(-*x);
        });

        conditional.accept_once(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]); // or_else branch executed
    }

    /// Test chain of closures with and_then() followed by conditional
    #[test]
    fn test_closure_chain_then_conditional() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let chained = (move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });

        let boxed = chained;
        let conditional = boxed.when(|x: &i32| *x < 15);
        conditional.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]); // Both execute because condition is true (5 < 15)
    }

    /// Test closure with noop() for comparison
    #[test]
    fn test_closure_vs_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };

        let boxed = closure.into_box_once();
        boxed.accept_once(&5);

        let noop = BoxConsumerOnce::<i32>::noop();
        noop.accept_once(&10);

        // Only first consumer added value
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    /// Test that into_box() preserves closure behavior
    #[test]
    fn test_closure_into_box_preserves_behavior() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        // Create two identical closures
        let closure1 = move |x: &i32| {
            l1.lock().unwrap().push(*x);
        };
        let closure2 = move |x: &i32| {
            l2.lock().unwrap().push(*x);
        };

        // Convert first to box
        let boxed1 = closure1.into_box_once();
        boxed1.accept_once(&5);

        // Use second directly
        closure2.accept_once(&5);

        assert_eq!(*log.lock().unwrap(), vec![5, 5]);
    }

    /// Test into_fn() returns FnOnce
    #[test]
    fn test_closure_into_fn_is_fnonce() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };

        let func = closure.into_fn_once();
        // func can only be called once (FnOnce)
        func(&5);

        assert_eq!(*log.lock().unwrap(), vec![10]);
        // Note: Cannot call func again - it's FnOnce
    }

    /// Test complex closure capturing multiple values
    #[test]
    fn test_complex_closure_capture() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let multiplier = 5;
        let addition = 10;

        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * multiplier + addition);
        };

        let boxed = closure.into_box_once();
        boxed.accept_once(&3);

        assert_eq!(*log.lock().unwrap(), vec![25]); // 3 * 5 + 10 = 25
    }

    /// Test closure with String type
    #[test]
    fn test_closure_into_box_with_string() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let closure = move |s: &String| {
            l.lock().unwrap().push(s.clone());
        };

        let boxed = closure.into_box_once();
        boxed.accept_once(&"Hello".to_string());

        assert_eq!(*log.lock().unwrap(), vec!["Hello".to_string()]);
    }

    /// Test function pointers can use to_box() (function pointers are Copy/Clone)
    #[test]
    fn test_fn_pointer_to_box() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        fn increment_counter(_x: &i32) {
            COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        // Function pointers are Copy, can call to_box()
        let boxed = increment_counter.to_box_once();
        boxed.accept_once(&1);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }

    /// Test function pointers can use to_fn() (function pointers are Copy/Clone)
    #[test]
    fn test_fn_pointer_to_fn() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        fn increment_counter(_x: &i32) {
            COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        // Function pointers are Copy, can call to_fn()
        let func = increment_counter.to_fn_once();
        func(&1);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }

    /// Test function pointers preserve original after to_box()
    #[test]
    fn test_fn_pointer_to_box_preserves_original() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER2: AtomicUsize = AtomicUsize::new(0);

        fn increment_counter2(_x: &i32) {
            COUNTER2.fetch_add(1, Ordering::SeqCst);
        }

        // to_box() doesn't consume the function pointer
        let boxed = increment_counter2.to_box_once();
        boxed.accept_once(&1);

        // Original function pointer can still be used
        increment_counter2.accept_once(&2);

        assert_eq!(COUNTER2.load(Ordering::SeqCst), 2);
    }

    /// Test function pointers preserve original after to_fn()
    #[test]
    fn test_fn_pointer_to_fn_preserves_original() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER3: AtomicUsize = AtomicUsize::new(0);

        fn increment_counter3(_x: &i32) {
            COUNTER3.fetch_add(1, Ordering::SeqCst);
        }

        // to_fn() doesn't consume the function pointer
        let func = increment_counter3.to_fn_once();
        func(&1);

        // Original function pointer can still be used
        increment_counter3.accept_once(&2);

        assert_eq!(COUNTER3.load(Ordering::SeqCst), 2);
    }

    /// Test function pointers called to_box() multiple times
    #[test]
    fn test_fn_pointer_to_box_multiple_times() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER4: AtomicUsize = AtomicUsize::new(0);

        fn increment_counter4(_x: &i32) {
            COUNTER4.fetch_add(1, Ordering::SeqCst);
        }

        // Function pointers are Copy, can call to_box() multiple times
        let boxed1 = increment_counter4.to_box_once();
        let boxed2 = increment_counter4.to_box_once();

        boxed1.accept_once(&1);
        boxed2.accept_once(&2);

        assert_eq!(COUNTER4.load(Ordering::SeqCst), 2);
    }

    /// Test function pointers called to_fn() multiple times
    #[test]
    fn test_fn_pointer_to_fn_multiple_times() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER5: AtomicUsize = AtomicUsize::new(0);

        fn increment_counter5(_x: &i32) {
            COUNTER5.fetch_add(1, Ordering::SeqCst);
        }

        // Function pointers are Copy, can call to_fn() multiple times
        let func1 = increment_counter5.to_fn_once();
        let func2 = increment_counter5.to_fn_once();

        func1(&1);
        func2(&2);

        assert_eq!(COUNTER5.load(Ordering::SeqCst), 2);
    }
}

// ============================================================================
// Advanced to_box() and to_fn() Scenarios
// ============================================================================

#[cfg(test)]
mod advanced_to_methods_tests {
    use super::*;

    #[derive(Clone)]
    struct CountingCloneableConsumer {
        log: Arc<Mutex<Vec<i32>>>,
    }

    impl CountingCloneableConsumer {
        fn new(log: Arc<Mutex<Vec<i32>>>) -> Self {
            Self { log }
        }
    }

    impl ConsumerOnce<i32> for CountingCloneableConsumer {
        fn accept_once(self, value: &i32) {
            self.log.lock().unwrap().push(*value);
        }
    }

    #[test]
    fn test_to_box_then_chain_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let consumer = CountingCloneableConsumer::new(l1);
        let boxed = consumer.to_box_once();
        let chained = boxed.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        });

        chained.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_to_fn_then_call() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingCloneableConsumer::new(log.clone());
        let func = consumer.to_fn_once();
        func(&7);
        assert_eq!(*log.lock().unwrap(), vec![7]);
    }

    #[test]
    fn test_to_box_preserves_original_for_reuse() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingCloneableConsumer::new(log.clone());

        // Call to_box() multiple times
        let boxed1 = consumer.to_box_once();
        let boxed2 = consumer.to_box_once();

        boxed1.accept_once(&1);
        boxed2.accept_once(&2);

        // Original can still be used
        consumer.accept_once(&3);

        assert_eq!(*log.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_to_fn_preserves_original_for_reuse() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CountingCloneableConsumer::new(log.clone());

        // Call to_fn() multiple times
        let func1 = consumer.to_fn_once();
        let func2 = consumer.to_fn_once();

        func1(&10);
        func2(&20);

        // Original can still be used
        consumer.accept_once(&30);

        assert_eq!(*log.lock().unwrap(), vec![10, 20, 30]);
    }

    #[test]
    fn test_default_trait_implementation_via_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));

        /// Custom consumer implementing only accept()
        #[derive(Clone)]
        struct SimpleConsumer {
            multiplier: i32,
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl ConsumerOnce<i32> for SimpleConsumer {
            fn accept_once(self, value: &i32) {
                self.log.lock().unwrap().push(*value * self.multiplier);
            }
        }

        let consumer = SimpleConsumer {
            multiplier: 5,
            log: log.clone(),
        };

        // Uses default to_box() implementation which calls accept()
        let boxed = consumer.to_box_once();
        boxed.accept_once(&4);

        assert_eq!(*log.lock().unwrap(), vec![20]);
    }

    #[test]
    fn test_default_trait_implementation_via_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));

        #[derive(Clone)]
        struct SimpleConsumer {
            multiplier: i32,
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl ConsumerOnce<i32> for SimpleConsumer {
            fn accept_once(self, value: &i32) {
                self.log.lock().unwrap().push(*value * self.multiplier);
            }
        }

        let consumer = SimpleConsumer {
            multiplier: 3,
            log: log.clone(),
        };

        // Uses default to_fn() implementation which calls accept()
        let func = consumer.to_fn_once();
        func(&6);

        assert_eq!(*log.lock().unwrap(), vec![18]);
    }
}
