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
        *value // 返回原值，因为 Consumer 不修改输入
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
