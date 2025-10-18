/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Tests for BiConsumer types

use prism3_function::{ArcBiConsumer, BiConsumer, BoxBiConsumer, FnBiConsumerOps, RcBiConsumer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod box_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut chained = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_noop() {
        let mut noop = BoxBiConsumer::<i32, i32>::noop();
        noop.accept(&42, &10);
        // Should not panic, values unchanged
    }

    #[test]
    fn test_print() {
        let mut print = BoxBiConsumer::<i32, i32>::print();
        print.accept(&42, &10); // Should print: (42, 10)
    }

    #[test]
    fn test_print_with() {
        let mut print = BoxBiConsumer::<i32, i32>::print_with("Values: ");
        print.accept(&42, &10); // Should print: Values: 42, 10
    }

    #[test]
    fn test_if_then_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut conditional = BoxBiConsumer::if_then(
            |x: &i32, y: &i32| *x > 0 && *y > 0,
            move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            },
        );

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_if_then_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut conditional = BoxBiConsumer::if_then(
            |x: &i32, y: &i32| *x > 0 && *y > 0,
            move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            },
        );

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    #[test]
    fn test_if_then_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut conditional = BoxBiConsumer::if_then_else(
            |x: &i32, y: &i32| *x > *y,
            move |x: &i32, _y: &i32| {
                l1.lock().unwrap().push(*x);
            },
            move |_x: &i32, y: &i32| {
                l2.lock().unwrap().push(*y);
            },
        );

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![5]);

        conditional.accept(&2, &7);
        assert_eq!(*log.lock().unwrap(), vec![5, 7]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let mut box_consumer = closure.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }
}

#[cfg(test)]
mod arc_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let mut clone1 = consumer.clone();
        let mut clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        clone2.accept(&10, &2);
        assert_eq!(*log.lock().unwrap(), vec![8, 12]);
    }

    #[test]
    fn test_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let second = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        let mut chained = first.and_then(&second);

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let mut func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_name() {
        let mut consumer = ArcBiConsumer::new(|_x: &i32, _y: &i32| {});
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }
}

#[cfg(test)]
mod rc_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let mut clone1 = consumer.clone();
        let mut clone2 = consumer.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);

        clone2.accept(&10, &2);
        assert_eq!(*log.borrow(), vec![8, 12]);
    }

    #[test]
    fn test_and_then() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let first = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        });
        let second = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });

        let mut chained = first.and_then(&second);

        chained.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8, 15]);
    }

    #[test]
    fn test_to_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let mut func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_into_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_name() {
        let mut consumer = RcBiConsumer::new(|_x: &i32, _y: &i32| {});
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
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        closure.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_closure_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
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
        let mut consumer = BoxBiConsumer::<i32, i32>::noop();
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        consumer.accept(&1, &2);
        // Should do nothing
    }

    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        })
        .and_then(BoxBiConsumer::noop());
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_complex_chain() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        })
        .and_then(BoxBiConsumer::noop())
        .and_then(move |x: &i32, y: &i32| {
            l3.lock().unwrap().push(*x - *y);
        });
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15, 2]);
    }

    #[test]
    fn test_with_different_types() {
        let log = Arc::new(Mutex::new(String::new()));
        let l = log.clone();
        let mut consumer = BoxBiConsumer::new(move |s: &String, n: &i32| {
            *l.lock().unwrap() = format!("{}: {}", s, n);
        });
        consumer.accept(&"Count".to_string(), &42);
        assert_eq!(*log.lock().unwrap(), "Count: 42");
    }

    #[test]
    fn test_arc_consumer_multiple_threads() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let mut cons = consumer.clone();
                std::thread::spawn(move || {
                    cons.accept(&i, &1);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Sum should be 1+2+3+...+10 = 55
        let mut result = log.lock().unwrap().clone();
        result.sort();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_rc_consumer_multiple_clones() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let mut cons1 = consumer.clone();
        let mut cons2 = consumer.clone();
        let mut cons3 = consumer.clone();

        cons1.accept(&1, &2);
        cons2.accept(&3, &4);
        cons3.accept(&5, &6);

        assert_eq!(*log.borrow(), vec![3, 7, 11]);
    }

    #[test]
    fn test_print_with_multiple_values() {
        let mut consumer = BoxBiConsumer::print();
        consumer.accept(&1, &2);
        consumer.accept(&3, &4);
        consumer.accept(&5, &6);
    }

    #[test]
    fn test_print_with_prefix_multiple_values() {
        let mut consumer = BoxBiConsumer::print_with("Pair: ");
        consumer.accept(&1, &2);
        consumer.accept(&3, &4);
        consumer.accept(&5, &6);
    }

    #[test]
    fn test_if_then_with_always_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxBiConsumer::if_then(
            |_: &i32, _: &i32| true,
            move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            },
        );
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    #[test]
    fn test_if_then_with_always_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxBiConsumer::if_then(
            |_: &i32, _: &i32| false,
            move |x: &i32, y: &i32| {
                l.lock().unwrap().push(*x + *y);
            },
        );
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_if_then_else_all_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxBiConsumer::if_then_else(
            |_: &i32, _: &i32| true,
            move |x: &i32, y: &i32| {
                l1.lock().unwrap().push(*x + *y);
            },
            move |x: &i32, y: &i32| {
                l2.lock().unwrap().push(*x * *y);
            },
        );
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_if_then_else_all_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut consumer = BoxBiConsumer::if_then_else(
            |_: &i32, _: &i32| false,
            move |x: &i32, y: &i32| {
                l1.lock().unwrap().push(*x + *y);
            },
            move |x: &i32, y: &i32| {
                l2.lock().unwrap().push(*x * *y);
            },
        );
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![15]);
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut func = consumer.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*log.lock().unwrap(), vec![3, 7, 11]);
    }

    #[test]
    fn test_rc_to_fn_multiple_calls() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut func = consumer.to_fn();
        func(&1, &2);
        func(&3, &4);
        func(&5, &6);
        assert_eq!(*log.borrow(), vec![3, 7, 11]);
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
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut box_consumer = arc_consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_arc_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut rc_consumer = arc_consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_rc_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let rc_consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut box_consumer = rc_consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_closure_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let mut box_consumer = closure.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_closure_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let mut arc_consumer = closure.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_closure_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        };
        let mut rc_consumer = closure.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }
}
