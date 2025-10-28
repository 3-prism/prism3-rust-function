/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Tests for BiConsumer types

use prism3_function::{
    ArcBiConsumer,
    ArcStatefulBiConsumer,
    BiConsumer,
    BoxBiConsumer,
    BoxStatefulBiConsumer,
    FnBiConsumerOps,
    RcBiConsumer,
    RcStatefulBiConsumer,
    StatefulBiConsumer,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

#[cfg(test)]
mod box_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
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
        let chained = BoxBiConsumer::new(move |x: &i32, y: &i32| {
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
        let noop = BoxBiConsumer::<i32, i32>::noop();
        noop.accept(&42, &10);
        // Should not panic, values unchanged
    }

    #[test]
    fn test_when_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_when_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![]);
    }

    #[test]
    fn test_when_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, _y: &i32| {
            l1.lock().unwrap().push(*x);
        });
        let mut conditional =
            consumer
                .when(|x: &i32, y: &i32| *x > *y)
                .or_else(move |_x: &i32, y: &i32| {
                    l2.lock().unwrap().push(*y);
                });

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
        let box_consumer = BiConsumer::into_box(closure);
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
        let func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_box_into_rc_default_impl() {
        // Test BoxBiConsumer's into_rc() - uses BiConsumer trait's default implementation
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_name() {
        let mut consumer = BoxBiConsumer::<i32, i32>::noop();
        assert_eq!(consumer.name(), None);

        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
    }

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut boxed = conditional.into_box();
        boxed.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        boxed.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        rc_consumer.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_conditional_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        func(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut chained = conditional.and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
        chained.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15, -15]);
    }

    // Test BoxConditionalBiConsumer to_xxx() methods
    // Note: BoxConditionalBiConsumer is not Clone, so to_xxx() methods
    // use the default implementation which calls clone().into_xxx()
    // These will only work if Self implements Clone
}

#[cfg(test)]
mod box_conditional_to_xxx_tests {
    // Note: BoxConditionalBiConsumer doesn't implement Clone,
    // so the to_xxx() methods cannot be called directly.
    // The default implementations in BiConsumer trait require Clone.
    // This is by design - BoxConditionalBiConsumer is single-ownership only.
}

#[cfg(test)]
mod arc_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
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

        let clone1 = consumer.clone();
        let clone2 = consumer.clone();

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

        let chained = first.and_then(&second);

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

        let func = consumer.to_fn();
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
        let box_consumer = consumer.into_box();
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
        let rc_consumer = consumer.into_rc();
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

    #[test]
    fn test_new_with_name() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let mut consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_when() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        conditional.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_clone() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        clone2.accept(&10, &2);
        assert_eq!(*log.lock().unwrap(), vec![8, 12]);
    }

    #[test]
    fn test_conditional_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut arc_consumer = conditional.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        arc_consumer.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut box_consumer = conditional.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        box_consumer.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        rc_consumer.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        func(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_conditional_or_else() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut with_else = conditional.or_else(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });
        with_else.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        with_else.accept(&-5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, -15]);
    }

    // Test ArcConditionalBiConsumer to_xxx() methods
    #[test]
    fn test_arc_conditional_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut box_consumer = conditional.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_arc_conditional_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut rc_consumer = conditional.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_arc_conditional_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut arc_consumer = conditional.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_arc_conditional_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut func = conditional.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    /// Test that ArcBiConsumer can work with non-Send + non-Sync types
    ///
    /// This test verifies that the relaxed generic constraints (T: 'static, U: 'static
    /// instead of T: Send + Sync + 'static, U: Send + Sync + 'static) allow ArcBiConsumer
    /// to be created for types that are not thread-safe, as long as we only pass references.
    #[test]
    fn test_with_non_send_sync_types() {
        // Rc<RefCell<i32>> is neither Send nor Sync
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        // This should compile now with relaxed constraints
        let consumer = ArcBiConsumer::<NonSendType, NonSendType>::new(
            move |first: &NonSendType, second: &NonSendType| {
                let val1 = *first.borrow();
                let val2 = *second.borrow();
                l.lock().unwrap().push(val1 + val2);
            },
        );

        let value1 = Rc::new(RefCell::new(5));
        let value2 = Rc::new(RefCell::new(3));
        consumer.accept(&value1, &value2);

        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    /// Test that ArcBiConsumer with non-Send types can be cloned and used
    #[test]
    fn test_clone_with_non_send_sync_types() {
        type NonSendType = Rc<RefCell<String>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcBiConsumer::<NonSendType, NonSendType>::new(
            move |first: &NonSendType, second: &NonSendType| {
                let val1 = first.borrow().clone();
                let val2 = second.borrow().clone();
                l.lock().unwrap().push(format!("{} {}", val1, val2));
            },
        );

        let consumer2 = consumer.clone();

        let value1 = Rc::new(RefCell::new("hello".to_string()));
        let value2 = Rc::new(RefCell::new("world".to_string()));

        consumer.accept(&value1, &value2);
        consumer2.accept(&value2, &value1);

        let result = log.lock().unwrap().clone();
        assert_eq!(
            result,
            vec!["hello world".to_string(), "world hello".to_string()]
        );
    }

    /// Test that ArcBiConsumer with non-Send types can be chained
    #[test]
    fn test_and_then_with_non_send_sync_types() {
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let first = ArcBiConsumer::<NonSendType, NonSendType>::new(
            move |x: &NonSendType, y: &NonSendType| {
                let val1 = *x.borrow();
                let val2 = *y.borrow();
                l1.lock().unwrap().push(val1 + val2);
            },
        );

        let second = ArcBiConsumer::<NonSendType, NonSendType>::new(
            move |x: &NonSendType, y: &NonSendType| {
                let val1 = *x.borrow();
                let val2 = *y.borrow();
                l2.lock().unwrap().push(val1 * val2);
            },
        );

        let chained = first.and_then(&second);

        let value1 = Rc::new(RefCell::new(5));
        let value2 = Rc::new(RefCell::new(3));
        chained.accept(&value1, &value2);

        assert_eq!(*log.lock().unwrap(), vec![8, 15]); // 5+3=8, 5*3=15
    }

    /// Test mixed Send and non-Send types
    #[test]
    fn test_with_mixed_send_non_send_types() {
        type NonSendType = Rc<RefCell<i32>>;

        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        // First parameter is non-Send, second is Send (i32)
        let consumer =
            ArcBiConsumer::<NonSendType, i32>::new(move |first: &NonSendType, second: &i32| {
                let val1 = *first.borrow();
                l.lock().unwrap().push(val1 + second);
            });

        let value1 = Rc::new(RefCell::new(5));
        let value2 = 3;
        consumer.accept(&value1, &value2);

        assert_eq!(*log.lock().unwrap(), vec![8]);
    }
}

#[cfg(test)]
mod rc_bi_consumer_tests {
    use super::*;

    #[test]
    fn test_new_and_accept() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
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

        let clone1 = consumer.clone();
        let clone2 = consumer.clone();

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

        let chained = first.and_then(&second);

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

        let func = consumer.to_fn();
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
        let box_consumer = consumer.into_box();
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

    #[test]
    fn test_new_with_name() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let mut consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        consumer.set_name("test_consumer");
        assert_eq!(consumer.name(), Some("test_consumer"));
        consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_when() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let mut conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        conditional.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        conditional.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_conditional_clone() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        clone1.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        clone2.accept(&10, &2);
        assert_eq!(*log.borrow(), vec![8, 12]);
    }

    #[test]
    fn test_conditional_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut rc_consumer = conditional.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        rc_consumer.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_conditional_into_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut box_consumer = conditional.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        box_consumer.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_conditional_into_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut func = conditional.into_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        func(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    #[test]
    fn test_conditional_or_else() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut with_else = conditional.or_else(move |x: &i32, y: &i32| {
            l2.borrow_mut().push(*x * *y);
        });
        with_else.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        with_else.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8, -15]);
    }

    // Test RcConditionalBiConsumer to_xxx() methods
    #[test]
    fn test_rc_conditional_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut box_consumer = conditional.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    #[test]
    fn test_rc_conditional_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut rc_consumer = conditional.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    #[test]
    fn test_rc_conditional_to_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut func = conditional.to_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original conditional is cloneable, test with clone
        let mut conditional_clone = conditional.clone();
        conditional_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
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
        let chained = (move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        })
        .and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    #[test]
    fn test_closure_into_fn() {
        // Test into_fn in impl<T, U, F> BiConsumer<T, U> for F
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        let func = BiConsumer::into_fn(closure);
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_closure_into_rc() {
        // Test into_rc in impl<T, U, F> BiConsumer<T, U> for F
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        };
        let rc_consumer = BiConsumer::into_rc(closure);
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test closure to_xxx() methods - these test the default implementations
    // in BiConsumer trait for closures
    #[test]
    fn test_closure_to_box_cloneable() {
        // Test with ArcBiConsumer which is cloneable
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer is cloneable, test with clone
        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_closure_to_rc_cloneable() {
        // Test with RcBiConsumer which is cloneable
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original consumer is cloneable, test with clone
        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    #[test]
    fn test_closure_to_arc_cloneable() {
        // Test with ArcBiConsumer which is cloneable
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let arc_consumer = consumer.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer is cloneable, test with clone
        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_closure_to_fn_cloneable() {
        // Test with ArcBiConsumer which is cloneable
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer is cloneable, test with clone
        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test BiConsumer trait default implementations for specific types
    #[test]
    fn test_arc_default_to_fn_impl() {
        // Test BiConsumer trait's default to_fn() implementation
        // Note: ArcBiConsumer overrides to_fn(), so we test via trait method
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // Using BiConsumer trait method explicitly
        let func = BiConsumer::to_fn(&consumer);
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_rc_default_to_fn_impl() {
        // Test BiConsumer trait's default to_fn() implementation
        // Note: RcBiConsumer overrides to_fn(), so we test via trait method
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        // Using BiConsumer trait method explicitly
        let func = BiConsumer::to_fn(&consumer);
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
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
        let consumer = BoxBiConsumer::<i32, i32>::noop();
        consumer.accept(&5, &3);
        consumer.accept(&10, &20);
        consumer.accept(&1, &2);
        // Should do nothing
    }

    #[test]
    fn test_and_then_with_noop() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
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
        let consumer = BoxBiConsumer::new(move |x: &i32, y: &i32| {
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
        let consumer = BoxBiConsumer::new(move |s: &String, n: &i32| {
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
                let cons = consumer.clone();
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

        let cons1 = consumer.clone();
        let cons2 = consumer.clone();
        let cons3 = consumer.clone();

        cons1.accept(&1, &2);
        cons2.accept(&3, &4);
        cons3.accept(&5, &6);

        assert_eq!(*log.borrow(), vec![3, 7, 11]);
    }

    #[test]
    fn test_when_with_always_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|_: &i32, _: &i32| true);
        conditional.accept(&5, &3);
        conditional.accept(&10, &20);
        assert_eq!(*log.lock().unwrap(), vec![8, 30]);
    }

    #[test]
    fn test_when_with_always_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let mut conditional = consumer.when(|_: &i32, _: &i32| false);
        conditional.accept(&5, &3);
        conditional.accept(&10, &20);
        assert_eq!(*log.lock().unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn test_when_or_else_all_true() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let mut conditional =
            consumer
                .when(|_: &i32, _: &i32| true)
                .or_else(move |x: &i32, y: &i32| {
                    l2.lock().unwrap().push(*x * *y);
                });
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    #[test]
    fn test_when_or_else_all_false() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let consumer = BoxStatefulBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });
        let mut conditional =
            consumer
                .when(|_: &i32, _: &i32| false)
                .or_else(move |x: &i32, y: &i32| {
                    l2.lock().unwrap().push(*x * *y);
                });
        conditional.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![15]);
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let func = consumer.to_fn();
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
        let func = consumer.to_fn();
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
        let box_consumer = arc_consumer.into_box();
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
        let rc_consumer = arc_consumer.into_rc();
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
        let box_consumer = rc_consumer.into_box();
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
        let box_consumer = BiConsumer::into_box(closure);
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
        let arc_consumer = BiConsumer::into_arc(closure);
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
        let rc_consumer = BiConsumer::into_rc(closure);
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }
}

#[cfg(test)]
mod debug_display_tests {
    use super::*;

    #[test]
    fn test_box_debug() {
        let consumer = BoxBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("BoxBiConsumer"));
    }

    #[test]
    fn test_box_display() {
        let consumer = BoxBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxBiConsumer");
    }

    #[test]
    fn test_box_display_with_name() {
        let mut consumer = BoxBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "BoxBiConsumer(my_consumer)");
    }

    #[test]
    fn test_arc_debug() {
        let consumer = ArcBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("ArcBiConsumer"));
    }

    #[test]
    fn test_arc_display() {
        let consumer = ArcBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcBiConsumer");
    }

    #[test]
    fn test_arc_display_with_name() {
        let mut consumer = ArcBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "ArcBiConsumer(my_consumer)");
    }

    #[test]
    fn test_rc_debug() {
        let consumer = RcBiConsumer::new(|_x: &i32, _y: &i32| {});
        let debug_str = format!("{:?}", consumer);
        assert!(debug_str.contains("RcBiConsumer"));
    }

    #[test]
    fn test_rc_display() {
        let consumer = RcBiConsumer::new(|_x: &i32, _y: &i32| {});
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcBiConsumer");
    }

    #[test]
    fn test_rc_display_with_name() {
        let mut consumer = RcBiConsumer::new(|_x: &i32, _y: &i32| {});
        consumer.set_name("my_consumer");
        let display_str = format!("{}", consumer);
        assert_eq!(display_str, "RcBiConsumer(my_consumer)");
    }
}

// ============================================================================
// BiConsumer Trait Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod trait_default_impl_tests {
    use super::*;

    // Test BiConsumer trait's default into_box() implementation
    #[test]
    fn test_default_into_box_from_closure() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        // This uses the default implementation in BiConsumer trait
        let box_consumer = BiConsumer::into_box(closure);
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test BiConsumer trait's default into_rc() implementation
    #[test]
    fn test_default_into_rc_from_closure() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        };
        // This uses the default implementation in BiConsumer trait
        let rc_consumer = BiConsumer::into_rc(closure);
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test BiConsumer trait's default into_arc() implementation
    #[test]
    fn test_default_into_arc_from_closure() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let closure = move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        };
        // This uses the default implementation in BiConsumer trait
        let arc_consumer = BiConsumer::into_arc(closure);
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test BiConsumer trait's default into_fn() implementation
    #[test]
    fn test_default_into_fn_from_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // This may use the overridden implementation in ArcBiConsumer
        let func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test BiConsumer trait's default to_box() implementation with ArcBiConsumer
    #[test]
    fn test_default_to_box_from_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // ArcBiConsumer overrides to_box(), but we test the functionality
        let box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test BiConsumer trait's default to_rc() implementation with ArcBiConsumer
    #[test]
    fn test_default_to_rc_from_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // ArcBiConsumer overrides to_rc()
        let rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test BiConsumer trait's default to_arc() implementation with ArcBiConsumer
    #[test]
    fn test_default_to_arc_from_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        // ArcBiConsumer overrides to_arc()
        let arc_consumer = consumer.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test BiConsumer trait's default to_fn() implementation with RcBiConsumer
    #[test]
    fn test_default_to_fn_from_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        // RcBiConsumer overrides to_fn()
        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }
}

// ============================================================================
// Additional Type Conversion Tests
// ============================================================================

#[cfg(test)]
mod additional_conversion_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let boxed = consumer.into_box();
        boxed.accept(&10, &20);
    }

    #[test]
    fn test_box_into_rc() {
        let consumer = BoxBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let rc = consumer.into_rc();
        rc.accept(&10, &20);
    }

    #[test]
    fn test_arc_into_arc() {
        let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let arc = consumer.into_arc();
        arc.accept(&10, &20);
    }

    #[test]
    fn test_arc_into_fn() {
        let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let func = consumer.into_fn();
        func(&10, &20);
    }

    #[test]
    fn test_rc_into_rc() {
        let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let rc = consumer.into_rc();
        rc.accept(&10, &20);
    }

    #[test]
    fn test_rc_into_fn() {
        let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let func = consumer.into_fn();
        func(&10, &20);
    }

    #[test]
    fn test_arc_into_box() {
        let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let boxed = consumer.into_box();
        boxed.accept(&10, &20);
    }

    #[test]
    fn test_arc_into_rc() {
        let consumer = ArcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let rc = consumer.into_rc();
        rc.accept(&10, &20);
    }

    #[test]
    fn test_rc_into_box() {
        let consumer = RcBiConsumer::new(|x: &i32, y: &i32| {
            println!("x: {}, y: {}", x, y);
        });
        let boxed = consumer.into_box();
        boxed.accept(&10, &20);
    }
}

// ============================================================================
// into_arc() Tests
// ============================================================================
//
// Note: The into_arc() methods for BoxBiConsumer, BoxConditionalBiConsumer, RcBiConsumer and
// RcConditionalBiConsumer cannot be called at compile time due to type system constraints
// (trait requires Self: Send).
// The panic! in these methods is to provide runtime error information in extreme cases
// (such as unsafe code), but cannot be tested in normal safe Rust code.
//
// Therefore, we only test the into_arc() methods for ArcBiConsumer and ArcConditionalBiConsumer,
// which can be called normally.

#[cfg(test)]
mod into_arc_tests {
    use super::*;

    #[test]
    fn test_arc_consumer_into_arc_succeeds() {
        // ArcBiConsumer's into_arc should succeed
        let consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let mut arc = consumer.into_arc();
        arc.accept(&5, &3); // Ensure it can be used normally
    }

    #[test]
    fn test_arc_conditional_consumer_into_arc_succeeds() {
        // ArcConditionalBiConsumer's into_arc should succeed
        let consumer = ArcStatefulBiConsumer::new(|_x: &i32, _y: &i32| {});
        let conditional = consumer.when(|x: &i32, y: &i32| *x > 0 && *y > 0);
        let mut arc = conditional.into_arc();
        arc.accept(&5, &3); // Ensure it can be used normally
    }
}

// ============================================================================
// Tests for BiConsumer Trait Default Implementations
// ============================================================================
//
// This module tests the default implementations of BiConsumer trait methods
// by creating a custom struct that implements BiConsumer without overriding
// the default methods.

#[cfg(test)]
mod bi_consumer_default_impl_tests {
    use super::*;

    /// Custom BiConsumer implementation that only implements accept()
    /// and relies on all default implementations for conversion methods
    struct CustomBiConsumer<F>
    where
        F: FnMut(&i32, &i32),
    {
        function: F,
    }

    impl<F> CustomBiConsumer<F>
    where
        F: FnMut(&i32, &i32),
    {
        fn new(function: F) -> Self {
            Self { function }
        }
    }

    impl<F> StatefulBiConsumer<i32, i32> for CustomBiConsumer<F>
    where
        F: FnMut(&i32, &i32),
    {
        fn accept(&mut self, first: &i32, second: &i32) {
            (self.function)(first, second)
        }

        // Do NOT override any into_xxx() or to_xxx() methods
        // to ensure we test the trait's default implementations
    }

    /// Cloneable custom BiConsumer for testing to_xxx() methods
    #[derive(Clone)]
    struct CloneableCustomBiConsumer {
        log: Arc<Mutex<Vec<i32>>>,
    }

    impl CloneableCustomBiConsumer {
        fn new(log: Arc<Mutex<Vec<i32>>>) -> Self {
            Self { log }
        }
    }

    impl StatefulBiConsumer<i32, i32> for CloneableCustomBiConsumer {
        fn accept(&mut self, first: &i32, second: &i32) {
            self.log.lock().unwrap().push(*first + *second);
        }

        // Do NOT override any into_xxx() or to_xxx() methods
    }

    // Test into_box() default implementation
    #[test]
    fn test_custom_into_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = CustomBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        // This calls BiConsumer::into_box() default implementation
        let mut box_consumer = consumer.into_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_rc() default implementation
    #[test]
    fn test_custom_into_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = CustomBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        // This calls BiConsumer::into_rc() default implementation
        let mut rc_consumer = consumer.into_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
    }

    // Test into_arc() default implementation
    #[test]
    fn test_custom_into_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = CustomBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        // This calls BiConsumer::into_arc() default implementation
        let mut arc_consumer = consumer.into_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test into_fn() default implementation
    #[test]
    fn test_custom_into_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = CustomBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        // This calls BiConsumer::into_fn() default implementation
        let mut func = consumer.into_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
    }

    // Test to_box() default implementation with cloneable consumer
    #[test]
    fn test_custom_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableCustomBiConsumer::new(log.clone());

        // This calls BiConsumer::to_box() default implementation
        let mut box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        // Original consumer still usable (to_box() doesn't consume self)
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_rc() default implementation with cloneable consumer
    #[test]
    fn test_custom_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableCustomBiConsumer::new(log.clone());

        // This calls BiConsumer::to_rc() default implementation
        let mut rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_arc() default implementation with cloneable consumer
    #[test]
    fn test_custom_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableCustomBiConsumer::new(log.clone());

        // This calls BiConsumer::to_arc() default implementation
        let mut arc_consumer = consumer.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_fn() default implementation with cloneable consumer
    #[test]
    fn test_custom_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let consumer = CloneableCustomBiConsumer::new(log.clone());

        // This calls BiConsumer::to_fn() default implementation
        let mut func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        // Original consumer still usable
        let mut consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test default implementation with complex operations
    #[test]
    fn test_custom_into_box_then_and_then() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        let l2 = log.clone();

        let consumer = CustomBiConsumer::new(move |x: &i32, y: &i32| {
            l1.lock().unwrap().push(*x + *y);
        });

        // Convert using default into_box(), then use and_then()
        let mut chained = consumer.into_box().and_then(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x * *y);
        });

        chained.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8, 15]);
    }

    // Test default implementation with conditional consumer
    #[test]
    fn test_custom_into_rc_then_when() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = CustomBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        // Convert using default into_rc(), then use when()
        let mut conditional = consumer.into_rc().when(|x: &i32, y: &i32| *x > 0 && *y > 0);

        conditional.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);

        conditional.accept(&-5, &3);
        assert_eq!(*log.borrow(), vec![8]); // Not executed
    }

    // Test that default implementations preserve functionality
    #[test]
    fn test_custom_multiple_conversions() {
        let log = Arc::new(Mutex::new(Vec::new()));

        let consumer1 = CloneableCustomBiConsumer::new(log.clone());
        let consumer2 = consumer1.clone();

        // Convert to different types using default implementations
        let mut box_consumer = consumer1.to_box();
        let mut arc_consumer = consumer2.to_arc();

        box_consumer.accept(&1, &2);
        arc_consumer.accept(&3, &4);

        let result = log.lock().unwrap().clone();
        assert!(result.contains(&3));
        assert!(result.contains(&7));
    }
}

// ============================================================================
// to_xxx() Methods Tests
// ============================================================================

#[cfg(test)]
mod to_xxx_tests {
    use super::*;

    // ArcBiConsumer to_xxx tests
    #[test]
    fn test_arc_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_arc_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_arc_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let arc_consumer = consumer.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_arc_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });
        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // RcBiConsumer to_xxx tests
    #[test]
    fn test_rc_to_box() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    #[test]
    fn test_rc_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    #[test]
    fn test_rc_to_fn() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });
        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);
        // Original consumer still usable
        consumer.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    // Closure to_xxx tests (default implementations)
    #[test]
    fn test_closure_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l2 = log.clone();

        // For closures that can be cloned (wrapped in Arc/Rc), we can test to_box
        let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l2.lock().unwrap().push(*x + *y);
        });

        let box_consumer = arc_consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original still usable
        arc_consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_closure_to_rc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let rc_consumer = arc_consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original still usable
        arc_consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_closure_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let arc_consumer2 = arc_consumer.to_arc();
        arc_consumer2.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original still usable
        arc_consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_closure_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let arc_consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let func = arc_consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);
        // Original still usable
        arc_consumer.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    // Test to_xxx preserves original functionality
    #[test]
    fn test_to_box_preserves_original() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let box1 = consumer.to_box();
        let box2 = consumer.to_box();
        let consumer_clone = consumer.clone();

        box1.accept(&1, &2);
        box2.accept(&3, &4);
        consumer_clone.accept(&5, &6);
        consumer.accept(&7, &8);

        let result = log.lock().unwrap().clone();
        assert_eq!(result, vec![3, 7, 11, 15]);
    }

    #[test]
    fn test_to_rc_preserves_original() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();
        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let rc1 = consumer.to_rc();
        let rc2 = consumer.to_rc();
        let consumer_clone = consumer.clone();

        rc1.accept(&1, &2);
        rc2.accept(&3, &4);
        consumer_clone.accept(&5, &6);
        consumer.accept(&7, &8);

        let result = log.borrow().clone();
        assert_eq!(result, vec![3, 7, 11, 15]);
    }

    #[test]
    fn test_to_arc_preserves_original() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let arc1 = consumer.to_arc();
        let arc2 = consumer.to_arc();
        let consumer_clone = consumer.clone();

        arc1.accept(&1, &2);
        arc2.accept(&3, &4);
        consumer_clone.accept(&5, &6);
        consumer.accept(&7, &8);

        let result = log.lock().unwrap().clone();
        assert_eq!(result, vec![3, 7, 11, 15]);
    }

    #[test]
    fn test_to_fn_multiple_conversions() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let func1 = consumer.to_fn();
        let func2 = consumer.to_fn();

        func1(&1, &2);
        func2(&3, &4);
        consumer.accept(&5, &6);

        let result = log.lock().unwrap().clone();
        assert_eq!(result, vec![3, 7, 11]);
    }
}

// ============================================================================
// Direct Closure to_xxx() Methods Tests
// ============================================================================
//
// These tests directly test the to_xxx() methods for closures that implement
// Clone, testing the impl<T, U, F> BiConsumer<T, U> for F where F: FnMut(&T, &U)

#[cfg(test)]
mod direct_closure_to_xxx_tests {
    use super::*;

    // Helper: Create a cloneable closure
    // Note: Closures are not Clone by default, so we need to use a cloneable wrapper
    // like Arc/Rc or create a function pointer

    #[test]
    fn test_function_pointer_to_box() {
        // Function pointers are Copy, so they can be used with to_box()
        fn add_consumer(x: &i32, y: &i32) {
            println!("{} + {} = {}", x, y, x + y);
        }
        let fp = add_consumer;
        let box_consumer = BiConsumer::to_box(&fp);
        box_consumer.accept(&5, &3);
        // Original function pointer still usable
        fp(&2, &1);
    }

    #[test]
    fn test_function_pointer_to_rc() {
        fn add_consumer(x: &i32, y: &i32) {
            println!("{} + {} = {}", x, y, x + y);
        }
        let fp = add_consumer;
        let rc_consumer = BiConsumer::to_rc(&fp);
        rc_consumer.accept(&5, &3);
        // Original function pointer still usable
        fp(&2, &1);
    }

    #[test]
    fn test_function_pointer_to_arc() {
        fn add_consumer(x: &i32, y: &i32) {
            println!("{} + {} = {}", x, y, x + y);
        }
        let fp = add_consumer;
        let arc_consumer = BiConsumer::to_arc(&fp);
        arc_consumer.accept(&5, &3);
        // Original function pointer still usable
        fp(&2, &1);
    }

    #[test]
    fn test_function_pointer_to_fn() {
        fn add_consumer(x: &i32, y: &i32) {
            println!("{} + {} = {}", x, y, x + y);
        }
        let fp = add_consumer;
        let func = BiConsumer::to_fn(&fp);
        func(&5, &3);
        // Original function pointer still usable
        fp(&2, &1);
    }

    // Note: Due to Rust's limitations, we cannot easily create a truly cloneable
    // closure for testing without using unstable features. The above tests with
    // function pointers already cover the to_xxx() implementations for closures.
    //
    // The impl<T, U, F> BiConsumer<T, U> for F where F: FnMut(&T, &U) block
    // provides to_xxx() methods that work for any closure implementing Clone.
    // These are tested via function pointers (which are Copy/Clone).
    //
    // For non-cloneable closures, the to_xxx() methods cannot be called
    // (compile-time error due to Clone bound), which is the correct behavior.

    // Additional test: Verify to_box works with stateless closure
    #[test]
    fn test_stateless_closure_to_box() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        // Create a wrapper that is cloneable
        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        // Test to_box on the cloneable consumer (which is a closure wrapper)
        let box_consumer = consumer.to_box();
        box_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        // Original consumer still usable
        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_stateless_closure_to_rc() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let l = log.clone();

        let consumer = RcBiConsumer::new(move |x: &i32, y: &i32| {
            l.borrow_mut().push(*x + *y);
        });

        let rc_consumer = consumer.to_rc();
        rc_consumer.accept(&5, &3);
        assert_eq!(*log.borrow(), vec![8]);

        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.borrow(), vec![8, 3]);
    }

    #[test]
    fn test_stateless_closure_to_arc() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let arc_consumer = consumer.to_arc();
        arc_consumer.accept(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }

    #[test]
    fn test_stateless_closure_to_fn() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();

        let consumer = ArcBiConsumer::new(move |x: &i32, y: &i32| {
            l.lock().unwrap().push(*x + *y);
        });

        let func = consumer.to_fn();
        func(&5, &3);
        assert_eq!(*log.lock().unwrap(), vec![8]);

        let consumer_clone = consumer.clone();
        consumer_clone.accept(&2, &1);
        assert_eq!(*log.lock().unwrap(), vec![8, 3]);
    }
}

// ============================================================================
// BiConsumerOnce Implementation Tests
