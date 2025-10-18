/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Tests for ReadonlyBiConsumer types

use prism3_function::{
    ArcReadonlyBiConsumer, BoxReadonlyBiConsumer, FnReadonlyBiConsumerOps, RcReadonlyBiConsumer,
    ReadonlyBiConsumer,
};
use std::rc::Rc;
use std::sync::Arc;

#[cfg(test)]
mod box_readonly_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        consumer.accept(&5, &3);
    }

    #[test]
    fn test_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let chained = BoxReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32, _y: &i32| {
            *c2.lock().unwrap() += 1;
        });

        chained.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_noop() {
        let noop = BoxReadonlyBiConsumer::<i32, i32>::noop();
        noop.accept(&42, &10);
        // Should not panic
    }

    #[test]
    fn test_into_box() {
        let closure = |x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        };
        let box_consumer = closure.into_box();
        box_consumer.accept(&5, &3);
    }

    #[test]
    fn test_into_fn() {
        let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let func = consumer.into_fn();
        func(&5, &3);
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxReadonlyBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }
}

#[cfg(test)]
mod arc_readonly_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        consumer.accept(&5, &3);
    }

    #[test]
    fn test_clone() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let consumer = ArcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let clone1 = consumer.clone();
        let clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

        clone2.accept(&10, &2);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_and_then() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let first = ArcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let second = ArcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let chained = first.and_then(&second);

        chained.accept(&5, &3);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_to_fn() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let consumer = ArcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_into_box() {
        let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
    }

    #[test]
    fn test_into_rc() {
        let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
    }

    #[test]
    fn test_name() {
        let mut consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }
}

#[cfg(test)]
mod rc_readonly_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        consumer.accept(&5, &3);
    }

    #[test]
    fn test_clone() {
        let counter = Rc::new(std::cell::Cell::new(0));
        let c = counter.clone();
        let consumer = RcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c.set(c.get() + 1);
        });

        let clone1 = consumer.clone();
        let clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(counter.get(), 1);

        clone2.accept(&10, &2);
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_and_then() {
        let counter = Rc::new(std::cell::Cell::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let first = RcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c1.set(c1.get() + 1);
        });
        let second = RcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c2.set(c2.get() + 1);
        });

        let chained = first.and_then(&second);

        chained.accept(&5, &3);
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_to_fn() {
        let counter = Rc::new(std::cell::Cell::new(0));
        let c = counter.clone();
        let consumer = RcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c.set(c.get() + 1);
        });

        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_into_box() {
        let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
    }

    #[test]
    fn test_name() {
        let mut consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let closure = |x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        };
        closure.accept(&5, &3);
    }

    #[test]
    fn test_closure_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let chained = (move |_x: &i32, _y: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32, _y: &i32| {
            *c2.lock().unwrap() += 1;
        });

        chained.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 2);
    }
}
