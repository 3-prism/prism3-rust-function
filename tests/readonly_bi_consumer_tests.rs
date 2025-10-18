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
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcReadonlyBiConsumer(my_consumer)");
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
