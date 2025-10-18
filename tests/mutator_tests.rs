/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for Mutator types

use prism3_function::{ArcMutator, BoxMutator, FnMutatorOps, Mutator, RcMutator};

// ============================================================================
// BoxMutator Tests
// ============================================================================

#[cfg(test)]
mod test_box_mutator {
    use super::*;

    #[test]
    fn test_new() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        mutator.mutate(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let mut string_mutator = BoxMutator::new(|s: &mut String| s.push('!'));
        let mut text = String::from("hello");
        string_mutator.mutate(&mut text);
        assert_eq!(text, "hello!");

        // Vec
        let mut vec_mutator = BoxMutator::new(|v: &mut Vec<i32>| v.push(42));
        let mut numbers = vec![1, 2, 3];
        vec_mutator.mutate(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3, 42]);

        // bool
        let mut bool_mutator = BoxMutator::new(|b: &mut bool| *b = !*b);
        let mut flag = true;
        bool_mutator.mutate(&mut flag);
        assert!(!flag);
    }

    #[test]
    fn test_and_then() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        mutator.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x += 1)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(|x: &mut i32| *x -= 5);

        let mut value = 10;
        mutator.mutate(&mut value);
        assert_eq!(value, 17); // ((10 + 1) * 2) - 5
    }

    #[test]
    fn test_and_then_with_box_consumer() {
        let c1 = BoxMutator::new(|x: &mut i32| *x *= 2);
        let c2 = BoxMutator::new(|x: &mut i32| *x += 10);
        let mut combined = c1.and_then(c2);

        let mut value = 5;
        combined.mutate(&mut value);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_noop() {
        let mut noop = BoxMutator::<i32>::noop();
        let mut value = 42;
        noop.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_with_different_types() {
        // Test with String
        let mut noop = BoxMutator::<String>::noop();
        let mut text = String::from("hello");
        noop.mutate(&mut text);
        assert_eq!(text, "hello");

        // Test with Vec
        let mut noop = BoxMutator::<Vec<i32>>::noop();
        let mut numbers = vec![1, 2, 3];
        noop.mutate(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn test_noop_chaining() {
        let mut chained = BoxMutator::<i32>::noop()
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(BoxMutator::<i32>::noop());

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_if_then_true() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = 5;
        mutator.mutate(&mut value);
        assert_eq!(value, 15);
    }

    #[test]
    fn test_if_then_false() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = -5;
        mutator.mutate(&mut value);
        assert_eq!(value, -5); // unchanged
    }

    #[test]
    fn test_if_then_else() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x = -*x);

        let mut positive = 10;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 20);

        let mut negative = -10;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 10);
    }

    #[test]
    fn test_into_box() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let mut boxed = mutator.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let mut rc = mutator.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    // Note: BoxMutator cannot be safely converted to ArcMutator because the
    // inner function may not be Send. This test has been removed.
}

// ============================================================================
// ArcMutator Tests
// ============================================================================

#[cfg(test)]
mod test_arc_mutator {
    use super::*;
    use std::thread;

    #[test]
    fn test_new() {
        let mutator = ArcMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        let mut c = mutator;
        c.mutate(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone1 = mutator.clone();
        let clone2 = mutator.clone();

        let mut value1 = 5;
        let mut c1 = clone1;
        c1.mutate(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        let mut c2 = clone2;
        c2.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = ArcMutator::new(|x: &mut i32| *x *= 2);
        let second = ArcMutator::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(&second);

        let mut value = 5;
        let mut c = chained;
        c.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        let mut f = first;
        f.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_thread_safety() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone = mutator.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            let mut c = clone;
            c.mutate(&mut value);
            value
        });

        let mut value = 3;
        let mut c = mutator;
        c.mutate(&mut value);
        assert_eq!(value, 6);

        assert_eq!(handle.join().unwrap(), 10);
    }

    #[test]
    fn test_into_box() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let mut boxed = mutator.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let mut rc = mutator.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_arc() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let mut arc = mutator.into_arc();
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_noop() {
        let noop = ArcMutator::<i32>::noop();
        let mut value = 42;
        let mut m = noop;
        m.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_clone() {
        let noop = ArcMutator::<i32>::noop();
        let clone1 = noop.clone();
        let clone2 = noop.clone();

        let mut value1 = 42;
        let mut m1 = clone1;
        m1.mutate(&mut value1);
        assert_eq!(value1, 42);

        let mut value2 = 100;
        let mut m2 = clone2;
        m2.mutate(&mut value2);
        assert_eq!(value2, 100);
    }

    #[test]
    fn test_noop_chaining() {
        let noop = ArcMutator::<i32>::noop();
        let double = ArcMutator::new(|x: &mut i32| *x *= 2);

        let chained = noop.and_then(&double);

        let mut value = 5;
        let mut c = chained;
        c.mutate(&mut value);
        assert_eq!(value, 10);
    }
}

// ============================================================================
// RcMutator Tests
// ============================================================================

#[cfg(test)]
mod test_rc_mutator {
    use super::*;

    #[test]
    fn test_new() {
        let mutator = RcMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        let mut c = mutator;
        c.mutate(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let clone1 = mutator.clone();
        let clone2 = mutator.clone();

        let mut value1 = 5;
        let mut c1 = clone1;
        c1.mutate(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        let mut c2 = clone2;
        c2.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = RcMutator::new(|x: &mut i32| *x *= 2);
        let second = RcMutator::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(&second);

        let mut value = 5;
        let mut c = chained;
        c.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        let mut f = first;
        f.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_into_box() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let mut boxed = mutator.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let mut rc = mutator.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_noop() {
        let noop = RcMutator::<i32>::noop();
        let mut value = 42;
        let mut m = noop;
        m.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_clone() {
        let noop = RcMutator::<i32>::noop();
        let clone1 = noop.clone();
        let clone2 = noop.clone();

        let mut value1 = 42;
        let mut m1 = clone1;
        m1.mutate(&mut value1);
        assert_eq!(value1, 42);

        let mut value2 = 100;
        let mut m2 = clone2;
        m2.mutate(&mut value2);
        assert_eq!(value2, 100);
    }

    #[test]
    fn test_noop_chaining() {
        let noop = RcMutator::<i32>::noop();
        let double = RcMutator::new(|x: &mut i32| *x *= 2);

        let chained = noop.and_then(&double);

        let mut value = 5;
        let mut c = chained;
        c.mutate(&mut value);
        assert_eq!(value, 10);
    }

    // Note: RcMutator cannot be converted to ArcMutator because Rc is not
    // Send. This test has been removed.
}

// ============================================================================
// Closure Extension Methods Tests
// ============================================================================

#[cfg(test)]
mod test_fn_mutator_ops {
    use super::*;

    #[test]
    fn test_closure_accept() {
        let mut closure = |x: &mut i32| *x *= 2;
        let mut value = 5;
        closure.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_and_then() {
        let mut chained = (|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_closure_into_box() {
        let closure = |x: &mut i32| *x *= 2;
        let mut boxed = closure.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_rc() {
        let closure = |x: &mut i32| *x *= 2;
        let mut rc = closure.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_arc() {
        let closure = |x: &mut i32| *x *= 2;
        let mut arc = closure.into_arc();
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);
    }
}

// ============================================================================
// Unified Interface Tests
// ============================================================================

#[cfg(test)]
mod test_unified_interface {
    use super::*;

    fn apply_mutator<C: Mutator<i32>>(mutator: &mut C, value: i32) -> i32 {
        let mut val = value;
        mutator.mutate(&mut val);
        val
    }

    #[test]
    fn test_with_box_consumer() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mut mutator, 5), 10);
    }

    #[test]
    fn test_with_arc_consumer() {
        let mut mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mut mutator, 5), 10);
    }

    #[test]
    fn test_with_rc_consumer() {
        let mut mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mut mutator, 5), 10);
    }

    #[test]
    fn test_with_closure() {
        let mut closure = |x: &mut i32| *x *= 2;
        assert_eq!(apply_mutator(&mut closure, 5), 10);
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
        let mut pipeline = BoxMutator::new(|x: &mut i32| {
            *x = (*x).clamp(0, 100);
        })
        .and_then(|x: &mut i32| *x /= 10)
        .and_then(|x: &mut i32| *x = *x * *x);

        let mut value1 = -50;
        pipeline.mutate(&mut value1);
        assert_eq!(value1, 0);

        let mut value2 = 200;
        pipeline.mutate(&mut value2);
        assert_eq!(value2, 100);

        let mut value3 = 30;
        pipeline.mutate(&mut value3);
        assert_eq!(value3, 9);
    }

    #[test]
    fn test_string_processing() {
        let mut processor = BoxMutator::new(|s: &mut String| s.retain(|c| !c.is_whitespace()))
            .and_then(|s: &mut String| *s = s.to_lowercase())
            .and_then(|s: &mut String| s.push_str("!!!"));

        let mut text = String::from("Hello World");
        processor.mutate(&mut text);
        assert_eq!(text, "helloworld!!!");
    }

    #[test]
    fn test_conditional_processing() {
        let cond1 = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let cond2 = BoxMutator::new(|x: &mut i32| *x = 100).when(|x: &i32| *x > 100);
        let mut processor = cond1.and_then(cond2);

        let mut small = 5;
        processor.mutate(&mut small);
        assert_eq!(small, 10);

        let mut large = 60;
        processor.mutate(&mut large);
        assert_eq!(large, 100);
    }

    #[test]
    fn test_mixed_operations() {
        let cond = BoxMutator::new(|x: &mut i32| *x -= 20).when(|x: &i32| *x > 50);
        let mut processor = BoxMutator::new(|x: &mut i32| *x += 10)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(cond);

        let mut value1 = 5;
        processor.mutate(&mut value1);
        assert_eq!(value1, 30); // (5 + 10) * 2 = 30

        let mut value2 = 20;
        processor.mutate(&mut value2);
        assert_eq!(value2, 40); // (20 + 10) * 2 = 60, 60 > 50 so 60 - 20 = 40
    }

    #[test]
    fn test_arc_mutator_reuse() {
        let double = ArcMutator::new(|x: &mut i32| *x *= 2);
        let add_ten = ArcMutator::new(|x: &mut i32| *x += 10);

        let pipeline1 = double.and_then(&add_ten);
        let pipeline2 = add_ten.and_then(&double);

        let mut value1 = 5;
        let mut p1 = pipeline1;
        p1.mutate(&mut value1);
        assert_eq!(value1, 20); // (5 * 2) + 10

        let mut value2 = 5;
        let mut p2 = pipeline2;
        p2.mutate(&mut value2);
        assert_eq!(value2, 30); // (5 + 10) * 2
    }

    #[test]
    fn test_rc_mutator_reuse() {
        let double = RcMutator::new(|x: &mut i32| *x *= 2);
        let add_ten = RcMutator::new(|x: &mut i32| *x += 10);

        let pipeline1 = double.and_then(&add_ten);
        let pipeline2 = add_ten.and_then(&double);

        let mut value1 = 5;
        let mut p1 = pipeline1;
        p1.mutate(&mut value1);
        assert_eq!(value1, 20); // (5 * 2) + 10

        let mut value2 = 5;
        let mut p2 = pipeline2;
        p2.mutate(&mut value2);
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
        let mut mutator = BoxMutator::new(|p: &mut Point| {
            p.x += 10;
            p.y += 10;
        });

        let mut point = Point { x: 5, y: 15 };
        mutator.mutate(&mut point);
        assert_eq!(point, Point { x: 15, y: 25 });
    }

    #[test]
    fn test_chaining_with_custom_struct() {
        let mut processor = BoxMutator::new(|p: &mut Point| p.x *= 2)
            .and_then(|p: &mut Point| p.y *= 2)
            .and_then(|p: &mut Point| p.x += p.y);

        let mut point = Point { x: 3, y: 4 };
        processor.mutate(&mut point);
        assert_eq!(point, Point { x: 14, y: 8 });
    }

    #[test]
    fn test_conditional_with_custom_struct() {
        let mut normalizer = BoxMutator::new(|p: &mut Point| {
            if p.x < 0 {
                p.x = 0;
            }
            if p.y < 0 {
                p.y = 0;
            }
        })
        .when(|p: &Point| p.x < 0 || p.y < 0);

        let mut point1 = Point { x: -5, y: 10 };
        normalizer.mutate(&mut point1);
        assert_eq!(point1, Point { x: 0, y: 10 });

        let mut point2 = Point { x: 5, y: -10 };
        normalizer.mutate(&mut point2);
        assert_eq!(point2, Point { x: 5, y: 0 });

        let mut point3 = Point { x: 5, y: 10 };
        normalizer.mutate(&mut point3);
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
        let mut mutator = BoxMutator::new(|x: &mut i32| *x += 1);
        let mut value = 0;
        mutator.mutate(&mut value);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_with_negative() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x = x.abs());
        let mut value = -42;
        mutator.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_max_value() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x = x.saturating_add(1));
        let mut value = i32::MAX;
        mutator.mutate(&mut value);
        assert_eq!(value, i32::MAX);
    }

    #[test]
    fn test_with_min_value() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x = x.saturating_sub(1));
        let mut value = i32::MIN;
        mutator.mutate(&mut value);
        assert_eq!(value, i32::MIN);
    }

    #[test]
    fn test_with_empty_string() {
        let mut mutator = BoxMutator::new(|s: &mut String| s.push_str("added"));
        let mut text = String::new();
        mutator.mutate(&mut text);
        assert_eq!(text, "added");
    }

    #[test]
    fn test_with_empty_vec() {
        let mut mutator = BoxMutator::new(|v: &mut Vec<i32>| v.push(1));
        let mut numbers = Vec::new();
        mutator.mutate(&mut numbers);
        assert_eq!(numbers, vec![1]);
    }

    #[test]
    fn test_unicode() {
        let mut mutator = BoxMutator::new(|s: &mut String| *s = s.to_uppercase());
        let mut text = String::from("héllo world");
        mutator.mutate(&mut text);
        assert_eq!(text, "HÉLLO WORLD");
    }
}

// ============================================================================
// into_fn Tests
// ============================================================================

#[cfg(test)]
mod test_into_fn {
    use super::*;

    #[test]
    fn test_box_mutator_into_fn() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let mut values = vec![1, 2, 3, 4, 5];

        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_box_mutator_into_fn_complex() {
        let processor = BoxMutator::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(processor.into_fn());

        assert_eq!(values, vec![12, 14, 16]); // (1*2)+10, (2*2)+10, (3*2)+10
    }

    #[test]
    fn test_arc_mutator_into_fn() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let mut values = vec![1, 2, 3];

        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![2, 4, 6]);
    }

    #[test]
    fn test_arc_mutator_into_fn_composition() {
        let is_positive = ArcMutator::new(|x: &mut i32| {
            if *x < 0 {
                *x = 0;
            }
        });
        let double = ArcMutator::new(|x: &mut i32| *x *= 2);
        let combined = is_positive.and_then(&double);

        let mut values = vec![-5, 1, 3, -2, 4];
        values.iter_mut().for_each(combined.into_fn());

        assert_eq!(values, vec![0, 2, 6, 0, 8]);
    }

    #[test]
    fn test_rc_mutator_into_fn() {
        let mutator = RcMutator::new(|x: &mut i32| *x += 10);
        let mut values = vec![1, 2, 3];

        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![11, 12, 13]);
    }

    #[test]
    fn test_rc_mutator_into_fn_chained() {
        let first = RcMutator::new(|x: &mut i32| *x *= 3);
        let second = RcMutator::new(|x: &mut i32| *x -= 1);
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
    fn test_closure_into_fn_direct() {
        // Test that closure.into_fn() returns the closure itself
        let closure = |x: &mut i32| *x *= 2;
        let mut fn_result = closure.into_fn();

        let mut value = 5;
        fn_result(&mut value);
        assert_eq!(value, 10);

        // Can be called multiple times
        let mut value2 = 3;
        fn_result(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_closure_into_fn_with_state() {
        // Test closure with captured state
        let mut count = 0;
        let closure = move |x: &mut i32| {
            count += 1;
            *x += count;
        };

        let mut fn_result = closure.into_fn();

        let mut value1 = 10;
        fn_result(&mut value1);
        assert_eq!(value1, 11); // 10 + 1

        let mut value2 = 10;
        fn_result(&mut value2);
        assert_eq!(value2, 12); // 10 + 2
    }

    #[test]
    fn test_into_fn_with_strings() {
        let mutator = BoxMutator::new(|s: &mut String| s.push('!'));
        let mut strings = vec![
            String::from("hello"),
            String::from("world"),
            String::from("rust"),
        ];

        strings.iter_mut().for_each(mutator.into_fn());

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
        let mutator = BoxMutator::new(|v: &mut Vec<i32>| v.push(0));
        let mut vecs = vec![vec![1], vec![2, 3], vec![4, 5, 6]];

        vecs.iter_mut().for_each(mutator.into_fn());

        assert_eq!(vecs, vec![vec![1, 0], vec![2, 3, 0], vec![4, 5, 6, 0]]);
    }

    #[test]
    fn test_into_fn_with_empty_iterator() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let mut values: Vec<i32> = vec![];

        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, Vec::<i32>::new());
    }

    #[test]
    fn test_into_fn_with_conditional() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);

        let mut values = vec![-2, -1, 0, 1, 2, 3];
        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![-2, -1, 0, 2, 4, 6]);
    }

    #[test]
    fn test_into_fn_with_transform() {
        let mutator = BoxMutator::new(|x: &mut i32| *x = *x * *x);

        let mut values = vec![1, 2, 3, 4, 5];
        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_into_fn_pipeline() {
        let mutator = BoxMutator::new(|x: &mut i32| {
            if *x < 0 {
                *x = 0;
            }
        })
        .and_then(|x: &mut i32| *x += 5)
        .and_then(|x: &mut i32| *x *= 2);

        let mut values = vec![-10, -5, 0, 5, 10];
        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![10, 10, 10, 20, 30]);
        // -10 -> 0 -> 5 -> 10
        // -5 -> 0 -> 5 -> 10
        // 0 -> 0 -> 5 -> 10
        // 5 -> 5 -> 10 -> 20
        // 10 -> 10 -> 15 -> 30
    }

    #[test]
    fn test_arc_mutator_into_fn_thread_safe() {
        use std::thread;

        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone = mutator.clone();

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
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let mut values = vec![1, 2, 3, 4, 5];

        // Use with iter_mut
        values.iter_mut().for_each(mutator.into_fn());

        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_into_fn_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        let mutator = BoxMutator::new(|p: &mut Point| {
            p.x *= 2;
            p.y *= 2;
        });

        let mut points = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];

        points.iter_mut().for_each(mutator.into_fn());

        assert_eq!(points, vec![Point { x: 2, y: 4 }, Point { x: 6, y: 8 }]);
    }
}

// ============================================================================
// Conditional Execution Tests (when/or_else with various parameter types)
// ============================================================================

#[cfg(test)]
mod test_conditional_execution {
    use super::*;
    use prism3_function::predicate::{ArcPredicate, BoxPredicate, RcPredicate};

    // Helper function pointer for testing
    fn is_positive(x: &i32) -> bool {
        *x > 0
    }

    fn negate(x: &mut i32) {
        *x = -*x;
    }

    // ========================================================================
    // BoxMutator::when() tests
    // ========================================================================

    #[test]
    fn test_box_when_with_closure() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_function_pointer() {
        let mut mutator =
            BoxMutator::new(|x: &mut i32| *x *= 2).when(is_positive as fn(&i32) -> bool);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_box_predicate() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_rc_predicate() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_arc_predicate() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // BoxConditionalMutator::or_else() tests
    // ========================================================================

    #[test]
    fn test_box_or_else_with_closure() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x -= 1);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -6);
    }

    #[test]
    fn test_box_or_else_with_function_pointer() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(negate as fn(&mut i32));

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 5);
    }

    #[test]
    fn test_box_or_else_with_box_mutator() {
        let else_mutator = BoxMutator::new(|x: &mut i32| *x = 0);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 0);
    }

    #[test]
    fn test_box_or_else_with_rc_mutator() {
        let else_mutator = RcMutator::new(|x: &mut i32| *x = 100);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 100);
    }

    #[test]
    fn test_box_or_else_with_arc_mutator() {
        let else_mutator = ArcMutator::new(|x: &mut i32| *x = 200);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 200);
    }

    // ========================================================================
    // BoxConditionalMutator::and_then() tests
    // ========================================================================

    #[test]
    fn test_box_conditional_and_then_with_closure() {
        let cond1 = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut chained = cond1.and_then(|x: &mut i32| *x += 10);

        let mut positive = 5;
        chained.mutate(&mut positive);
        assert_eq!(positive, 20); // 5 * 2 + 10

        let mut negative = -5;
        chained.mutate(&mut negative);
        assert_eq!(negative, 5); // -5 + 10 (not doubled)
    }

    #[test]
    fn test_box_conditional_and_then_with_box_mutator() {
        let cond1 = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let next = BoxMutator::new(|x: &mut i32| *x += 100);
        let mut chained = cond1.and_then(next);

        let mut positive = 10;
        chained.mutate(&mut positive);
        assert_eq!(positive, 120); // 10 * 2 + 100

        let mut negative = -10;
        chained.mutate(&mut negative);
        assert_eq!(negative, 90); // -10 + 100 (not doubled)
    }

    #[test]
    fn test_box_conditional_and_then_conditional() {
        let cond1 = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let cond2 = BoxMutator::new(|x: &mut i32| *x = 100).when(|x: &i32| *x > 100);
        let mut chained = cond1.and_then(cond2);

        let mut small = 5;
        chained.mutate(&mut small);
        assert_eq!(small, 10); // 5 * 2 = 10 (< 100, not capped)

        let mut large = 60;
        chained.mutate(&mut large);
        assert_eq!(large, 100); // 60 * 2 = 120 (> 100, capped)
    }

    // ========================================================================
    // RcMutator::when() tests
    // ========================================================================

    #[test]
    fn test_rc_when_with_closure() {
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_when_with_function_pointer() {
        let conditional =
            RcMutator::new(|x: &mut i32| *x *= 2).when(is_positive as fn(&i32) -> bool);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_when_with_rc_predicate() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(pred);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_when_with_box_predicate() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(pred);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // RcConditionalMutator::or_else() tests
    // ========================================================================

    #[test]
    fn test_rc_or_else_with_closure() {
        let mut mutator = RcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x -= 1);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -6);
    }

    #[test]
    fn test_rc_or_else_with_function_pointer() {
        let mut mutator = RcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(negate as fn(&mut i32));

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 5);
    }

    #[test]
    fn test_rc_or_else_with_rc_mutator() {
        let else_mutator = RcMutator::new(|x: &mut i32| *x = 100);
        let mut mutator = RcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 100);
    }

    #[test]
    fn test_rc_or_else_with_box_mutator() {
        let else_mutator = BoxMutator::new(|x: &mut i32| *x = 200);
        let mut mutator = RcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 200);
    }

    // ========================================================================
    // RcConditionalMutator::clone() tests
    // ========================================================================

    #[test]
    fn test_rc_conditional_clone() {
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        let mut value1 = 5;
        clone1.mutate(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = -5;
        clone2.mutate(&mut value2);
        assert_eq!(value2, -5);
    }

    // ========================================================================
    // ArcMutator::when() tests
    // ========================================================================

    #[test]
    fn test_arc_when_with_closure() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_when_with_function_pointer() {
        let conditional =
            ArcMutator::new(|x: &mut i32| *x *= 2).when(is_positive as fn(&i32) -> bool);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_when_with_arc_predicate() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(pred);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // ArcConditionalMutator::or_else() tests
    // ========================================================================

    #[test]
    fn test_arc_or_else_with_closure() {
        let mut mutator = ArcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x -= 1);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -6);
    }

    #[test]
    fn test_arc_or_else_with_function_pointer() {
        let mut mutator = ArcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(negate as fn(&mut i32));

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 5);
    }

    #[test]
    fn test_arc_or_else_with_arc_mutator() {
        let else_mutator = ArcMutator::new(|x: &mut i32| *x = 100);
        let mut mutator = ArcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, 100);
    }

    // Note: BoxMutator is not Send, so it cannot be used with ArcMutator::or_else()

    // ========================================================================
    // ArcConditionalMutator::clone() tests
    // ========================================================================

    #[test]
    fn test_arc_conditional_clone() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        let mut value1 = 5;
        clone1.mutate(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = -5;
        clone2.mutate(&mut value2);
        assert_eq!(value2, -5);
    }

    // ========================================================================
    // Thread safety tests for ArcConditionalMutator
    // ========================================================================

    #[test]
    fn test_arc_conditional_thread_safety() {
        use std::thread;

        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let clone = conditional.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            let mut m = clone;
            m.mutate(&mut value);
            value
        });

        let mut value = -5;
        let mut m = conditional;
        m.mutate(&mut value);
        assert_eq!(value, -5);

        assert_eq!(handle.join().unwrap(), 10);
    }

    #[test]
    fn test_arc_or_else_thread_safety() {
        use std::thread;

        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x = 0);

        let clone = mutator.clone();

        let handle = thread::spawn(move || {
            let mut value = -5;
            let mut m = clone;
            m.mutate(&mut value);
            value
        });

        let mut value = 5;
        let mut m = mutator;
        m.mutate(&mut value);
        assert_eq!(value, 10);

        assert_eq!(handle.join().unwrap(), 0);
    }

    // ========================================================================
    // Type conversion tests for ConditionalMutator
    // ========================================================================

    #[test]
    fn test_box_conditional_into_box() {
        let conditional = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut boxed = conditional.into_box();

        let mut positive = 5;
        boxed.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        boxed.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_conditional_into_rc() {
        let conditional = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut rc = conditional.into_rc();

        let mut positive = 5;
        rc.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        rc.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_conditional_into_box() {
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut boxed = conditional.into_box();

        let mut positive = 5;
        boxed.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        boxed.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_conditional_into_rc() {
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut rc = conditional.into_rc();

        let mut positive = 5;
        rc.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        rc.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_conditional_into_box() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut boxed = conditional.into_box();

        let mut positive = 5;
        boxed.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        boxed.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_conditional_into_rc() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut rc = conditional.into_rc();

        let mut positive = 5;
        rc.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        rc.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_conditional_into_arc() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut arc = conditional.into_arc();

        let mut positive = 5;
        arc.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        arc.mutate(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // into_fn tests for ConditionalMutator
    // ========================================================================

    #[test]
    fn test_box_conditional_into_fn() {
        let conditional = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut values = vec![-2, -1, 0, 1, 2, 3];

        values.iter_mut().for_each(conditional.into_fn());

        assert_eq!(values, vec![-2, -1, 0, 2, 4, 6]);
    }

    #[test]
    fn test_rc_conditional_into_fn() {
        let conditional = RcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut values = vec![-2, -1, 0, 1, 2, 3];

        values.iter_mut().for_each(conditional.into_fn());

        assert_eq!(values, vec![-2, -1, 0, 2, 4, 6]);
    }

    #[test]
    fn test_arc_conditional_into_fn() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut values = vec![-2, -1, 0, 1, 2, 3];

        values.iter_mut().for_each(conditional.into_fn());

        assert_eq!(values, vec![-2, -1, 0, 2, 4, 6]);
    }

    // ========================================================================
    // Complex conditional composition tests
    // ========================================================================

    #[test]
    fn test_nested_conditionals() {
        // When x > 0: multiply by 2, then if result > 10: cap at 10
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .and_then(BoxMutator::new(|x: &mut i32| *x = 10).when(|x: &i32| *x > 10));

        let mut small = 3;
        mutator.mutate(&mut small);
        assert_eq!(small, 6); // 3 * 2 = 6 (not capped)

        let mut medium = 5;
        mutator.mutate(&mut medium);
        assert_eq!(medium, 10); // 5 * 2 = 10 (not capped)

        let mut large = 8;
        mutator.mutate(&mut large);
        assert_eq!(large, 10); // 8 * 2 = 16 -> capped to 10

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -5); // Not doubled (condition failed)
    }

    #[test]
    fn test_or_else_chaining() {
        // If positive: double, else: triple
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x *= 3);

        let mut positive = 5;
        mutator.mutate(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.mutate(&mut negative);
        assert_eq!(negative, -15);

        let mut zero = 0;
        mutator.mutate(&mut zero);
        assert_eq!(zero, 0); // 0 * 3
    }

    #[test]
    fn test_combined_predicate_types() {
        use prism3_function::predicate::FnPredicateOps;

        // Combine predicates: x > 0 AND x < 100
        let pred = (|x: &i32| *x > 0).and(|x: &i32| *x < 100);
        let mut mutator = BoxMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut in_range = 50;
        mutator.mutate(&mut in_range);
        assert_eq!(in_range, 100); // Doubled

        let mut too_small = -10;
        mutator.mutate(&mut too_small);
        assert_eq!(too_small, -10); // Not doubled

        let mut too_large = 150;
        mutator.mutate(&mut too_large);
        assert_eq!(too_large, 150); // Not doubled
    }
}
