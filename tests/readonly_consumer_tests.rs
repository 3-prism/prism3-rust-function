/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Tests for ReadonlyConsumer types

use prism3_function::{
    ArcReadonlyConsumer, BoxReadonlyConsumer, FnReadonlyConsumerOps, RcReadonlyConsumer,
    ReadonlyConsumer,
};
use std::rc::Rc;
use std::sync::Arc;

#[cfg(test)]
mod box_readonly_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let consumer = BoxReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        consumer.accept(&5);
    }

    #[test]
    fn test_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let chained = BoxReadonlyConsumer::new(move |_x: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32| {
            *c2.lock().unwrap() += 1;
        });

        chained.accept(&5);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_and_then_with_box_consumer() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let first = BoxReadonlyConsumer::new(move |_x: &i32| {
            *c1.lock().unwrap() += 1;
        });

        let second = BoxReadonlyConsumer::new(move |_x: &i32| {
            *c2.lock().unwrap() += 1;
        });

        let chained = first.and_then(second);
        chained.accept(&5);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let chained = BoxReadonlyConsumer::new(move |_x: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32| {
            *c2.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32| {
            *c3.lock().unwrap() += 1;
        });

        chained.accept(&5);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_noop() {
        let noop = BoxReadonlyConsumer::<i32>::noop();
        noop.accept(&42);
        // Should not panic
    }

    #[test]
    fn test_into_box() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        let box_consumer = closure.into_box();
        box_consumer.accept(&5);
    }

    #[test]
    fn test_into_rc() {
        let consumer = BoxReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5);
    }

    #[test]
    fn test_into_fn() {
        let consumer = BoxReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let func = consumer.into_fn();
        func(&5);
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxReadonlyConsumer::<i32>::noop();
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_debug() {
        let consumer = BoxReadonlyConsumer::<i32>::noop();
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxReadonlyConsumer"));
    }

    #[test]
    fn test_display() {
        let mut consumer = BoxReadonlyConsumer::<i32>::noop();
        assert_eq!(format!("{}", consumer), "BoxReadonlyConsumer");

        consumer.set_name("my_consumer");
        assert_eq!(format!("{}", consumer), "BoxReadonlyConsumer(my_consumer)");
    }

    #[test]
    fn test_with_different_types() {
        let string_consumer = BoxReadonlyConsumer::new(|s: &String| {
            println!("String: {}", s);
        });
        string_consumer.accept(&"Hello".to_string());

        let vec_consumer = BoxReadonlyConsumer::new(|v: &Vec<i32>| {
            println!("Vec length: {}", v.len());
        });
        vec_consumer.accept(&vec![1, 2, 3]);
    }
}

#[cfg(test)]
mod arc_readonly_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        consumer.accept(&5);
    }

    #[test]
    fn test_clone() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let consumer = ArcReadonlyConsumer::new(move |_x: &i32| {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let clone = consumer.clone();
        consumer.accept(&5);
        clone.accept(&10);

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_and_then() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let first = ArcReadonlyConsumer::new(move |_x: &i32| {
            c1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let second = ArcReadonlyConsumer::new(move |_x: &i32| {
            c2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let chained = first.and_then(&second);
        chained.accept(&5);

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);

        // Original consumers remain usable
        first.accept(&10);
        second.accept(&15);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 4);
    }

    #[test]
    fn test_into_box() {
        let consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let box_consumer = consumer.into_box();
        box_consumer.accept(&5);
    }

    #[test]
    fn test_into_rc() {
        let consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5);
    }

    #[test]
    fn test_into_arc() {
        let consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let arc_consumer = consumer.into_arc();
        arc_consumer.accept(&5);
    }

    #[test]
    fn test_into_fn() {
        let consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let func = consumer.into_fn();
        func(&5);
    }

    #[test]
    fn test_to_fn() {
        let consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let func = consumer.to_fn();
        func(&5);

        // Original consumer remains usable
        consumer.accept(&10);
    }

    #[test]
    fn test_name() {
        let mut consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_debug() {
        let consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcReadonlyConsumer"));
    }

    #[test]
    fn test_display() {
        let mut consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        assert_eq!(format!("{}", consumer), "ArcReadonlyConsumer");

        consumer.set_name("my_consumer");
        assert_eq!(format!("{}", consumer), "ArcReadonlyConsumer(my_consumer)");
    }

    #[test]
    fn test_thread_safety() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let consumer = ArcReadonlyConsumer::new(move |_x: &i32| {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let consumer_clone = consumer.clone();
                std::thread::spawn(move || {
                    consumer_clone.accept(&i);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 10);
    }
}

#[cfg(test)]
mod rc_readonly_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        consumer.accept(&5);
    }

    #[test]
    fn test_clone() {
        let counter = Rc::new(std::cell::RefCell::new(0));
        let c = counter.clone();
        let consumer = RcReadonlyConsumer::new(move |_x: &i32| {
            *c.borrow_mut() += 1;
        });

        let clone = consumer.clone();
        consumer.accept(&5);
        clone.accept(&10);

        assert_eq!(*counter.borrow(), 2);
    }

    #[test]
    fn test_and_then() {
        let counter = Rc::new(std::cell::RefCell::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let first = RcReadonlyConsumer::new(move |_x: &i32| {
            *c1.borrow_mut() += 1;
        });

        let second = RcReadonlyConsumer::new(move |_x: &i32| {
            *c2.borrow_mut() += 1;
        });

        let chained = first.and_then(&second);
        chained.accept(&5);

        assert_eq!(*counter.borrow(), 2);

        // Original consumers remain usable
        first.accept(&10);
        second.accept(&15);
        assert_eq!(*counter.borrow(), 4);
    }

    #[test]
    fn test_into_box() {
        let consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let box_consumer = consumer.into_box();
        box_consumer.accept(&5);
    }

    #[test]
    fn test_into_rc() {
        let consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5);
    }

    #[test]
    fn test_into_fn() {
        let consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let func = consumer.into_fn();
        func(&5);
    }

    #[test]
    fn test_to_fn() {
        let consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let func = consumer.to_fn();
        func(&5);

        // Original consumer remains usable
        consumer.accept(&10);
    }

    #[test]
    fn test_name() {
        let mut consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_debug() {
        let consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcReadonlyConsumer"));
    }

    #[test]
    fn test_display() {
        let mut consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        assert_eq!(format!("{}", consumer), "RcReadonlyConsumer");

        consumer.set_name("my_consumer");
        assert_eq!(format!("{}", consumer), "RcReadonlyConsumer(my_consumer)");
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        closure.accept(&5);
    }

    #[test]
    fn test_closure_into_box() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        let box_consumer = closure.into_box();
        box_consumer.accept(&5);
    }

    #[test]
    fn test_closure_into_rc() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        let rc_consumer = closure.into_rc();
        rc_consumer.accept(&5);
    }

    #[test]
    fn test_closure_into_arc() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        let arc_consumer = closure.into_arc();
        arc_consumer.accept(&5);
    }

    #[test]
    fn test_closure_into_fn() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        let func = closure.into_fn();
        func(&5);
    }

    #[test]
    fn test_closure_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let chained = (move |_x: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32| {
            *c2.lock().unwrap() += 1;
        });

        chained.accept(&5);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_closure_and_then_multiple() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let chained = (move |_x: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32| {
            *c2.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32| {
            *c3.lock().unwrap() += 1;
        });

        chained.accept(&5);
        assert_eq!(*counter.lock().unwrap(), 3);
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_box_to_rc() {
        let box_consumer = BoxReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let rc_consumer = box_consumer.into_rc();
        rc_consumer.accept(&5);
    }

    #[test]
    fn test_arc_to_box() {
        let arc_consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let box_consumer = arc_consumer.into_box();
        box_consumer.accept(&5);
    }

    #[test]
    fn test_arc_to_rc() {
        let arc_consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let rc_consumer = arc_consumer.into_rc();
        rc_consumer.accept(&5);
    }

    #[test]
    fn test_rc_to_box() {
        let rc_consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        let box_consumer = rc_consumer.into_box();
        box_consumer.accept(&5);
    }

    // Note: Box and Rc cannot be converted to Arc because they don't implement Send+Sync
    // These conversions are prevented at compile time, not runtime
}

#[cfg(test)]
mod generic_tests {
    use super::*;

    fn apply_consumer<C: ReadonlyConsumer<i32>>(consumer: &C, value: &i32) {
        consumer.accept(value);
    }

    #[test]
    fn test_with_box_consumer() {
        let box_consumer = BoxReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        apply_consumer(&box_consumer, &5);
    }

    #[test]
    fn test_with_arc_consumer() {
        let arc_consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        apply_consumer(&arc_consumer, &5);
    }

    #[test]
    fn test_with_rc_consumer() {
        let rc_consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        apply_consumer(&rc_consumer, &5);
    }

    #[test]
    fn test_with_closure() {
        let closure = |x: &i32| {
            println!("Value: {}", x);
        };
        apply_consumer(&closure, &5);
    }
}

// ============================================================================
// Name Tests - Testing name() and set_name() methods
// ============================================================================

#[cfg(test)]
mod name_tests {
    use super::*;

    #[test]
    fn test_box_consumer_name() {
        let mut consumer = BoxReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        assert_eq!(consumer.name(), None);

        consumer.set_name("printer");
        assert_eq!(consumer.name(), Some("printer"));
    }

    #[test]
    fn test_arc_consumer_name() {
        let mut consumer = ArcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        assert_eq!(consumer.name(), None);

        consumer.set_name("printer");
        assert_eq!(consumer.name(), Some("printer"));
    }

    #[test]
    fn test_rc_consumer_name() {
        let mut consumer = RcReadonlyConsumer::new(|x: &i32| {
            println!("Value: {}", x);
        });
        assert_eq!(consumer.name(), None);

        consumer.set_name("printer");
        assert_eq!(consumer.name(), Some("printer"));
    }

    #[test]
    fn test_box_consumer_name_with_accept() {
        let mut consumer = BoxReadonlyConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&1);
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_arc_consumer_name_with_accept() {
        let mut consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&1);
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_rc_consumer_name_with_accept() {
        let mut consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&1);
        assert_eq!(consumer.name(), Some("test_consumer"));
    }
}

// ============================================================================
// Display and Debug Tests
// ============================================================================

#[cfg(test)]
mod display_debug_tests {
    use super::*;

    #[test]
    fn test_box_consumer_debug() {
        let consumer = BoxReadonlyConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxReadonlyConsumer"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_box_consumer_display_without_name() {
        let consumer = BoxReadonlyConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxReadonlyConsumer");
    }

    #[test]
    fn test_box_consumer_display_with_name() {
        let mut consumer = BoxReadonlyConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxReadonlyConsumer(test_consumer)");
    }

    #[test]
    fn test_arc_consumer_debug() {
        let consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcReadonlyConsumer"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_arc_consumer_display_without_name() {
        let consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcReadonlyConsumer");
    }

    #[test]
    fn test_arc_consumer_display_with_name() {
        let mut consumer = ArcReadonlyConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcReadonlyConsumer(test_consumer)");
    }

    #[test]
    fn test_rc_consumer_debug() {
        let consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcReadonlyConsumer"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_rc_consumer_display_without_name() {
        let consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyConsumer");
    }

    #[test]
    fn test_rc_consumer_display_with_name() {
        let mut consumer = RcReadonlyConsumer::new(|_x: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyConsumer(test_consumer)");
    }
}
