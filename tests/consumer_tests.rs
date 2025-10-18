/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for Consumer types (immutable)

use prism3_function::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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
        let mut consumer = BoxConsumer::new(move |x: &i32| {
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
        let mut consumer = BoxConsumer::new(move |s: &String| {
            *l.lock().unwrap() = format!("Got: {}", s);
        });
        let text = String::from("hello");
        consumer.accept(&text);
        assert_eq!(*log.lock().unwrap(), "Got: hello");

        // Vec
        let log = Arc::new(Mutex::new(0));
        let l = log.clone();
        let mut consumer = BoxConsumer::new(move |v: &Vec<i32>| {
            *l.lock().unwrap() = v.len();
        });
        let numbers = vec![1, 2, 3];
        consumer.accept(&numbers);
        assert_eq!(*log.lock().unwrap(), 3);

        // bool
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::new(move |b: &bool| {
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
        let mut consumer = BoxConsumer::new(move |x: &i32| {
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
        let mut consumer = BoxConsumer::new(move |x: &i32| {
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
        let c1 = BoxConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        });
        let c2 = BoxConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });
        let mut combined = c1.and_then(c2);

        let value = 5;
        combined.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_noop() {
        let mut noop = BoxConsumer::<i32>::noop();
        let value = 42;
        noop.accept(&value);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_print() {
        let mut print = BoxConsumer::<i32>::print();
        let value = 42;
        print.accept(&value);
        // No assertion, just ensure it doesn't panic
    }

    #[test]
    fn test_print_with() {
        let mut print = BoxConsumer::<i32>::print_with("Value: ");
        let value = 42;
        print.accept(&value);
        // No assertion, just ensure it doesn't panic
    }

    // println and println_with methods are not available for immutable Consumer

    #[test]
    fn test_if_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut conditional = BoxConsumer::if_then(
            |x: &i32| *x > 0,
            move |x: &i32| {
                l.lock().unwrap().push(*x);
            },
        );

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
        let mut conditional = BoxConsumer::if_then_else(
            |x: &i32| *x > 0,
            move |x: &i32| {
                l1.lock().unwrap().push(*x);
            },
            move |x: &i32| {
                l2.lock().unwrap().push(-*x);
            },
        );

        let positive = 5;
        conditional.accept(&positive);
        assert_eq!(*log.lock().unwrap(), vec![5]);

        let negative = -5;
        conditional.accept(&negative);
        assert_eq!(*log.lock().unwrap(), vec![5, 5]); // -(-5) = 5
    }

    #[test]
    fn test_debug() {
        let consumer = BoxConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let consumer = BoxConsumer::new_with_name("test_consumer", |_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = BoxConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let consumer = BoxConsumer::new_with_name("my_consumer", |_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let mut consumer = ArcConsumer::new(move |x: &i32| {
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
        let mut consumer = ArcConsumer::new(move |x: &i32| {
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
        let first = ArcConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x * 2);
        });
        let second = ArcConsumer::new(move |x: &i32| {
            l2.lock().unwrap().push(*x + 10);
        });
        let mut chained = first.and_then(&second);

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.lock().unwrap(), vec![10, 15]);
    }

    #[test]
    fn test_thread_safety() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let mut noop = ArcConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_debug() {
        let consumer = ArcConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let consumer = ArcConsumer::new_with_name("test_consumer", |_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = ArcConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let consumer = ArcConsumer::new_with_name("my_consumer", |_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        let arc_consumer = consumer.into_arc();
        let mut arc_consumer2 = arc_consumer.clone();
        arc_consumer2.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
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
        let mut consumer = RcConsumer::new(move |x: &i32| {
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
        let mut consumer = RcConsumer::new(move |x: &i32| {
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
        let first = RcConsumer::new(move |x: &i32| {
            l1.borrow_mut().push(*x * 2);
        });
        let second = RcConsumer::new(move |x: &i32| {
            l2.borrow_mut().push(*x + 10);
        });
        let mut chained = first.and_then(&second);

        let value = 5;
        chained.accept(&value);
        assert_eq!(*log.borrow(), vec![10, 15]);
    }

    #[test]
    fn test_noop() {
        let mut noop = RcConsumer::<i32>::noop();
        noop.accept(&42);
        // No assertion needed, just ensure it doesn't panic
    }

    #[test]
    fn test_debug() {
        let consumer = RcConsumer::new(|_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcConsumer"));
    }

    #[test]
    fn test_debug_with_name() {
        let consumer = RcConsumer::new_with_name("test_consumer", |_x: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcConsumer"));
        assert!(debug_str.contains("test_consumer"));
    }

    #[test]
    fn test_display() {
        let consumer = RcConsumer::new(|_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcConsumer");
    }

    #[test]
    fn test_display_with_name() {
        let consumer = RcConsumer::new_with_name("my_consumer", |_x: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcConsumer(my_consumer)");
    }

    #[test]
    fn test_into_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let box_consumer = BoxConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        let mut rc_consumer: RcConsumer<i32> = box_consumer.into_rc();
        rc_consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    // RcConsumer cannot be converted to ArcConsumer because Rc is not Send
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
        let mut chained = (move |x: &i32| {
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
        let mut boxed: BoxConsumer<i32> = closure.into_box();
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
        let mut arc: ArcConsumer<i32> = closure.into_arc();
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
        let mut rc: RcConsumer<i32> = closure.into_rc();
        rc.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
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
        let mut consumer = BoxConsumer::new_with_name("logger", move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        assert_eq!(consumer.name(), Some("logger"));
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_box_consumer_set_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::new(move |x: &i32| {
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
        let mut consumer = ArcConsumer::new_with_name("logger", move |x: &i32| {
            l.lock().unwrap().push(*x);
        });
        assert_eq!(consumer.name(), Some("logger"));
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_arc_consumer_set_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcConsumer::new(move |x: &i32| {
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
        let mut consumer = RcConsumer::new_with_name("logger", move |x: &i32| {
            l.borrow_mut().push(*x);
        });
        assert_eq!(consumer.name(), Some("logger"));
        consumer.accept(&5);
        assert_eq!(*log.borrow(), vec![5]);
    }

    #[test]
    fn test_rc_consumer_set_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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

    #[test]
    fn test_print_with_multiple_values() {
        let mut consumer = BoxConsumer::print();
        consumer.accept(&1);
        consumer.accept(&2);
        consumer.accept(&3);
    }

    #[test]
    fn test_print_with_prefix_multiple_values() {
        let mut consumer = BoxConsumer::print_with("Value: ");
        consumer.accept(&1);
        consumer.accept(&2);
        consumer.accept(&3);
    }

    #[test]
    fn test_if_then_with_always_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::if_then(
            |_: &i32| true,
            move |x: &i32| {
                l.lock().unwrap().push(*x);
            },
        );
        consumer.accept(&5);
        consumer.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![5, 10]);
    }

    #[test]
    fn test_if_then_with_always_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::if_then(
            |_: &i32| false,
            move |x: &i32| {
                l.lock().unwrap().push(*x);
            },
        );
        consumer.accept(&5);
        consumer.accept(&10);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_if_then_else_all_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxConsumer::if_then_else(
            |_: &i32| true,
            move |x: &i32| {
                l1.lock().unwrap().push(*x);
            },
            move |x: &i32| {
                l2.lock().unwrap().push(*x * 100);
            },
        );
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5]);
    }

    #[test]
    fn test_if_then_else_all_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxConsumer::if_then_else(
            |_: &i32| false,
            move |x: &i32| {
                l1.lock().unwrap().push(*x);
            },
            move |x: &i32| {
                l2.lock().unwrap().push(*x * 100);
            },
        );
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![500]);
    }

    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x);
        })
        .and_then(BoxConsumer::noop());
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
        let mut consumer = BoxConsumer::new(move |x: &i32| {
            l1.lock().unwrap().push(*x);
        })
        .and_then(move |x: &i32| {
            l2.lock().unwrap().push(*x * 2);
        })
        .and_then(BoxConsumer::noop())
        .and_then(move |x: &i32| {
            l3.lock().unwrap().push(*x + 10);
        })
        .and_then(move |x: &i32| {
            l4.lock().unwrap().push(*x - 5);
        });
        consumer.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![5, 10, 15, 0]);
    }
}
