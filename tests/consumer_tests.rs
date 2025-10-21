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

    // print and print_with methods have been removed

    #[test]
    fn test_if_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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

    #[test]
    fn test_into_fn() {
        // Test into_fn in impl<T, F> Consumer<T> for F
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32| {
            l.lock().unwrap().push(*x * 2);
        };
        let mut func = closure.into_fn();
        func(&5);
        assert_eq!(*log.lock().unwrap(), vec![10]);
    }

    // 测试闭包的 to_xxx 方法
    // 注意：只有 Clone 闭包才能使用 to_xxx 方法
    // 由于标准闭包不实现 Clone,我们使用函数指针(函数指针实现了 Clone)

    #[test]
    fn test_closure_to_box_with_fn_pointer() {
        // 使用 Arc<Mutex> 来验证函数被调用
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

        // 验证原始闭包仍然可用
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

        // 验证原始闭包仍然可用
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

        // 验证原始闭包仍然可用
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

        // 为 to_fn 和后续测试使用不同的实例
        let consumer_fn1 = make_consumer(c1.clone());
        let consumer_fn2 = make_consumer(c1.clone());

        // 测试 to_fn() - 第一个实例
        let mut func = consumer_fn1.to_fn();
        func(&5);  // 5 + 10 = 15
        func(&7);  // 7 + 10 = 17

        // 验证第一部分结果
        assert_eq!(*counter.lock().unwrap(), 32); // 15 + 17

        // 使用第二个独立实例验证原始闭包仍然可用
        let mut original_func = consumer_fn2;
        original_func(&3);  // 3 + 10 = 13
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

    // print and print_with methods have been removed

    #[test]
    fn test_if_then_with_always_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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

    #[test]
    fn test_box_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = BoxConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = ArcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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
        let consumer = RcConsumer::new(move |x: &i32| {
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

    // 定义一个自定义的 Consumer 实现，用于测试默认的 into_xxx() 方法
    struct CustomConsumer {
        log: Arc<Mutex<Vec<i32>>>,
    }

    impl Consumer<i32> for CustomConsumer {
        fn accept(&mut self, value: &i32) {
            self.log.lock().unwrap().push(*value * 10);
        }

        // 不实现 into_box, into_rc, into_arc, into_fn
        // 使用默认实现
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

    // 测试自定义 Consumer 与其他 Consumer 的组合
    #[test]
    fn test_custom_consumer_chaining_with_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let custom = CustomConsumer { log: l1 };
        let box_consumer = BoxConsumer::new(move |x: &i32| {
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

        // 转换为 RcConsumer 后可以克隆
        let rc_consumer = custom.into_rc();
        let mut clone1 = rc_consumer.clone();
        let mut clone2 = rc_consumer.clone();

        clone1.accept(&1);
        clone2.accept(&2);

        let mut result = log.lock().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![10, 20]);
    }

    // 定义一个有状态的自定义 Consumer
    struct StatefulConsumer {
        log: Arc<Mutex<Vec<i32>>>,
        multiplier: i32,
    }

    impl Consumer<i32> for StatefulConsumer {
        fn accept(&mut self, value: &i32) {
            self.log.lock().unwrap().push(*value * self.multiplier);
            self.multiplier += 1; // 每次调用后增加乘数
        }
    }

    #[test]
    fn test_stateful_consumer_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let stateful = StatefulConsumer {
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
        let stateful = StatefulConsumer {
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
        let stateful = StatefulConsumer {
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
        let stateful = StatefulConsumer {
            log: log.clone(),
            multiplier: 1,
        };

        let mut func = stateful.into_fn();
        func(&10); // 10 * 1 = 10
        func(&10); // 10 * 2 = 20
        func(&10); // 10 * 3 = 30

        assert_eq!(*log.lock().unwrap(), vec![10, 20, 30]);
    }

    // 测试线程安全的自定义 Consumer
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

    let consumer = ArcConsumer::new(move |x: &i32| {
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

    let consumer = RcConsumer::new(move |x: &i32| {
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

    // 注意:闭包必须实现 Clone 才能使用 to_xxx 方法
    // 我们需要使用可克隆的闭包或者包装类型

    #[test]
    fn test_arc_consumer_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 3);
        });

        // 测试 to_box() - 应该保留原 consumer
        let mut boxed = consumer.to_box();
        boxed.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![15]);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&10);
        assert_eq!(*log.lock().unwrap(), vec![15, 30]);
    }

    #[test]
    fn test_arc_consumer_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 4);
        });

        // 测试 to_rc() - 应该保留原 consumer
        let mut rc = consumer.to_rc();
        rc.accept(&5);
        assert_eq!(*log.lock().unwrap(), vec![20]);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&2);
        assert_eq!(*log.lock().unwrap(), vec![20, 8]);
    }

    #[test]
    fn test_arc_consumer_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 5);
        });

        // 测试 to_arc() - 应该保留原 consumer
        let mut arc = consumer.to_arc();
        arc.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![15]);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&4);
        assert_eq!(*log.lock().unwrap(), vec![15, 20]);
    }

    #[test]
    fn test_arc_consumer_to_fn_preserves_original() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcConsumer::new(move |x: &i32| {
            l.lock().unwrap().push(*x * 6);
        });

        // 测试 to_fn() - 应该保留原 consumer
        let mut func = consumer.to_fn();
        func(&2);
        assert_eq!(*log.lock().unwrap(), vec![12]);

        // 因为 to_fn() 借用了 consumer,需要先完成 func 的使用
        drop(func);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![12, 18]);
    }

    #[test]
    fn test_rc_consumer_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 7);
        });

        // 测试 to_box() - 应该保留原 consumer
        let mut boxed = consumer.to_box();
        boxed.accept(&2);
        assert_eq!(*log.borrow(), vec![14]);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&3);
        assert_eq!(*log.borrow(), vec![14, 21]);
    }

    #[test]
    fn test_rc_consumer_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 8);
        });

        // 测试 to_rc() - 应该保留原 consumer
        let mut rc = consumer.to_rc();
        rc.accept(&2);
        assert_eq!(*log.borrow(), vec![16]);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&1);
        assert_eq!(*log.borrow(), vec![16, 8]);
    }

    #[test]
    fn test_rc_consumer_to_fn_preserves_original() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcConsumer::new(move |x: &i32| {
            l.borrow_mut().push(*x * 9);
        });

        // 测试 to_fn() - 应该保留原 consumer
        let mut func = consumer.to_fn();
        func(&1);
        assert_eq!(*log.borrow(), vec![9]);

        // 因为 to_fn() 借用了 consumer,需要先完成 func 的使用
        drop(func);

        // 原 consumer 仍然可用
        let mut original = consumer;
        original.accept(&2);
        assert_eq!(*log.borrow(), vec![9, 18]);
    }

    #[test]
    fn test_custom_consumer_to_box() {
        struct CustomConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl Consumer<i32> for CustomConsumer {
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

        // 测试 to_box() - 使用默认实现
        let mut boxed = custom.to_box();
        boxed.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![30]);

        // 原 custom consumer 仍然可用
        let mut original = custom;
        original.accept(&4);
        assert_eq!(*log.lock().unwrap(), vec![30, 40]);
    }

    #[test]
    fn test_custom_consumer_to_rc() {
        struct CustomConsumer {
            log: Rc<RefCell<Vec<i32>>>,
        }

        impl Consumer<i32> for CustomConsumer {
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

        // 测试 to_rc() - 使用默认实现
        let mut rc = custom.to_rc();
        rc.accept(&2);
        assert_eq!(*log.borrow(), vec![22]);

        // 原 custom consumer 仍然可用
        let mut original = custom;
        original.accept(&3);
        assert_eq!(*log.borrow(), vec![22, 33]);
    }

    #[test]
    fn test_custom_consumer_to_arc() {
        struct CustomConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl Consumer<i32> for CustomConsumer {
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

        // 测试 to_arc() - 使用默认实现
        let mut arc = custom.to_arc();
        arc.accept(&2);
        assert_eq!(*log.lock().unwrap(), vec![24]);

        // 原 custom consumer 仍然可用
        let mut original = custom;
        original.accept(&3);
        assert_eq!(*log.lock().unwrap(), vec![24, 36]);
    }

    #[test]
    fn test_custom_consumer_to_fn() {
        struct CustomConsumer {
            log: Arc<Mutex<Vec<i32>>>,
        }

        impl Consumer<i32> for CustomConsumer {
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

        // 测试 to_fn() - 使用默认实现
        let mut func = custom.to_fn();
        func(&2);
        assert_eq!(*log.lock().unwrap(), vec![26]);

        // 因为 to_fn() 借用了 custom,需要先完成 func 的使用
        drop(func);

        // 原 custom consumer 仍然可用
        let mut original = custom;
        original.accept(&1);
        assert_eq!(*log.lock().unwrap(), vec![26, 13]);
    }
}
