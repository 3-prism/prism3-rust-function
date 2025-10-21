/// Tests for ReadonlyBiConsumer types
use prism3_function::{
    ArcReadonlyBiConsumer, BoxReadonlyBiConsumer, FnReadonlyBiConsumerOps, RcReadonlyBiConsumer,
    ReadonlyBiConsumer,
};
use std::cell::RefCell;
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
    fn test_box_into_box() {
        let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxReadonlyBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_debug() {
        let consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxReadonlyBiConsumer"));
    }

    #[test]
    fn test_display() {
        let consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxReadonlyBiConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxReadonlyBiConsumer(my_consumer)");
    }

    #[test]
    fn test_into_rc() {
        let consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
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

    #[test]
    fn test_debug() {
        let consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcReadonlyBiConsumer"));
    }

    #[test]
    fn test_display() {
        let consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcReadonlyBiConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcReadonlyBiConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let func = consumer.into_fn();
        func(&5, &3);
    }

    #[test]
    fn test_arc_into_fn_with_state() {
        use std::sync::Mutex;
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let func = consumer.into_fn();
        func(&5, &3);
        func(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    #[test]
    fn test_into_arc() {
        let consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let arc_consumer = consumer.into_arc();
        arc_consumer.accept(&5, &3);
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

    #[test]
    fn test_debug() {
        let consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcReadonlyBiConsumer"));
    }

    #[test]
    fn test_display() {
        let consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyBiConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let mut consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyBiConsumer(test_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let func = consumer.into_fn();
        func(&5, &3);
    }

    #[test]
    fn test_into_rc() {
        let consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
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

    #[test]
    fn test_closure_into_fn() {
        // Test into_fn in impl<T, U, F> ReadonlyBiConsumer<T, U> for F
        let closure = |x: &i32, y: &i32| {
            println!("Sum: {}", x + y);
        };
        let func = closure.into_fn();
        func(&5, &3);
    }

    #[test]
    fn test_closure_into_conversions_default_impls() {
        // Create a closure that increments an Arc mutex counter
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        };

        // Use default into_box
        let box_consumer = closure.into_box();
        box_consumer.accept(&5, &3);

        // Use default into_rc by creating a new closure (Rc requires non-Send closures)
        let counter2 = Rc::new(std::cell::RefCell::new(0));
        let c2 = counter2.clone();
        let closure2 = move |x: &i32, y: &i32| {
            *c2.borrow_mut() += x + y;
        };
        let rc_consumer = closure2.into_rc();
        rc_consumer.accept(&2, &3);

        // Test into_arc (requires Send + Sync closure)
        let counter3 = Arc::new(std::sync::Mutex::new(0));
        let c3 = counter3.clone();
        let closure3 = move |x: &i32, y: &i32| {
            *c3.lock().unwrap() += x + y;
        };
        let arc_consumer = closure3.into_arc();
        arc_consumer.accept(&1, &1);

        // Verify increments
        assert_eq!(*counter.lock().unwrap(), 8); // 5+3
        assert_eq!(*counter2.borrow(), 5); // 2+3
        assert_eq!(*counter3.lock().unwrap(), 2); // 1+1
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_noop_multiple_calls() {
        let consumer = BoxReadonlyBiConsumer::<i32, i32>::noop();
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        consumer.accept(&1, &2);
        // Should do nothing
    }

    #[test]
    fn test_and_then_with_noop() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let consumer = BoxReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            *c.lock().unwrap() += 1;
        })
        .and_then(BoxReadonlyBiConsumer::noop());
        consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_complex_chain() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();
        let consumer = BoxReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            *c1.lock().unwrap() += 1;
        })
        .and_then(move |_x: &i32, _y: &i32| {
            *c2.lock().unwrap() += 1;
        })
        .and_then(BoxReadonlyBiConsumer::noop())
        .and_then(move |_x: &i32, _y: &i32| {
            *c3.lock().unwrap() += 1;
        });
        consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_with_different_types() {
        let counter = Arc::new(std::sync::Mutex::new(String::new()));
        let c = counter.clone();
        let consumer = BoxReadonlyBiConsumer::new(move |s: &String, n: &i32| {
            *c.lock().unwrap() = format!("{}: {}", s, n);
        });
        consumer.accept(&"Count".to_string(), &42);
        assert_eq!(*counter.lock().unwrap(), "Count: 42");
    }

    #[test]
    fn test_arc_consumer_multiple_threads() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cons = consumer.clone();
                std::thread::spawn(move || {
                    cons.accept(&i, &1);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Sum of (0+1) + (1+1) + ... + (9+1) = 55
        assert_eq!(*counter.lock().unwrap(), 55);
    }

    #[test]
    fn test_rc_consumer_multiple_clones() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });

        let cons1 = consumer.clone();
        let cons2 = consumer.clone();
        let cons3 = consumer.clone();

        cons1.accept(&1, &2);
        cons2.accept(&3, &4);
        cons3.accept(&5, &6);

        assert_eq!(*counter.borrow(), 21); // 3 + 7 + 11
    }

    #[test]
    fn test_name_with_and_then() {
        let mut consumer1 = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer1.set_name("first");
        let consumer2 = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let chained = consumer1.and_then(consumer2);
        // Name is not preserved through and_then
        assert_eq!(chained.name(), None);
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });
        let func = consumer.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*counter.lock().unwrap(), 21); // 3 + 7 + 11
    }

    #[test]
    fn test_rc_to_fn_multiple_calls() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });
        let func = consumer.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*counter.borrow(), 21); // 3 + 7 + 11
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_arc_to_box() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });
        let box_consumer = arc_consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    #[test]
    fn test_arc_to_rc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });
        let rc_consumer = arc_consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    #[test]
    fn test_rc_to_box() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let rc_consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });
        let box_consumer = rc_consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.borrow(), 8);
    }

    #[test]
    fn test_closure_to_box() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        };
        let box_consumer = closure.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    #[test]
    fn test_closure_to_arc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        };
        let arc_consumer = closure.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    #[test]
    fn test_closure_to_rc() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        };
        let rc_consumer = closure.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.borrow(), 8);
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
        let mut consumer = BoxReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("{} + {} = {}", x, y, x + y);
        });
        assert_eq!(consumer.name(), None);

        consumer.set_name("add_printer");
        assert_eq!(consumer.name(), Some("add_printer"));
    }

    #[test]
    fn test_arc_consumer_name() {
        let mut consumer = ArcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("{} + {} = {}", x, y, x + y);
        });
        assert_eq!(consumer.name(), None);

        consumer.set_name("add_printer");
        assert_eq!(consumer.name(), Some("add_printer"));
    }

    #[test]
    fn test_rc_consumer_name() {
        let mut consumer = RcReadonlyBiConsumer::new(|x: &i32, y: &i32| {
            println!("{} + {} = {}", x, y, x + y);
        });
        assert_eq!(consumer.name(), None);

        consumer.set_name("add_printer");
        assert_eq!(consumer.name(), Some("add_printer"));
    }

    #[test]
    fn test_box_consumer_name_with_accept() {
        let mut consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&1, &2);
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_arc_consumer_name_with_accept() {
        let mut consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&1, &2);
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_rc_consumer_name_with_accept() {
        let mut consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&1, &2);
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_box_consumer_name_change() {
        let mut consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("name1");
        assert_eq!(consumer.name(), Some("name1"));
        consumer.set_name("name2");
        assert_eq!(consumer.name(), Some("name2"));
    }

    #[test]
    fn test_arc_consumer_name_change() {
        let mut consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("name1");
        assert_eq!(consumer.name(), Some("name1"));
        consumer.set_name("name2");
        assert_eq!(consumer.name(), Some("name2"));
    }

    #[test]
    fn test_rc_consumer_name_change() {
        let mut consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("name1");
        assert_eq!(consumer.name(), Some("name1"));
        consumer.set_name("name2");
        assert_eq!(consumer.name(), Some("name2"));
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
        let consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxReadonlyBiConsumer"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_box_consumer_display_without_name() {
        let consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxReadonlyBiConsumer");
    }

    #[test]
    fn test_box_consumer_display_with_name() {
        let mut consumer = BoxReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxReadonlyBiConsumer(test_consumer)");
    }

    #[test]
    fn test_arc_consumer_debug() {
        let consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcReadonlyBiConsumer"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_arc_consumer_display_without_name() {
        let consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcReadonlyBiConsumer");
    }

    #[test]
    fn test_arc_consumer_display_with_name() {
        let mut consumer = ArcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcReadonlyBiConsumer(test_consumer)");
    }

    #[test]
    fn test_rc_consumer_debug() {
        let consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcReadonlyBiConsumer"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_rc_consumer_display_without_name() {
        let consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyBiConsumer");
    }

    #[test]
    fn test_rc_consumer_display_with_name() {
        let mut consumer = RcReadonlyBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("test_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyBiConsumer(test_consumer)");
    }
}

// ============================================================================
// Custom ReadonlyBiConsumer Implementation Tests - Testing default into_xxx methods
// ============================================================================

#[cfg(test)]
mod custom_readonly_bi_consumer_tests {
    use super::*;

    /// Custom ReadonlyBiConsumer implementation for testing trait's default methods
    struct CustomReadonlyBiConsumer<T, U> {
        counter: Arc<std::sync::Mutex<i32>>,
        _phantom: std::marker::PhantomData<(T, U)>,
    }

    impl<T, U> CustomReadonlyBiConsumer<T, U> {
        fn new(counter: Arc<std::sync::Mutex<i32>>) -> Self {
            Self {
                counter,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    impl<T, U> ReadonlyBiConsumer<T, U> for CustomReadonlyBiConsumer<T, U> {
        fn accept(&self, _first: &T, _second: &U) {
            *self.counter.lock().unwrap() += 1;
        }
        // Use default into_xxx implementations from the trait
    }

    #[test]
    fn test_custom_into_box() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Test default into_box implementation
        let box_consumer = custom.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 1);

        box_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_custom_into_rc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Test default into_rc implementation
        let rc_consumer = custom.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 1);

        // Test RcReadonlyBiConsumer's clone
        let rc_clone = rc_consumer.clone();
        rc_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 2);

        rc_clone.accept(&15, &25);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_custom_into_arc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Test default into_arc implementation (requires Send + Sync)
        let arc_consumer = custom.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 1);

        // Test ArcReadonlyBiConsumer's clone
        let arc_clone = arc_consumer.clone();
        arc_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 2);

        arc_clone.accept(&15, &25);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_custom_into_fn() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Test default into_fn implementation
        let func = custom.into_fn();
        func(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 1);

        func(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 2);

        func(&15, &25);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_custom_into_box_then_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Convert to BoxReadonlyBiConsumer and test and_then
        let box_consumer = custom.into_box();
        let c2 = counter.clone();
        let chained = box_consumer.and_then(move |_: &i32, _: &i32| {
            *c2.lock().unwrap() += 10;
        });

        chained.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 11); // 1 + 10
    }

    #[test]
    fn test_custom_into_rc_then_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Convert to RcReadonlyBiConsumer and test and_then
        let rc_consumer = custom.into_rc();
        let c2 = counter.clone();
        let second = RcReadonlyBiConsumer::new(move |_: &i32, _: &i32| {
            *c2.lock().unwrap() += 10;
        });

        let chained = rc_consumer.and_then(&second);

        chained.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 11); // 1 + 10
    }

    #[test]
    fn test_custom_into_arc_then_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Convert to ArcReadonlyBiConsumer and test and_then
        let arc_consumer = custom.into_arc();
        let c2 = counter.clone();
        let second = ArcReadonlyBiConsumer::new(move |_: &i32, _: &i32| {
            *c2.lock().unwrap() += 10;
        });

        let chained = arc_consumer.and_then(&second);

        chained.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 11); // 1 + 10
    }

    #[test]
    fn test_custom_multiple_conversions() {
        // Test that the same custom implementation can be converted to different types
        let counter1 = Arc::new(std::sync::Mutex::new(0));
        let custom1 = CustomReadonlyBiConsumer::new(counter1.clone());
        let box_consumer = custom1.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter1.lock().unwrap(), 1);

        let counter2 = Arc::new(std::sync::Mutex::new(0));
        let custom2 = CustomReadonlyBiConsumer::new(counter2.clone());
        let rc_consumer = custom2.into_rc();
        rc_consumer.accept(&10, &20);
        assert_eq!(*counter2.lock().unwrap(), 1);

        let counter3 = Arc::new(std::sync::Mutex::new(0));
        let custom3 = CustomReadonlyBiConsumer::new(counter3.clone());
        let arc_consumer = custom3.into_arc();
        arc_consumer.accept(&15, &25);
        assert_eq!(*counter3.lock().unwrap(), 1);
    }

    #[test]
    fn test_custom_with_different_types() {
        // Test custom implementation with different parameter types
        let counter = Arc::new(std::sync::Mutex::new(0));

        struct StringIntConsumer {
            counter: Arc<std::sync::Mutex<i32>>,
        }

        impl ReadonlyBiConsumer<String, i32> for StringIntConsumer {
            fn accept(&self, _first: &String, second: &i32) {
                *self.counter.lock().unwrap() += second;
            }
        }

        let custom = StringIntConsumer {
            counter: counter.clone(),
        };

        let box_consumer = custom.into_box();
        box_consumer.accept(&"test".to_string(), &5);
        assert_eq!(*counter.lock().unwrap(), 5);

        box_consumer.accept(&"hello".to_string(), &10);
        assert_eq!(*counter.lock().unwrap(), 15);
    }

    #[test]
    fn test_custom_into_fn_with_state() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());

        // Convert to function and call multiple times
        let func = custom.into_fn();

        // Simulate usage in different contexts
        let simulate_usage = |f: &dyn Fn(&i32, &i32)| {
            f(&1, &2);
            f(&3, &4);
        };

        simulate_usage(&func);
        assert_eq!(*counter.lock().unwrap(), 2);

        func(&5, &6);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_custom_arc_send_sync() {
        // Test thread safety after converting custom implementation to Arc
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomReadonlyBiConsumer::new(counter.clone());
        let arc_consumer = custom.into_arc();

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let cons = arc_consumer.clone();
                std::thread::spawn(move || {
                    cons.accept(&1, &2);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(*counter.lock().unwrap(), 5);
    }
}

#[cfg(test)]
mod noop_tests {
    use super::*;

    #[test]
    fn test_box_noop_multiple_accepts() {
        let noop = BoxReadonlyBiConsumer::<i32, i32>::noop();
        noop.accept(&1, &2);
        noop.accept(&3, &4);
        noop.accept(&5, &6);
        // Should not panic and do nothing
    }

    #[test]
    fn test_arc_noop_multiple_accepts() {
        let noop = ArcReadonlyBiConsumer::<i32, i32>::noop();
        noop.accept(&1, &2);
        noop.accept(&3, &4);
        noop.accept(&5, &6);
        // Should not panic and do nothing
    }

    #[test]
    fn test_rc_noop_multiple_accepts() {
        let noop = RcReadonlyBiConsumer::<i32, i32>::noop();
        noop.accept(&1, &2);
        noop.accept(&3, &4);
        noop.accept(&5, &6);
        // Should not panic and do nothing
    }

    #[test]
    fn test_box_noop_with_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let active = BoxReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            *c.lock().unwrap() += 1;
        });
        let chained = active.and_then(BoxReadonlyBiConsumer::noop());
        chained.accept(&1, &2);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_arc_noop_with_and_then() {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let active = ArcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let noop = ArcReadonlyBiConsumer::<i32, i32>::noop();
        let chained = active.and_then(&noop);
        chained.accept(&1, &2);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_rc_noop_with_and_then() {
        let counter = Rc::new(std::cell::Cell::new(0));
        let c = counter.clone();
        let active = RcReadonlyBiConsumer::new(move |_x: &i32, _y: &i32| {
            c.set(c.get() + 1);
        });
        let chained = active.and_then(&RcReadonlyBiConsumer::<i32, i32>::noop());
        chained.accept(&1, &2);
        assert_eq!(counter.get(), 1);
    }
}

// ============================================================================
// to_xxx Methods Tests - Testing non-consuming conversion methods
// ============================================================================

#[cfg(test)]
mod to_methods_tests {
    use super::*;

    // ========================================================================
    // ArcReadonlyBiConsumer to_xxx tests
    // ========================================================================

    #[test]
    fn test_arc_to_box() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });

        let box_consumer = arc_consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original arc_consumer is still usable
        arc_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_arc_to_rc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });

        let rc_consumer = arc_consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original arc_consumer is still usable
        arc_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_arc_to_arc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });

        let arc_consumer2 = arc_consumer.to_arc();
        arc_consumer2.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original arc_consumer is still usable
        arc_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_arc_to_fn_preserves_original() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });

        let func = arc_consumer.to_fn();
        func(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original arc_consumer is still usable
        arc_consumer.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let arc_consumer = ArcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        });

        let func = arc_consumer.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*counter.lock().unwrap(), 21); // 3 + 7 + 11
    }

    // ========================================================================
    // RcReadonlyBiConsumer to_xxx tests
    // ========================================================================

    #[test]
    fn test_rc_to_box() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let rc_consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });

        let box_consumer = rc_consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.borrow(), 8);

        // Original rc_consumer is still usable
        rc_consumer.accept(&10, &20);
        assert_eq!(*counter.borrow(), 38); // 8 + 30
    }

    #[test]
    fn test_rc_to_rc() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let rc_consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });

        let rc_consumer2 = rc_consumer.to_rc();
        rc_consumer2.accept(&5, &3);
        assert_eq!(*counter.borrow(), 8);

        // Original rc_consumer is still usable
        rc_consumer.accept(&10, &20);
        assert_eq!(*counter.borrow(), 38); // 8 + 30
    }

    #[test]
    fn test_rc_to_fn_preserves_original() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let rc_consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });

        let func = rc_consumer.to_fn();
        func(&5, &3);
        assert_eq!(*counter.borrow(), 8);

        // Original rc_consumer is still usable
        rc_consumer.accept(&10, &20);
        assert_eq!(*counter.borrow(), 38); // 8 + 30
    }

    #[test]
    fn test_rc_to_fn_multiple_calls() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let rc_consumer = RcReadonlyBiConsumer::new(move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        });

        let func = rc_consumer.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*counter.borrow(), 21); // 3 + 7 + 11
    }

    // ========================================================================
    // Closure to_xxx tests
    // ========================================================================

    #[test]
    fn test_closure_to_box() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        };

        let box_consumer = closure.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original closure is still usable
        closure(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_closure_to_rc() {
        let counter = Rc::new(RefCell::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.borrow_mut() += x + y;
        };

        let rc_consumer = closure.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.borrow(), 8);

        // Original closure is still usable
        closure(&10, &20);
        assert_eq!(*counter.borrow(), 38); // 8 + 30
    }

    #[test]
    fn test_closure_to_arc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        };

        let arc_consumer = closure.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original closure is still usable
        closure(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_closure_to_fn() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let c = counter.clone();
        let closure = move |x: &i32, y: &i32| {
            *c.lock().unwrap() += x + y;
        };

        let func = closure.to_fn();
        func(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original closure is still usable
        closure(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    // ========================================================================
    // Custom ReadonlyBiConsumer to_xxx tests
    // ========================================================================

    /// Custom ReadonlyBiConsumer implementation for testing default to_xxx methods
    #[derive(Clone)]
    struct CustomConsumer {
        counter: Arc<std::sync::Mutex<i32>>,
    }

    impl CustomConsumer {
        fn new(counter: Arc<std::sync::Mutex<i32>>) -> Self {
            Self { counter }
        }
    }

    impl ReadonlyBiConsumer<i32, i32> for CustomConsumer {
        fn accept(&self, first: &i32, second: &i32) {
            *self.counter.lock().unwrap() += first + second;
        }
        // Use default to_xxx implementations from the trait
    }

    unsafe impl Send for CustomConsumer {}
    unsafe impl Sync for CustomConsumer {}

    #[test]
    fn test_custom_to_box() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let box_consumer = custom.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original custom is still usable
        custom.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_custom_to_rc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let rc_consumer = custom.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original custom is still usable
        custom.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_custom_to_arc() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let arc_consumer = custom.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original custom is still usable
        custom.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_custom_to_fn() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let func = custom.to_fn();
        func(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        // Original custom is still usable
        custom.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30
    }

    #[test]
    fn test_custom_to_fn_multiple_calls() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let func = custom.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*counter.lock().unwrap(), 21); // 3 + 7 + 11

        // Original custom is still usable
        custom.accept(&10, &10);
        assert_eq!(*counter.lock().unwrap(), 41); // 21 + 20
    }

    #[test]
    fn test_custom_to_box_then_and_then() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let box_consumer = custom.to_box();
        let c2 = counter.clone();
        let chained = box_consumer.and_then(move |x: &i32, y: &i32| {
            *c2.lock().unwrap() += x * y;
        });

        chained.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 23); // (5 + 3) + (5 * 3) = 8 + 15

        // Original custom is still usable
        custom.accept(&2, &2);
        assert_eq!(*counter.lock().unwrap(), 27); // 23 + 4
    }

    #[test]
    fn test_custom_to_rc_then_clone() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let rc_consumer = custom.to_rc();
        let rc_clone = rc_consumer.clone();

        rc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        rc_clone.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30

        // Original custom is still usable
        custom.accept(&1, &1);
        assert_eq!(*counter.lock().unwrap(), 40); // 38 + 2
    }

    #[test]
    fn test_custom_to_arc_then_clone() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let arc_consumer = custom.to_arc();
        let arc_clone = arc_consumer.clone();

        arc_consumer.accept(&5, &3);
        assert_eq!(*counter.lock().unwrap(), 8);

        arc_clone.accept(&10, &20);
        assert_eq!(*counter.lock().unwrap(), 38); // 8 + 30

        // Original custom is still usable
        custom.accept(&1, &1);
        assert_eq!(*counter.lock().unwrap(), 40); // 38 + 2
    }

    #[test]
    fn test_custom_to_arc_thread_safety() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        let arc_consumer = custom.to_arc();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cons = arc_consumer.clone();
                std::thread::spawn(move || {
                    cons.accept(&i, &1);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Sum of (0+1) + (1+1) + ... + (9+1) = 55
        assert_eq!(*counter.lock().unwrap(), 55);

        // Original custom is still usable
        custom.accept(&5, &5);
        assert_eq!(*counter.lock().unwrap(), 65); // 55 + 10
    }

    #[test]
    fn test_custom_multiple_to_conversions() {
        let counter = Arc::new(std::sync::Mutex::new(0));
        let custom = CustomConsumer::new(counter.clone());

        // Test multiple conversions from the same custom instance
        let box_consumer = custom.to_box();
        let rc_consumer = custom.to_rc();
        let arc_consumer = custom.to_arc();
        let func = custom.to_fn();

        box_consumer.accept(&1, &1);
        assert_eq!(*counter.lock().unwrap(), 2);

        rc_consumer.accept(&2, &2);
        assert_eq!(*counter.lock().unwrap(), 6); // 2 + 4

        arc_consumer.accept(&3, &3);
        assert_eq!(*counter.lock().unwrap(), 12); // 6 + 6

        func(&4, &4);
        assert_eq!(*counter.lock().unwrap(), 20); // 12 + 8

        // Original custom is still usable
        custom.accept(&5, &5);
        assert_eq!(*counter.lock().unwrap(), 30); // 20 + 10
    }

    #[test]
    fn test_custom_to_conversions_with_different_types() {
        let counter = Arc::new(std::sync::Mutex::new(String::new()));

        #[derive(Clone)]
        struct StringConsumer {
            counter: Arc<std::sync::Mutex<String>>,
        }

        impl ReadonlyBiConsumer<String, i32> for StringConsumer {
            fn accept(&self, first: &String, second: &i32) {
                let mut c = self.counter.lock().unwrap();
                if !c.is_empty() {
                    c.push(',');
                }
                c.push_str(&format!("{}:{}", first, second));
            }
        }

        unsafe impl Send for StringConsumer {}
        unsafe impl Sync for StringConsumer {}

        let custom = StringConsumer {
            counter: counter.clone(),
        };

        let box_consumer = custom.to_box();
        box_consumer.accept(&"a".to_string(), &1);
        assert_eq!(*counter.lock().unwrap(), "a:1");

        let rc_consumer = custom.to_rc();
        rc_consumer.accept(&"b".to_string(), &2);
        assert_eq!(*counter.lock().unwrap(), "a:1,b:2");

        let arc_consumer = custom.to_arc();
        arc_consumer.accept(&"c".to_string(), &3);
        assert_eq!(*counter.lock().unwrap(), "a:1,b:2,c:3");

        // Original custom is still usable
        custom.accept(&"d".to_string(), &4);
        assert_eq!(*counter.lock().unwrap(), "a:1,b:2,c:3,d:4");
    }
}
