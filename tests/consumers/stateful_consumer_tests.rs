/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for StatefulConsumer types

use prism3_function::{
    ArcConsumer,
    ArcStatefulConsumer,
    BoxConsumer,
    BoxStatefulConsumer,
    Consumer,
    ConsumerOnce,
    FnConsumerOps,
    RcConsumer,
    RcStatefulConsumer,
    StatefulConsumer,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

// ============================================================================
// BoxConsumer Tests
// ============================================================================

#[cfg(test)]
mod test_box_consumer {
    use super::*;

    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |s: &String| {
            *l.lock().unwrap() = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept(&text);
        assert_eq!(*log.lock().unwrap(), "Got: hello");

        // Vec
        let log = Arc::new(Mutex::new(0));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |v: &Vec<i32>| {
            *l.lock().unwrap() = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept(&numbers);
        assert_eq!(*log.lock().unwrap(), 3);

        // bool
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |b: &bool| {
            *l.lock().unwrap() = if *b { "true" } else { "false" }.to_string();
        });
        let flag = true;
        consumer.accept(&flag);
        assert_eq!(*log.lock().unwrap(), "true");
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });

        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]); // 5*2=10, 5+10=15
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x + 1);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l3.lock().unwrap().push(*x - 5);
        });

        let value = 10;
        consumer.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![11, 20, 5]); // 10+1=11, 10*2=20, 10-5=5
    }

    #[test]
    fn test_and_then_with_box_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let c1 = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        });
        let c2 = BoxStatefulConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });
        let mut combined = c1.and_then(c2);

        let value = 5;
        combined.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_noop() {
        let noop = BoxConsumer::<i32>::noop();
        let value = 42;
        noop.accept(&value);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new_with_name("test_consumer", move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    // print and print_with methods have been removed

    #[test]
    fn test_if_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0);

        let positive = 5;
        conditional.accept(&positive);
        assert_eq!(*log.lock().unwrap(), vec![5]);

        let negative = -5;
        conditional.accept(&negative);
        assert_eq!(*log.lock().unwrap(), vec![5]); // Unchanged
    }

    #[test]
    fn test_if_then_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0).or_else(move |x: &i32| {
            l2.lock().unwrap().push(-*x);
        });

        let positive = 5;
        conditional.accept(&positive);
        assert_eq!(*log.lock().unwrap(), vec![5]);

        let negative = -5;
        conditional.accept(&negative);
        assert_eq!(*log.lock().unwrap(), vec![5, 5]); // -(-5) = 5
    }

    #[test]
    fn test_debug() {
        let consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut func = consumer.into_fn();
        func(&5);
        func(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_into_rc_from_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }
}

// ============================================================================
// ArcConsumer Tests
// ============================================================================

#[cfg(test)]
mod test_arc_consumer {
    use super::*;

    #[test]
    fn test_new() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut clone = consumer.clone();

        consumer.accept(&5);
        clone.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = ArcStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        });
        let second = ArcStatefulConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });
        let mut chained = first.and_then(second);

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_thread_safety() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        let mut c1 = consumer.clone();
        let mut c2 = consumer.clone();

        let h1 = std::thread::spawn(move || {
            c1.accept(&1);
        });

        let h2 = std::thread::spawn(move || {
            c2.accept(&2);
        });

        h1.join().unwrap();
        h2.join().unwrap();

        let mut result = log.lock().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_noop() {
        let noop = ArcConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new_with_name("test_consumer", move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_noop_stateful() {
        let mut noop = ArcStatefulConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_debug() {
        let consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = ArcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut func = consumer.into_fn();
        func(&5);
        func(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let arc_consumer = consumer.into_arc();
        let mut arc_consumer2 = arc_consumer.clone();
        arc_consumer2.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    // ============================================================================
    // ArcConsumer ConsumerOnce Tests
    // ============================================================================

    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        consumer.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_accept_once_with_different_types() {
        // String
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |s: &String| {
            *l.lock().unwrap() = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept_once(&text);
        assert_eq!(*log.lock().unwrap(), "Got: hello");

        // Vec
        let log = Arc::new(Mutex::new(0));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |v: &Vec<i32>| {
            *l.lock().unwrap() = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept_once(&numbers);
        assert_eq!(*log.lock().unwrap(), 3);

        // bool
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |b: &bool| {
            *l.lock().unwrap() = if *b { "true" } else { "false" }.to_string();
        });
        let flag = true;
        consumer.accept_once(&flag);
        assert_eq!(*log.lock().unwrap(), "true");
    }

    #[test]
    fn test_into_box_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let box_consumer_once = consumer.into_box_once();
        box_consumer_once.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_fn_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let func = consumer.into_fn_once();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_to_box_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let box_consumer_once = consumer.to_box_once();
        box_consumer_once.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        // Original consumer still usable
        consumer.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    }

    #[test]
    fn test_to_fn_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let func = consumer.to_fn_once();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        // Original consumer still usable
        consumer.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![5, 3]);
    }

    #[test]
    fn test_consumer_once_with_state_modification() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            counter += 1;
            l.lock().unwrap().push(*x + counter);
        });
        consumer.accept_once(&10);
        assert_eq!(*log.lock().unwrap(), vec![11]); // 10 + 1
    }

    #[test]
    fn test_consumer_once_consumes_self() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        // This should compile - accept_once consumes self
        consumer.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);

        // This would not compile - consumer is moved
        // consumer.accept(&3); // Would not compile
    }

    /// Test that ArcConsumer can work with non-Send + non-Sync types
    ///
    /// This test verifies that the relaxed generic constraints (T: 'static instead
    /// of T: Send + Sync + 'static) allow ArcConsumer to be created for types that
    /// are not thread-safe, as long as we only pass references to them.
    #[test]
    fn test_with_non_send_sync_type() {
        // Rc<RefCell<i32>> is neither Send nor Sync
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        // This should compile now with relaxed constraints
        let consumer = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = *value.borrow();
            l.lock().unwrap().push(val);
        });

        let value = Rc::new(RefCell::new(42));
        consumer.accept(&value);

        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    /// Test that ArcConsumer with non-Send type can be cloned and used
    #[test]
    fn test_clone_with_non_send_sync_type() {
        type NonSendType = Rc<RefCell<String>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = value.borrow().clone();
            l.lock().unwrap().push(val);
        });

        let consumer2 = consumer.clone();

        let value1 = Rc::new(RefCell::new("hello".to_string()));
        let value2 = Rc::new(RefCell::new("world".to_string()));

        consumer.accept(&value1);
        consumer2.accept(&value2);

        let result = log.lock().unwrap().clone();
        assert_eq!(result, vec!["hello".to_string(), "world".to_string()]);
    }

    /// Test that ArcConsumer with non-Send type can be chained
    #[test]
    fn test_and_then_with_non_send_sync_type() {
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let first = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = *value.borrow();
            l1.lock().unwrap().push(val * 2);
        });

        let second = ArcConsumer::<NonSendType>::new(move |value: &NonSendType| {
            let val = *value.borrow();
            l2.lock().unwrap().push(val + 10);
        });

        let chained = first.and_then(second);

        let value = Rc::new(RefCell::new(5));
        chained.accept(&value);

        assert_eq!(*log.lock().unwrap(), vec![10, 15]); // 5*2=10, 5+10=15
    }
}

// ============================================================================
// RcConsumer Tests
// ============================================================================

#[cfg(test)]
mod test_rc_consumer {
    use super::*;

    #[test]
    fn test_new() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let value = 5;
        consumer.accept(&value);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut clone = consumer.clone();

        consumer.accept(&5);
        clone.accept(&10);
        assert_eq!(*log.borrow(), vec![5, 10]);
    }

    #[test]
    fn test_and_then() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = RcStatefulConsumer::new(move |x: &i32| {
            l1.borrow_mut().push(*x * 2);
        });
        let second = RcStatefulConsumer::new(move |x: &i32| {
            l2.borrow_mut().push(*x + 10);
        });
        let mut chained = first.and_then(second);

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.borrow(), vec![10, 15]);
    }

    #[test]
    fn test_noop() {
        let noop = RcConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_new_with_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new_with_name("test_consumer", move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_noop_stateful() {
        let mut noop = RcStatefulConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_debug() {
        let consumer = RcStatefulConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let mut consumer = RcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = RcStatefulConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = RcStatefulConsumer::new(|_x: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut func = consumer.into_fn();
        func(&5);
        func(&10);
        assert_eq!(*log.borrow(), vec![5, 10]);
    }

    #[test]
    fn test_into_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let rc_consumer = consumer.into_rc();
        let mut rc_consumer2 = rc_consumer.clone();
        rc_consumer2.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

#[cfg(test)]
mod test_conversions {
    use super::*;

    #[test]
    fn test_box_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let box_consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut rc_consumer: RcStatefulConsumer<i32> = box_consumer.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    // RcConsumer cannot be converted to ArcConsumer because Rc is not Send

    // ============================================================================
    // RcConsumer ConsumerOnce Tests
    // ============================================================================

    #[test]
    fn test_accept_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        consumer.accept_once(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_accept_once_with_different_types() {
        // String
        let log = Rc::new(RefCell::new(String::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |s: &String| {
            *l.borrow_mut() = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept_once(&text);
        assert_eq!(*log.borrow(), "Got: hello");

        // Vec
        let log = Rc::new(RefCell::new(0));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |v: &Vec<i32>| {
            *l.borrow_mut() = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept_once(&numbers);
        assert_eq!(*log.borrow(), 3);

        // bool
        let log = Rc::new(RefCell::new(String::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |b: &bool| {
            *l.borrow_mut() = if *b { "true" } else { "false" }.to_string();
        });
        let flag = true;
        consumer.accept_once(&flag);
        assert_eq!(*log.borrow(), "true");
    }

    #[test]
    fn test_into_box_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let box_consumer_once = consumer.into_box_once();
        box_consumer_once.accept_once(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_into_fn_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let func = consumer.into_fn_once();
        func(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_to_box_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let box_consumer_once = consumer.to_box_once();
        box_consumer_once.accept_once(&5);
        assert_eq!(*log.borrow(), vec![5]);
        // Original consumer still usable
        consumer.accept(&3);
        assert_eq!(*log.borrow(), vec![5, 3]);
    }

    #[test]
    fn test_to_fn_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let func = consumer.to_fn_once();
        func(&5);
        assert_eq!(*log.borrow(), vec![5]);
        // Original consumer still usable
        consumer.accept(&3);
        assert_eq!(*log.borrow(), vec![5, 3]);
    }

    #[test]
    fn test_consumer_once_with_state_modification() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            counter += 1;
            l.borrow_mut().push(*x + counter);
        });
        consumer.accept_once(&10);
        assert_eq!(*log.borrow(), vec![11]); // 10 + 1
    }

    #[test]
    fn test_consumer_once_consumes_self() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });

        // This should compile - accept_once consumes self
        consumer.accept_once(&5);
        assert_eq!(*log.borrow(), vec![5]);

        // This would not compile - consumer is moved
        // consumer.accept(&3); // Would not compile
    }
}

// ============================================================================
// Unified Interface Tests
// ============================================================================

#[cfg(test)]
mod test_unified_interface {
    use super::*;

    fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: &i32) -> i32 {
        consumer.accept(value);
        *value // Return original value since Consumer doesn't modify input
    }

    #[test]
    fn test_with_box_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let result = apply_consumer(&mut consumer, &5);
        assert_eq!(result, 5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_with_arc_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });
        let result = apply_consumer(&mut consumer, &5);
        assert_eq!(result, 5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    #[test]
    fn test_with_rc_consumer() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 2);
        });
        let result = apply_consumer(&mut consumer, &5);
        assert_eq!(result, 5);
        assert_eq!(*log.borrow(), vec![10]);
    }
}

// ============================================================================
// FnConsumerOps Tests
// ============================================================================

#[cfg(test)]
mod test_fn_consumer_ops {
    use super::*;

    #[test]
    fn test_and_then_with_closure() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let chained = (move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };
        let boxed: BoxConsumer<i32> = Consumer::into_box(closure);
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };
        let arc: ArcConsumer<i32> = Consumer::into_arc(closure);
        arc.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.borrow_mut().push(*x);
        };
        let rc: RcConsumer<i32> = Consumer::into_rc(closure);
        rc.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_into_fn() {
        // Test into_fn in impl<T, F> Consumer<T> for F
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        let func = Consumer::into_fn(closure);
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    // Test closure's to_xxx methods
    // Note: Only Clone closures can use to_xxx methods
    // Since standard closures do not implement Clone, we use function pointers (function pointers implement Clone)

    #[test]
    fn test_closure_to_box_with_fn_pointer() {
        // Use Arc<Mutex> to verify function call
        let counter = Arc::new(Mutex::new(0));
        let c1 = counter.clone();

        fn make_consumer(c: Arc<Mutex<i32>>) -> impl FnMut(&i32) + Clone {
            move |x: &i32| {
                *c.lock().unwrap() += *x;
            }
        }

        let consumer_fn = make_consumer(c1);
        let mut boxed = consumer_fn.to_box();
        boxed.accept(&5);
        boxed.accept(&10);

        assert_eq!(*counter.lock().unwrap(), 15);

        // Verify original closure is still available
        let original = consumer_fn;
        let mut func = original;
        func(&7);
        assert_eq!(*counter.lock().unwrap(), 22);
    }

    #[test]
    fn test_closure_to_rc_with_fn_pointer() {
        let counter = Rc::new(RefCell::new(0));
        let c1 = counter.clone();

        fn make_consumer(c: Rc<RefCell<i32>>) -> impl FnMut(&i32) + Clone {
            move |x: &i32| {
                *c.borrow_mut() += *x * 2;
            }
        }

        let consumer_fn = make_consumer(c1);
        let mut rc = consumer_fn.to_rc();
        rc.accept(&3);
        rc.accept(&4);

        assert_eq!(*counter.borrow(), 14); // 3*2 + 4*2

        // Verify original closure is still available
        let original = consumer_fn;
        let mut func = original;
        func(&5);
        assert_eq!(*counter.borrow(), 24); // 14 + 5*2
    }

    #[test]
    fn test_closure_to_arc_with_fn_pointer() {
        let counter = Arc::new(Mutex::new(0));
        let c1 = counter.clone();

        fn make_consumer(c: Arc<Mutex<i32>>) -> impl FnMut(&i32) + Clone + Send {
            move |x: &i32| {
                *c.lock().unwrap() += *x * 3;
            }
        }

        let consumer_fn = make_consumer(c1);
        let mut arc = consumer_fn.to_arc();
        arc.accept(&2);
        arc.accept(&3);

        assert_eq!(*counter.lock().unwrap(), 15); // 2*3 + 3*3

        // Verify original closure is still available
        let original = consumer_fn;
        let mut func = original;
        func(&4);
        assert_eq!(*counter.lock().unwrap(), 27); // 15 + 4*3
    }

    #[test]
    fn test_closure_to_fn_with_fn_pointer() {
        let counter = Arc::new(Mutex::new(0));
        let c1 = counter.clone();

        fn make_consumer(c: Arc<Mutex<i32>>) -> impl FnMut(&i32) + Clone {
            move |x: &i32| {
                *c.lock().unwrap() += *x + 10;
            }
        }

        // Use different instances for to_fn and subsequent tests
        let consumer_fn1 = make_consumer(c1.clone());
        let consumer_fn2 = make_consumer(c1.clone());

        // Test to_fn() - first instance
        let mut func = consumer_fn1.to_fn();
        func(&5); // 5 + 10 = 15
        func(&7); // 7 + 10 = 17

        // Verify first part result
        assert_eq!(*counter.lock().unwrap(), 32); // 15 + 17

        // Use second independent instance to verify original closure is still available
        let mut original_func = consumer_fn2;
        original_func(&3); // 3 + 10 = 13
        assert_eq!(*counter.lock().unwrap(), 45); // 32 + 13
    }
}

// ============================================================================
// Name Tests
// ============================================================================

#[cfg(test)]
mod test_consumer_names {
    use super::*;

    #[test]
    fn test_box_consumer_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        consumer.set_name("logger");
        assert_eq!(consumer.name(), Some("logger"));
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_box_consumer_set_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    #[test]
    fn test_arc_consumer_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        consumer.set_name("logger");
        assert_eq!(consumer.name(), Some("logger"));
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_consumer_set_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }

    #[test]
    fn test_rc_consumer_with_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        consumer.set_name("logger");
        assert_eq!(consumer.name(), Some("logger"));
        consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_rc_consumer_set_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        assert_eq!(consumer.name(), None);
        consumer.set_name("my_consumer");
        assert_eq!(consumer.name(), Some("my_consumer"));
    }
}

// ============================================================================
// to_fn Tests
// ============================================================================

#[cfg(test)]
mod test_to_fn {
    use super::*;

    #[test]
    fn test_arc_consumer_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut func = consumer.to_fn();
        func(&5);
        func(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_rc_consumer_to_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut func = consumer.to_fn();
        func(&5);
        func(&10);
        assert_eq!(*log.borrow(), vec![5, 10]);
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod test_edge_cases {
    use super::*;

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
            l.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| true);
        conditional.accept(&5);
        conditional.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_if_then_with_always_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| false);
        conditional.accept(&5);
        conditional.accept(&10);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_if_then_else_all_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| true).or_else(move |x: &i32| {
            l2.lock().unwrap().push(*x * 100);
        });
        conditional.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_if_then_else_all_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|_: &i32| false).or_else(move |x: &i32| {
            l2.lock().unwrap().push(*x * 100);
        });
        conditional.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![500]);
    }

    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        })
        .and_then(BoxStatefulConsumer::noop());
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_complex_chain() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let l4 = log.clone();
        let mut consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        })
        .and_then(BoxStatefulConsumer::noop())
        .and_then(move |x: &i32| {
            l3.lock().unwrap().push(*x + 10);
        })
        .and_then(move |x: &i32| {
            l4.lock().unwrap().push(*x - 5);
        });
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10, 15, 0]);
    }

    #[test]
    fn test_box_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut boxed = conditional.into_box();
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        boxed.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_box_conditional_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
        rc_consumer.accept(&-5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_box_conditional_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut func = conditional.into_fn();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        func(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_box_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut chained = conditional.and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        });
        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
        chained.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10, -10]);
    }

    #[test]
    fn test_arc_when() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let mut conditional = consumer.when(|x: &i32| *x > 0);
        conditional.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        conditional.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_conditional_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        clone2.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_arc_conditional_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut arc_consumer = conditional.into_arc();
        arc_consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        arc_consumer.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut box_consumer = conditional.into_box();
        box_consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        box_consumer.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_conditional_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        rc_consumer.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_conditional_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut func = conditional.into_fn();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        func(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_conditional_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut with_else = conditional.or_else(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        });
        with_else.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
        with_else.accept(&-5);
        assert_eq!(*log.lock().unwrap(), vec![5, -10]);
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
    fn test_rc_conditional_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
        rc_consumer.accept(&-5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_rc_conditional_into_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut box_consumer = conditional.into_box();
        box_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
        box_consumer.accept(&-5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_rc_conditional_into_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let conditional = consumer.when(|x: &i32| *x > 0);
        let mut func = conditional.into_fn();
        func(&5);
        assert_eq!(*log.borrow(), vec![5]);
        func(&-5);
        assert_eq!(*log.borrow(), vec![5]);
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
}

// ============================================================================
// Default Implementation Tests for into_xxx() methods
// ============================================================================

#[cfg(test)]
mod test_default_into_implementations {
    use super::*;

    // Define a custom Consumer implementation for testing default into_xxx() methods
    struct CustomConsumer {
        log: Arc<Mutex<Vec<i32>>>,
    }

    impl StatefulConsumer<i32> for CustomConsumer {
        fn accept(&mut self, value: &i32) {
            self.log.lock().unwrap().push(*value * 10);
        }

        // Do not implement into_box, into_rc, into_arc, into_fn
        // Use default implementations
    }

    #[test]
    fn test_custom_consumer_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        let mut box_consumer = custom.into_box();
        box_consumer.accept(&5);
        box_consumer.accept(&10);

        assert_eq!(*log.lock().unwrap(), vec![50, 100]);
    }

    #[test]
    fn test_custom_consumer_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        let mut rc_consumer = custom.into_rc();
        rc_consumer.accept(&3);
        rc_consumer.accept(&7);

        assert_eq!(*log.lock().unwrap(), vec![30, 70]);
    }

    #[test]
    fn test_custom_consumer_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        let mut arc_consumer = custom.into_arc();
        arc_consumer.accept(&2);
        arc_consumer.accept(&8);

        assert_eq!(*log.lock().unwrap(), vec![20, 80]);
    }

    #[test]
    fn test_custom_consumer_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        let mut func = custom.into_fn();
        func(&4);
        func(&6);

        assert_eq!(*log.lock().unwrap(), vec![40, 60]);
    }

    // Test custom Consumer composition with other Consumers
    #[test]
    fn test_custom_consumer_chaining_with_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let custom = CustomConsumer { log: l1 };
        let box_consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 1);
        });

        let mut chained = custom.into_box().and_then(box_consumer);
        chained.accept(&5);

        // custom: 5 * 10 = 50, box: 5 + 1 = 6
        assert_eq!(*log.lock().unwrap(), vec![50, 6]);
    }

    #[test]
    fn test_custom_consumer_cloneable() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        // Can clone after converting to RcConsumer
        let rc_consumer = custom.into_rc();
        let mut clone1 = rc_consumer.clone();
        let mut clone2 = rc_consumer.clone();

        clone1.accept(&1);
        clone2.accept(&2);

        let mut result = log.lock().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![10, 20]);
    }

    // Define a stateful custom Consumer
    struct StatefulConsumerImpl {
        log: Arc<Mutex<Vec<i32>>>,
        multiplier: i32,
    }

    impl StatefulConsumer<i32> for StatefulConsumerImpl {
        fn accept(&mut self, value: &i32) {
            self.log.lock().unwrap().push(*value * self.multiplier);
            self.multiplier += 1; // Increment multiplier after each call
        }
    }

    #[test]
    fn test_stateful_consumer_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let stateful = StatefulConsumerImpl {
            log: log.clone(),
            multiplier: 2,
        };

        let mut box_consumer = stateful.into_box();
        box_consumer.accept(&5); // 5 * 2 = 10
        box_consumer.accept(&5); // 5 * 3 = 15
        box_consumer.accept(&5); // 5 * 4 = 20

        assert_eq!(*log.lock().unwrap(), vec![10, 15, 20]);
    }

    #[test]
    fn test_stateful_consumer_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let stateful = StatefulConsumerImpl {
            log: log.clone(),
            multiplier: 3,
        };

        let mut rc_consumer = stateful.into_rc();
        rc_consumer.accept(&4); // 4 * 3 = 12
        rc_consumer.accept(&4); // 4 * 4 = 16

        assert_eq!(*log.lock().unwrap(), vec![12, 16]);
    }

    #[test]
    fn test_stateful_consumer_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let stateful = StatefulConsumerImpl {
            log: log.clone(),
            multiplier: 5,
        };

        let mut arc_consumer = stateful.into_arc();
        arc_consumer.accept(&2); // 2 * 5 = 10
        arc_consumer.accept(&2); // 2 * 6 = 12
        arc_consumer.accept(&2); // 2 * 7 = 14

        assert_eq!(*log.lock().unwrap(), vec![10, 12, 14]);
    }

    #[test]
    fn test_stateful_consumer_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let stateful = StatefulConsumerImpl {
            log: log.clone(),
            multiplier: 1,
        };

        let mut func = stateful.into_fn();
        func(&10); // 10 * 1 = 10
        func(&10); // 10 * 2 = 20
        func(&10); // 10 * 3 = 30

        assert_eq!(*log.lock().unwrap(), vec![10, 20, 30]);
    }

    // Test thread-safe custom Consumer
    #[test]
    fn test_custom_consumer_thread_safety() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        let arc_consumer = custom.into_arc();
        let mut c1 = arc_consumer.clone();
        let mut c2 = arc_consumer.clone();

        let h1 = std::thread::spawn(move || {
            c1.accept(&1);
        });

        let h2 = std::thread::spawn(move || {
            c2.accept(&2);
        });

        h1.join().unwrap();
        h2.join().unwrap();

        let mut result = log.lock().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![10, 20]);
    }
}

#[test]
fn test_arcconsumer_to_box_rc_arc_and_fn() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();

    let consumer = ArcStatefulConsumer::new(move |x: &i32| {
        l.lock().unwrap().push(*x + 1);
    });

    // to_box()
    let mut boxed = consumer.to_box();
    boxed.accept(&5);
    assert_eq!(*log.lock().unwrap(), vec![6]);

    // to_rc()
    let mut rc = consumer.to_rc();
    rc.accept(&7);
    assert_eq!(*log.lock().unwrap(), vec![6, 8]);

    // to_arc() returns clone
    let arc_clone = consumer.to_arc();
    let mut c = arc_clone;
    c.accept(&1);
    assert_eq!(*log.lock().unwrap(), vec![6, 8, 2]);

    // to_fn()
    let mut f = consumer.to_fn();
    f(&3);
    assert_eq!(*log.lock().unwrap(), vec![6, 8, 2, 4]);
}

#[test]
fn test_rcconsumer_to_box_rc_and_fn() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l = log.clone();

    let consumer = RcStatefulConsumer::new(move |x: &i32| {
        l.borrow_mut().push(*x + 2);
    });

    let mut boxed = consumer.to_box();
    boxed.accept(&4);
    assert_eq!(*log.borrow(), vec![6]);

    let mut rc2 = consumer.to_rc();
    rc2.accept(&5);
    assert_eq!(*log.borrow(), vec![6, 7]);

    let mut f = consumer.to_fn();
    f(&1);
    assert_eq!(*log.borrow(), vec![6, 7, 3]);
}

// ============================================================================
// Closure to_xxx Tests - Testing closure's Consumer trait implementation
// ============================================================================

#[cfg(test)]
mod test_closure_to_methods {
    use super::*;

    // Note: closures must implement Clone to use to_xxx methods
    // We need to use cloneable closures or wrapper types

    #[test]
    fn test_arc_consumer_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 3);
        });

        // Test to_box() - should preserve original consumer
        let mut boxed = consumer.to_box();
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![15]);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![15, 30]);
    }

    #[test]
    fn test_arc_consumer_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 4);
        });

        // Test to_rc() - should preserve original consumer
        let mut rc = consumer.to_rc();
        rc.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![20]);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&2);
        assert_eq!(*log.lock().unwrap(), vec![20, 8]);
    }

    #[test]
    fn test_arc_consumer_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 5);
        });

        // Test to_arc() - should preserve original consumer
        let mut arc = consumer.to_arc();
        arc.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![15]);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&4);
        assert_eq!(*log.lock().unwrap(), vec![15, 20]);
    }

    #[test]
    fn test_arc_consumer_to_fn_preserves_original() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 6);
        });

        // Test to_fn() - should preserve original consumer
        let mut func = consumer.to_fn();
        func(&2);
        assert_eq!(*log.lock().unwrap(), vec![12]);

        // Because to_fn() borrows the consumer, it needs to complete func usage first
        drop(func);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![12, 18]);
    }

    #[test]
    fn test_rc_consumer_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 7);
        });

        // Test to_box() - should preserve original consumer
        let mut boxed = consumer.to_box();
        boxed.accept(&2);
        assert_eq!(*log.borrow(), vec![14]);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&3);
        assert_eq!(*log.borrow(), vec![14, 21]);
    }

    #[test]
    fn test_rc_consumer_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 8);
        });

        // Test to_rc() - should preserve original consumer
        let mut rc = consumer.to_rc();
        rc.accept(&2);
        assert_eq!(*log.borrow(), vec![16]);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&1);
        assert_eq!(*log.borrow(), vec![16, 8]);
    }

    #[test]
    fn test_rc_consumer_to_fn_preserves_original() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcStatefulConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 9);
        });

        // Test to_fn() - should preserve original consumer
        let mut func = consumer.to_fn();
        func(&1);
        assert_eq!(*log.borrow(), vec![9]);

        // Because to_fn() borrows the consumer, it needs to complete func usage first
        drop(func);

        // Original consumer is still available
        let mut original = consumer;
        original.accept(&2);
        assert_eq!(*log.borrow(), vec![9, 18]);
    }

    #[test]
    fn test_custom_consumer_to_box() {
        struct CustomConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl StatefulConsumer<i32> for CustomConsumer {
            fn accept(&mut self, value: &i32) {
                self.log.lock().unwrap().push(*value * 10);
            }
        }

        impl Clone for CustomConsumer {
            fn clone(&self) -> Self {
                CustomConsumer {
                    log: self.log.clone(),
                }
            }
        }

        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        // Test to_box() - using default implementation
        let mut boxed = custom.to_box();
        boxed.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![30]);

        // Original custom consumer is still available
        let mut original = custom;
        original.accept(&4);
        assert_eq!(*log.lock().unwrap(), vec![30, 40]);
    }

    #[test]
    fn test_custom_consumer_to_rc() {
        struct CustomConsumer {
            log: Rc<RefCell<Vec<i32>>>,
        }

        impl StatefulConsumer<i32> for CustomConsumer {
            fn accept(&mut self, value: &i32) {
                self.log.borrow_mut().push(*value * 11);
            }
        }

        impl Clone for CustomConsumer {
            fn clone(&self) -> Self {
                CustomConsumer {
                    log: self.log.clone(),
                }
            }
        }

        let log = Rc::new(RefCell::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        // Test to_rc() - using default implementation
        let mut rc = custom.to_rc();
        rc.accept(&2);
        assert_eq!(*log.borrow(), vec![22]);

        // Original custom consumer is still available
        let mut original = custom;
        original.accept(&3);
        assert_eq!(*log.borrow(), vec![22, 33]);
    }

    #[test]
    fn test_custom_consumer_to_arc() {
        struct CustomConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl StatefulConsumer<i32> for CustomConsumer {
            fn accept(&mut self, value: &i32) {
                self.log.lock().unwrap().push(*value * 12);
            }
        }

        impl Clone for CustomConsumer {
            fn clone(&self) -> Self {
                CustomConsumer {
                    log: self.log.clone(),
                }
            }
        }

        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        // Test to_arc() - using default implementation
        let mut arc = custom.to_arc();
        arc.accept(&2);
        assert_eq!(*log.lock().unwrap(), vec![24]);

        // Original custom consumer is still available
        let mut original = custom;
        original.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![24, 36]);
    }

    #[test]
    fn test_custom_consumer_to_fn() {
        struct CustomConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl StatefulConsumer<i32> for CustomConsumer {
            fn accept(&mut self, value: &i32) {
                self.log.lock().unwrap().push(*value * 13);
            }
        }

        impl Clone for CustomConsumer {
            fn clone(&self) -> Self {
                CustomConsumer {
                    log: self.log.clone(),
                }
            }
        }

        let log = Arc::new(Mutex::new(Vec::new()));
        let custom = CustomConsumer { log: log.clone() };

        // Test to_fn() - using default implementation
        let mut func = custom.to_fn();
        func(&2);
        assert_eq!(*log.lock().unwrap(), vec![26]);

        // Because to_fn() borrows the custom, it needs to complete func usage first
        drop(func);

        // Original custom consumer is still available
        let mut original = custom;
        original.accept(&1);
        assert_eq!(*log.lock().unwrap(), vec![26, 13]);
    }

    // ============================================================================
    // BoxConsumer ConsumerOnce Tests
    // ============================================================================

    #[test]
    fn test_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        consumer.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_accept_once_with_different_types() {
        // String
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |s: &String| {
            *l.lock().unwrap() = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept_once(&text);
        assert_eq!(*log.lock().unwrap(), "Got: hello");

        // Vec
        let log = Arc::new(Mutex::new(0));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |v: &Vec<i32>| {
            *l.lock().unwrap() = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept_once(&numbers);
        assert_eq!(*log.lock().unwrap(), 3);

        // bool
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |b: &bool| {
            *l.lock().unwrap() = if *b { "true" } else { "false" }.to_string();
        });
        let flag = true;
        consumer.accept_once(&flag);
        assert_eq!(*log.lock().unwrap(), "true");
    }

    #[test]
    fn test_into_box_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let box_consumer_once = consumer.into_box_once();
        box_consumer_once.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_into_fn_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let func = consumer.into_fn_once();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_consumer_once_with_state_modification() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            counter += 1;
            l.lock().unwrap().push(*x + counter);
        });
        consumer.accept_once(&10);
        assert_eq!(*log.lock().unwrap(), vec![11]); // 10 + 1
    }

    #[test]
    fn test_consumer_once_consumes_self() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        // This should compile - accept_once consumes self
        consumer.accept_once(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);

        // This would not compile - consumer is moved
        // consumer.accept(&3); // Would not compile
    }
}

// ============================================================================
// ConsumerOnce Implementation Tests
// ============================================================================

#[cfg(test)]
mod consumer_once_compat_tests {
    use super::*;

    // Helper function that accepts ConsumerOnce
    fn accept_consumer_once<C: ConsumerOnce<i32>>(consumer: C, value: &i32) {
        consumer.accept_once(value);
    }

    #[test]
    fn test_box_consumer_as_consumer_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        // BoxConsumer can be used as ConsumerOnce
        accept_consumer_once(consumer, &42);
        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_arc_consumer_as_consumer_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        });

        // ArcConsumer can be used as ConsumerOnce
        accept_consumer_once(consumer, &21);
        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_rc_consumer_as_consumer_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x + 10);
        });

        // RcConsumer can be used as ConsumerOnce
        accept_consumer_once(consumer, &32);
        assert_eq!(*log.borrow(), vec![42]);
    }

    #[test]
    fn test_box_consumer_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        consumer.accept_once(&100);
        assert_eq!(*log.lock().unwrap(), vec![100]);
    }

    #[test]
    fn test_arc_consumer_accept_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 3);
        });

        consumer.accept_once(&7);
        assert_eq!(*log.lock().unwrap(), vec![21]);
    }

    #[test]
    fn test_rc_consumer_accept_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x - 5);
        });

        consumer.accept_once(&15);
        assert_eq!(*log.borrow(), vec![10]);
    }

    #[test]
    fn test_box_consumer_into_box_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        let once_consumer = consumer.into_box_once();
        once_consumer.accept_once(&77);
        assert_eq!(*log.lock().unwrap(), vec![77]);
    }

    #[test]
    fn test_arc_consumer_into_box_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 5);
        });

        let once_consumer = consumer.into_box_once();
        once_consumer.accept_once(&8);
        assert_eq!(*log.lock().unwrap(), vec![40]);
    }

    #[test]
    fn test_rc_consumer_into_box_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x + 20);
        });

        let once_consumer = consumer.into_box_once();
        once_consumer.accept_once(&22);
        assert_eq!(*log.borrow(), vec![42]);
    }

    #[test]
    fn test_box_consumer_into_fn_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });

        let func = consumer.into_fn_once();
        func(&99);
        assert_eq!(*log.lock().unwrap(), vec![99]);
    }

    #[test]
    fn test_arc_consumer_into_fn_once() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x / 2);
        });

        let func = consumer.into_fn_once();
        func(&84);
        assert_eq!(*log.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_rc_consumer_into_fn_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x - 8);
        });

        let func = consumer.into_fn_once();
        func(&50);
        assert_eq!(*log.borrow(), vec![42]);
    }
}

// ============================================================================
// Closure StatefulConsumer into_xxx Tests
// ============================================================================

#[cfg(test)]
mod test_closure_stateful_consumer_into_methods {
    use super::*;

    #[test]
    fn test_closure_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x);
        };
        let mut consumer = StatefulConsumer::into_box(closure);
        consumer.accept(&5);
        consumer.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_closure_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.borrow_mut().push(*x * 2);
        };
        let mut consumer = StatefulConsumer::into_rc(closure);
        consumer.accept(&5);
        consumer.accept(&10);
        assert_eq!(*log.borrow(), vec![10, 20]);
    }

    #[test]
    fn test_closure_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x + 1);
        };
        let mut consumer = StatefulConsumer::into_arc(closure);
        consumer.accept(&5);
        consumer.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![6, 11]);
    }

    #[test]
    fn test_closure_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x - 1);
        };
        let mut func = StatefulConsumer::into_fn(closure);
        func(&5);
        func(&10);
        assert_eq!(*log.lock().unwrap(), vec![4, 9]);
    }

    #[test]
    fn test_closure_into_box_with_state() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let closure = move |x: &i32| {
            counter += 1;
            l.lock().unwrap().push(counter + x);
        };
        let mut consumer = StatefulConsumer::into_box(closure);
        consumer.accept(&10);
        consumer.accept(&10);
        consumer.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![11, 12, 13]);
    }

    #[test]
    fn test_closure_into_rc_with_state() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut counter = 0;
        let closure = move |x: &i32| {
            counter += 1;
            l.borrow_mut().push(counter * x);
        };
        let mut consumer = StatefulConsumer::into_rc(closure);
        consumer.accept(&10);
        consumer.accept(&10);
        consumer.accept(&10);
        assert_eq!(*log.borrow(), vec![10, 20, 30]);
    }

    #[test]
    fn test_closure_into_arc_with_state() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut sum = 0;
        let closure = move |x: &i32| {
            sum += x;
            l.lock().unwrap().push(sum);
        };
        let mut consumer = StatefulConsumer::into_arc(closure);
        consumer.accept(&5);
        consumer.accept(&10);
        consumer.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![5, 15, 18]);
    }
}

// ============================================================================
// FnStatefulConsumerOps and_then Tests
// ============================================================================

#[cfg(test)]
mod test_fn_stateful_consumer_ops {
    use super::*;
    use prism3_function::FnStatefulConsumerOps;

    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let mut chained = FnStatefulConsumerOps::and_then(
            move |x: &i32| {
                l1.lock().unwrap().push(*x * 2);
            },
            move |x: &i32| {
                l2.lock().unwrap().push(*x + 10);
            },
        );

        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_closure_and_then_with_box_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let second = BoxStatefulConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });

        let mut chained = FnStatefulConsumerOps::and_then(
            move |x: &i32| {
                l1.lock().unwrap().push(*x * 2);
            },
            second,
        );

        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_closure_and_then_multiple() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();

        let first = move |x: &i32| {
            l1.lock().unwrap().push(*x);
        };
        let second = move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        };
        let third = BoxStatefulConsumer::new(move |x: &i32| {
            l3.lock().unwrap().push(*x + 100);
        });

        let chained = FnStatefulConsumerOps::and_then(first, second);
        let mut chained = chained.and_then(third);

        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10, 105]);
    }

    #[test]
    fn test_closure_and_then_with_arc_consumer() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let second = ArcStatefulConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x * 3);
        });

        let mut chained = FnStatefulConsumerOps::and_then(
            move |x: &i32| {
                l1.lock().unwrap().push(*x + 1);
            },
            second,
        );

        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![6, 15]);
    }

    #[test]
    fn test_closure_and_then_with_arc_consumer_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let second = ArcStatefulConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });

        // Clone second to preserve it
        let mut chained = FnStatefulConsumerOps::and_then(
            move |x: &i32| {
                l1.lock().unwrap().push(*x * 2);
            },
            second.clone(),
        );

        chained.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);

        // Original second still usable
        let mut second_copy = second;
        second_copy.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![10, 15, 13]);
    }
}
