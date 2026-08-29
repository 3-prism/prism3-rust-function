// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::sync::Arc;
use std::sync::Mutex;

///
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
use qubit_function::BoxConsumerOnce;
///
/// # ConsumerOnce Tests
///
/// Unit tests for the ConsumerOnce trait and its implementations.
use qubit_function::ConsumerOnce;

// ============================================================================
// BoxConsumerOnce Tests
// ============================================================================

#[cfg(test)]
mod custom_consumer_once_tests {
    use super::Arc;
    use super::ConsumerOnce;
    use super::Mutex;

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
            self.log
                .lock()
                .expect("mutex should not be poisoned")
                .push(*value * self.multiplier);
        }

        // Note: We do not override into_box() and into_fn(),
        // but use the default implementations provided by the trait
    }

    #[test]
    fn test_custom_consumer_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CustomConsumer::new(log.clone(), 3);
        consumer.accept(&5);
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![15]);
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
        assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![30]);
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
                .expect("mutex should not be poisoned")
                .push(format!("{}{}", self.prefix, value));
        }
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
            *self.counter.lock().expect("mutex should not be poisoned") += 1;
            self.value_log
                .lock()
                .expect("mutex should not be poisoned")
                .push(*value);
        }
    }
}

// ============================================================================
// BoxConditionalConsumerOnce Focused Tests
// ============================================================================
