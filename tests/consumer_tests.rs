/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for Consumer types

use prism3_function::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};

// ============================================================================
// BoxConsumer Tests
// ============================================================================

#[cfg(test)]
mod test_box_consumer {
    use super::*;

    #[test]
    fn test_new() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        consumer.accept(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let mut string_consumer = BoxConsumer::new(|s: &mut String| s.push('!'));
        let mut text = String::from("hello");
        string_consumer.accept(&mut text);
        assert_eq!(text, "hello!");

        // Vec
        let mut vec_consumer = BoxConsumer::new(|v: &mut Vec<i32>| v.push(42));
        let mut numbers = vec![1, 2, 3];
        vec_consumer.accept(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3, 42]);

        // bool
        let mut bool_consumer = BoxConsumer::new(|b: &mut bool| *b = !*b);
        let mut flag = true;
        bool_consumer.accept(&mut flag);
        assert_eq!(flag, false);
    }

    #[test]
    fn test_and_then() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        consumer.accept(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x += 1)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(|x: &mut i32| *x -= 5);

        let mut value = 10;
        consumer.accept(&mut value);
        assert_eq!(value, 17); // ((10 + 1) * 2) - 5
    }

    #[test]
    fn test_and_then_with_box_consumer() {
        let c1 = BoxConsumer::new(|x: &mut i32| *x *= 2);
        let c2 = BoxConsumer::new(|x: &mut i32| *x += 10);
        let mut combined = c1.and_then(c2);

        let mut value = 5;
        combined.accept(&mut value);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_noop() {
        let mut noop = BoxConsumer::<i32>::noop();
        let mut value = 42;
        noop.accept(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_print() {
        let mut print = BoxConsumer::<i32>::print();
        let mut value = 42;
        print.accept(&mut value);
        assert_eq!(value, 42); // 值不变
    }

    #[test]
    fn test_print_with() {
        let mut print = BoxConsumer::<i32>::print_with("Value: ");
        let mut value = 42;
        print.accept(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_transform() {
        let mut transform = BoxConsumer::transform(|x: &i32| x * 2);
        let mut value = 5;
        transform.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_if_then_true() {
        let mut consumer = BoxConsumer::if_then(|x: &i32| *x > 0, |x: &mut i32| *x += 10);

        let mut value = 5;
        consumer.accept(&mut value);
        assert_eq!(value, 15);
    }

    #[test]
    fn test_if_then_false() {
        let mut consumer = BoxConsumer::if_then(|x: &i32| *x > 0, |x: &mut i32| *x += 10);

        let mut value = -5;
        consumer.accept(&mut value);
        assert_eq!(value, -5); // 未改变
    }

    #[test]
    fn test_if_then_else() {
        let mut consumer = BoxConsumer::if_then_else(
            |x: &i32| *x > 0,
            |x: &mut i32| *x *= 2,
            |x: &mut i32| *x = -*x,
        );

        let mut positive = 10;
        consumer.accept(&mut positive);
        assert_eq!(positive, 20);

        let mut negative = -10;
        consumer.accept(&mut negative);
        assert_eq!(negative, 10);
    }

    #[test]
    fn test_into_box() {
        let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
        let mut boxed = consumer.into_box();
        let mut value = 5;
        boxed.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
        let mut rc = consumer.into_rc();
        let mut value = 5;
        rc.accept(&mut value);
        assert_eq!(value, 10);
    }

    // Note: BoxConsumer cannot be safely converted to ArcConsumer because the
    // inner function may not be Send. This test has been removed.
}

// ============================================================================
// ArcConsumer Tests
// ============================================================================

#[cfg(test)]
mod test_arc_consumer {
    use super::*;
    use std::thread;

    #[test]
    fn test_new() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        let mut c = consumer;
        c.accept(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let clone1 = consumer.clone();
        let clone2 = consumer.clone();

        let mut value1 = 5;
        let mut c1 = clone1;
        c1.accept(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        let mut c2 = clone2;
        c2.accept(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let second = ArcConsumer::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(&second);

        let mut value = 5;
        let mut c = chained;
        c.accept(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        let mut f = first;
        f.accept(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_thread_safety() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let clone = consumer.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            let mut c = clone;
            c.accept(&mut value);
            value
        });

        let mut value = 3;
        let mut c = consumer;
        c.accept(&mut value);
        assert_eq!(value, 6);

        assert_eq!(handle.join().unwrap(), 10);
    }

    #[test]
    fn test_into_box() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let mut boxed = consumer.into_box();
        let mut value = 5;
        boxed.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let mut rc = consumer.into_rc();
        let mut value = 5;
        rc.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_arc() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let mut arc = consumer.into_arc();
        let mut value = 5;
        arc.accept(&mut value);
        assert_eq!(value, 10);
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
        let consumer = RcConsumer::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        let mut c = consumer;
        c.accept(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let consumer = RcConsumer::new(|x: &mut i32| *x *= 2);
        let clone1 = consumer.clone();
        let clone2 = consumer.clone();

        let mut value1 = 5;
        let mut c1 = clone1;
        c1.accept(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        let mut c2 = clone2;
        c2.accept(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = RcConsumer::new(|x: &mut i32| *x *= 2);
        let second = RcConsumer::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(&second);

        let mut value = 5;
        let mut c = chained;
        c.accept(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        let mut f = first;
        f.accept(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_into_box() {
        let consumer = RcConsumer::new(|x: &mut i32| *x *= 2);
        let mut boxed = consumer.into_box();
        let mut value = 5;
        boxed.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let consumer = RcConsumer::new(|x: &mut i32| *x *= 2);
        let mut rc = consumer.into_rc();
        let mut value = 5;
        rc.accept(&mut value);
        assert_eq!(value, 10);
    }

    // Note: RcConsumer cannot be converted to ArcConsumer because Rc is not
    // Send. This test has been removed.
}

// ============================================================================
// Closure Extension Methods Tests
// ============================================================================

#[cfg(test)]
mod test_fn_consumer_ops {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let mut closure = |x: &mut i32| *x *= 2;
        let mut value = 5;
        closure.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_and_then() {
        let mut chained = (|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        chained.accept(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_closure_into_box() {
        let closure = |x: &mut i32| *x *= 2;
        let mut boxed = closure.into_box();
        let mut value = 5;
        boxed.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_rc() {
        let closure = |x: &mut i32| *x *= 2;
        let mut rc = closure.into_rc();
        let mut value = 5;
        rc.accept(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_arc() {
        let closure = |x: &mut i32| *x *= 2;
        let mut arc = closure.into_arc();
        let mut value = 5;
        arc.accept(&mut value);
        assert_eq!(value, 10);
    }
}

// ============================================================================
// Unified Interface Tests
// ============================================================================

#[cfg(test)]
mod test_unified_interface {
    use super::*;

    fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: i32) -> i32 {
        let mut val = value;
        consumer.accept(&mut val);
        val
    }

    #[test]
    fn test_with_box_consumer() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_consumer(&mut consumer, 5), 10);
    }

    #[test]
    fn test_with_arc_consumer() {
        let mut consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_consumer(&mut consumer, 5), 10);
    }

    #[test]
    fn test_with_rc_consumer() {
        let mut consumer = RcConsumer::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_consumer(&mut consumer, 5), 10);
    }

    #[test]
    fn test_with_closure() {
        let mut closure = |x: &mut i32| *x *= 2;
        assert_eq!(apply_consumer(&mut closure, 5), 10);
    }
}

// ============================================================================
// Complex Scenarios Tests
// ============================================================================

#[cfg(test)]
mod test_complex_scenarios {
    use super::*;

    #[test]
    fn test_data_processing_pipeline() {
        let mut pipeline = BoxConsumer::new(|x: &mut i32| {
            if *x < 0 {
                *x = 0;
            }
            if *x > 100 {
                *x = 100;
            }
        })
        .and_then(|x: &mut i32| *x /= 10)
        .and_then(|x: &mut i32| *x = *x * *x);

        let mut value1 = -50;
        pipeline.accept(&mut value1);
        assert_eq!(value1, 0);

        let mut value2 = 200;
        pipeline.accept(&mut value2);
        assert_eq!(value2, 100);

        let mut value3 = 30;
        pipeline.accept(&mut value3);
        assert_eq!(value3, 9);
    }

    #[test]
    fn test_string_processing() {
        let mut processor = BoxConsumer::new(|s: &mut String| s.retain(|c| !c.is_whitespace()))
            .and_then(|s: &mut String| *s = s.to_lowercase())
            .and_then(|s: &mut String| s.push_str("!!!"));

        let mut text = String::from("Hello World");
        processor.accept(&mut text);
        assert_eq!(text, "helloworld!!!");
    }

    #[test]
    fn test_conditional_processing() {
        let mut processor = BoxConsumer::if_then(|x: &i32| *x > 0, |x: &mut i32| *x *= 2).and_then(
            BoxConsumer::if_then(|x: &i32| *x > 100, |x: &mut i32| *x = 100),
        );

        let mut small = 5;
        processor.accept(&mut small);
        assert_eq!(small, 10);

        let mut large = 60;
        processor.accept(&mut large);
        assert_eq!(large, 100);
    }

    #[test]
    fn test_mixed_operations() {
        let mut processor = BoxConsumer::new(|x: &mut i32| *x += 10)
            .and_then(BoxConsumer::transform(|x: &i32| x * 2))
            .and_then(BoxConsumer::if_then(
                |x: &i32| *x > 50,
                |x: &mut i32| *x -= 20,
            ));

        let mut value1 = 5;
        processor.accept(&mut value1);
        assert_eq!(value1, 30);

        let mut value2 = 20;
        processor.accept(&mut value2);
        assert_eq!(value2, 40);
    }

    #[test]
    fn test_arc_consumer_reuse() {
        let double = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let add_ten = ArcConsumer::new(|x: &mut i32| *x += 10);

        let pipeline1 = double.and_then(&add_ten);
        let pipeline2 = add_ten.and_then(&double);

        let mut value1 = 5;
        let mut p1 = pipeline1;
        p1.accept(&mut value1);
        assert_eq!(value1, 20); // (5 * 2) + 10

        let mut value2 = 5;
        let mut p2 = pipeline2;
        p2.accept(&mut value2);
        assert_eq!(value2, 30); // (5 + 10) * 2
    }

    #[test]
    fn test_rc_consumer_reuse() {
        let double = RcConsumer::new(|x: &mut i32| *x *= 2);
        let add_ten = RcConsumer::new(|x: &mut i32| *x += 10);

        let pipeline1 = double.and_then(&add_ten);
        let pipeline2 = add_ten.and_then(&double);

        let mut value1 = 5;
        let mut p1 = pipeline1;
        p1.accept(&mut value1);
        assert_eq!(value1, 20); // (5 * 2) + 10

        let mut value2 = 5;
        let mut p2 = pipeline2;
        p2.accept(&mut value2);
        assert_eq!(value2, 30); // (5 + 10) * 2
    }
}

// ============================================================================
// Custom Types Tests
// ============================================================================

#[cfg(test)]
mod test_custom_types {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[test]
    fn test_with_custom_struct() {
        let mut consumer = BoxConsumer::new(|p: &mut Point| {
            p.x += 10;
            p.y += 10;
        });

        let mut point = Point { x: 5, y: 15 };
        consumer.accept(&mut point);
        assert_eq!(point, Point { x: 15, y: 25 });
    }

    #[test]
    fn test_chaining_with_custom_struct() {
        let mut processor = BoxConsumer::new(|p: &mut Point| p.x *= 2)
            .and_then(|p: &mut Point| p.y *= 2)
            .and_then(|p: &mut Point| p.x += p.y);

        let mut point = Point { x: 3, y: 4 };
        processor.accept(&mut point);
        assert_eq!(point, Point { x: 14, y: 8 });
    }

    #[test]
    fn test_conditional_with_custom_struct() {
        let mut normalizer = BoxConsumer::if_then(
            |p: &Point| p.x < 0 || p.y < 0,
            |p: &mut Point| {
                if p.x < 0 {
                    p.x = 0;
                }
                if p.y < 0 {
                    p.y = 0;
                }
            },
        );

        let mut point1 = Point { x: -5, y: 10 };
        normalizer.accept(&mut point1);
        assert_eq!(point1, Point { x: 0, y: 10 });

        let mut point2 = Point { x: 5, y: -10 };
        normalizer.accept(&mut point2);
        assert_eq!(point2, Point { x: 5, y: 0 });

        let mut point3 = Point { x: 5, y: 10 };
        normalizer.accept(&mut point3);
        assert_eq!(point3, Point { x: 5, y: 10 });
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod test_edge_cases {
    use super::*;

    #[test]
    fn test_with_zero() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x += 1);
        let mut value = 0;
        consumer.accept(&mut value);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_with_negative() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x = x.abs());
        let mut value = -42;
        consumer.accept(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_max_value() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x = x.saturating_add(1));
        let mut value = i32::MAX;
        consumer.accept(&mut value);
        assert_eq!(value, i32::MAX);
    }

    #[test]
    fn test_with_min_value() {
        let mut consumer = BoxConsumer::new(|x: &mut i32| *x = x.saturating_sub(1));
        let mut value = i32::MIN;
        consumer.accept(&mut value);
        assert_eq!(value, i32::MIN);
    }

    #[test]
    fn test_with_empty_string() {
        let mut consumer = BoxConsumer::new(|s: &mut String| s.push_str("added"));
        let mut text = String::new();
        consumer.accept(&mut text);
        assert_eq!(text, "added");
    }

    #[test]
    fn test_with_empty_vec() {
        let mut consumer = BoxConsumer::new(|v: &mut Vec<i32>| v.push(1));
        let mut numbers = Vec::new();
        consumer.accept(&mut numbers);
        assert_eq!(numbers, vec![1]);
    }

    #[test]
    fn test_unicode() {
        let mut consumer = BoxConsumer::new(|s: &mut String| *s = s.to_uppercase());
        let mut text = String::from("héllo 世界");
        consumer.accept(&mut text);
        assert_eq!(text, "HÉLLO 世界");
    }
}

// ============================================================================
// into_fn Tests
// ============================================================================

#[cfg(test)]
mod test_into_fn {
    use super::*;

    #[test]
    fn test_box_consumer_into_fn() {
        let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
        let mut values = vec![1, 2, 3, 4, 5];

        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_box_consumer_into_fn_complex() {
        let processor = BoxConsumer::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(processor.into_fn());

        assert_eq!(values, vec![12, 14, 16]); // (1*2)+10, (2*2)+10, (3*2)+10
    }

    #[test]
    fn test_arc_consumer_into_fn() {
        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let mut values = vec![1, 2, 3];

        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![2, 4, 6]);
    }

    #[test]
    fn test_arc_consumer_into_fn_composition() {
        let is_positive = ArcConsumer::new(|x: &mut i32| {
            if *x < 0 {
                *x = 0;
            }
        });
        let double = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let combined = is_positive.and_then(&double);

        let mut values = vec![-5, 1, 3, -2, 4];
        values.iter_mut().for_each(combined.into_fn());

        assert_eq!(values, vec![0, 2, 6, 0, 8]);
    }

    #[test]
    fn test_rc_consumer_into_fn() {
        let consumer = RcConsumer::new(|x: &mut i32| *x += 10);
        let mut values = vec![1, 2, 3];

        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![11, 12, 13]);
    }

    #[test]
    fn test_rc_consumer_into_fn_chained() {
        let first = RcConsumer::new(|x: &mut i32| *x *= 3);
        let second = RcConsumer::new(|x: &mut i32| *x -= 1);
        let chained = first.and_then(&second);

        let mut values = vec![2, 4, 6];
        values.iter_mut().for_each(chained.into_fn());

        assert_eq!(values, vec![5, 11, 17]); // (2*3)-1, (4*3)-1, (6*3)-1
    }

    #[test]
    fn test_closure_into_fn() {
        let closure = |x: &mut i32| *x *= 2;
        let mut values = vec![1, 2, 3, 4];

        values.iter_mut().for_each(closure.into_fn());

        assert_eq!(values, vec![2, 4, 6, 8]);
    }

    #[test]
    fn test_into_fn_with_strings() {
        let consumer = BoxConsumer::new(|s: &mut String| s.push('!'));
        let mut strings = vec![
            String::from("hello"),
            String::from("world"),
            String::from("rust"),
        ];

        strings.iter_mut().for_each(consumer.into_fn());

        assert_eq!(
            strings,
            vec![
                String::from("hello!"),
                String::from("world!"),
                String::from("rust!")
            ]
        );
    }

    #[test]
    fn test_into_fn_with_vec() {
        let consumer = BoxConsumer::new(|v: &mut Vec<i32>| v.push(0));
        let mut vecs = vec![vec![1], vec![2, 3], vec![4, 5, 6]];

        vecs.iter_mut().for_each(consumer.into_fn());

        assert_eq!(vecs, vec![vec![1, 0], vec![2, 3, 0], vec![4, 5, 6, 0]]);
    }

    #[test]
    fn test_into_fn_with_empty_iterator() {
        let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
        let mut values: Vec<i32> = vec![];

        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, Vec::<i32>::new());
    }

    #[test]
    fn test_into_fn_with_conditional() {
        let consumer = BoxConsumer::if_then(|x: &i32| *x > 0, |x: &mut i32| *x *= 2);

        let mut values = vec![-2, -1, 0, 1, 2, 3];
        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![-2, -1, 0, 2, 4, 6]);
    }

    #[test]
    fn test_into_fn_with_transform() {
        let consumer = BoxConsumer::transform(|x: &i32| x * x);

        let mut values = vec![1, 2, 3, 4, 5];
        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_into_fn_pipeline() {
        let consumer = BoxConsumer::new(|x: &mut i32| {
            if *x < 0 {
                *x = 0;
            }
        })
        .and_then(|x: &mut i32| *x += 5)
        .and_then(|x: &mut i32| *x *= 2);

        let mut values = vec![-10, -5, 0, 5, 10];
        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![10, 10, 10, 20, 30]);
        // -10 -> 0 -> 5 -> 10
        // -5 -> 0 -> 5 -> 10
        // 0 -> 0 -> 5 -> 10
        // 5 -> 5 -> 10 -> 20
        // 10 -> 10 -> 15 -> 30
    }

    #[test]
    fn test_arc_consumer_into_fn_thread_safe() {
        use std::thread;

        let consumer = ArcConsumer::new(|x: &mut i32| *x *= 2);
        let clone = consumer.clone();

        let handle = thread::spawn(move || {
            let mut values = vec![1, 2, 3];
            values.iter_mut().for_each(clone.into_fn());
            values
        });

        let result = handle.join().unwrap();
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_into_fn_with_filter_map() {
        let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
        let mut values = vec![1, 2, 3, 4, 5];

        // Use with iter_mut
        values.iter_mut().for_each(consumer.into_fn());

        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_into_fn_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        let consumer = BoxConsumer::new(|p: &mut Point| {
            p.x *= 2;
            p.y *= 2;
        });

        let mut points = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];

        points.iter_mut().for_each(consumer.into_fn());

        assert_eq!(points, vec![Point { x: 2, y: 4 }, Point { x: 6, y: 8 }]);
    }
}
