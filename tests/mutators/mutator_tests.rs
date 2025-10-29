/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Unit tests for Mutator types (stateless Fn(&mut T))

use prism3_function::{
    ArcMutator,
    BoxMutator,
    FnMutatorOps,
    Mutator,
    RcMutator,
};

// ============================================================================
// BoxMutator Tests
// ============================================================================

#[cfg(test)]
mod test_box_mutator {
    use super::*;

    #[test]
    fn test_new() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        mutator.mutate(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let string_mutator = BoxMutator::new(|s: &mut String| s.push('!'));
        let mut text = String::from("hello");
        string_mutator.mutate(&mut text);
        assert_eq!(text, "hello!");

        // Vec
        let vec_mutator = BoxMutator::new(|v: &mut Vec<i32>| v.push(42));
        let mut numbers = vec![1, 2, 3];
        vec_mutator.mutate(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3, 42]);

        // bool
        let bool_mutator = BoxMutator::new(|b: &mut bool| *b = !*b);
        let mut flag = true;
        bool_mutator.mutate(&mut flag);
        assert!(!flag);
    }

    #[test]
    fn test_and_then() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        mutator.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 1)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(|x: &mut i32| *x -= 5);

        let mut value = 10;
        mutator.mutate(&mut value);
        assert_eq!(value, 17); // ((10 + 1) * 2) - 5
    }

    #[test]
    fn test_and_then_with_box_mutator() {
        let c1 = BoxMutator::new(|x: &mut i32| *x *= 2);
        let c2 = BoxMutator::new(|x: &mut i32| *x += 10);
        let combined = c1.and_then(c2);

        let mut value = 5;
        combined.mutate(&mut value);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_noop() {
        let noop = BoxMutator::<i32>::noop();
        let mut value = 42;
        noop.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_with_different_types() {
        // Test with String
        let noop = BoxMutator::<String>::noop();
        let mut text = String::from("hello");
        noop.mutate(&mut text);
        assert_eq!(text, "hello");

        // Test with Vec
        let noop = BoxMutator::<Vec<i32>>::noop();
        let mut numbers = vec![1, 2, 3];
        noop.mutate(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn test_noop_chaining() {
        let chained = BoxMutator::<i32>::noop()
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(BoxMutator::<i32>::noop());

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_if_then_true() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = 5;
        mutator.mutate(&mut value);
        assert_eq!(value, 15);
    }

    #[test]
    fn test_if_then_false() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = -5;
        mutator.mutate(&mut value);
        assert_eq!(value, -5); // unchanged
    }

    #[test]
    fn test_if_then_else() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
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
        let boxed = mutator.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.into_rc();
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
        mutator.mutate(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone1 = mutator.clone();
        let clone2 = mutator.clone();

        let mut value1 = 5;
        clone1.mutate(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        clone2.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = ArcMutator::new(|x: &mut i32| *x *= 2);
        let second = ArcMutator::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(&second);

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        first.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_thread_safety() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone = mutator.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            clone.mutate(&mut value);
            value
        });

        let mut value = 3;
        mutator.mutate(&mut value);
        assert_eq!(value, 6);

        assert_eq!(handle.join().unwrap(), 10);
    }

    #[test]
    fn test_into_box() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let boxed = mutator.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_arc() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let arc = mutator.into_arc();
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_noop() {
        let noop = ArcMutator::<i32>::noop();
        let mut value = 42;
        noop.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_clone() {
        let noop = ArcMutator::<i32>::noop();
        let clone1 = noop.clone();
        let clone2 = noop.clone();

        let mut value1 = 42;
        clone1.mutate(&mut value1);
        assert_eq!(value1, 42);

        let mut value2 = 100;
        clone2.mutate(&mut value2);
        assert_eq!(value2, 100);
    }

    #[test]
    fn test_noop_chaining() {
        let noop = ArcMutator::<i32>::noop();
        let double = ArcMutator::new(|x: &mut i32| *x *= 2);

        let chained = noop.and_then(&double);

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_box() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let boxed = mutator.to_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_rc() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.to_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_arc() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let arc = mutator.to_arc();
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_fn() {
        let mutator = ArcMutator::new(|x: &mut i32| *x += 10);
        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(mutator.to_fn());
        assert_eq!(values, vec![11, 12, 13]);
    }

    #[test]
    fn test_to_box_preserves_original() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let boxed = mutator.to_box();

        // Original still usable
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 10);

        // Boxed version also works
        let mut value2 = 3;
        boxed.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_rc_preserves_original() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.to_rc();

        // Original still usable
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 10);

        // to_rc version also works
        let mut value2 = 3;
        rc.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_arc_preserves_original() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let arc = mutator.to_arc();

        // Original still usable
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 10);

        // to_arc version also works
        let mut value2 = 3;
        arc.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_fn_preserves_original() {
        let mutator = ArcMutator::new(|x: &mut i32| *x += 10);

        // to_fn version works
        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(mutator.to_fn());
        assert_eq!(values, vec![11, 12, 13]);

        // Original still usable after to_fn (because ArcMutator is Clone)
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 15);
    }

    #[test]
    fn test_to_arc_thread_safe() {
        use std::thread;

        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let arc = mutator.to_arc();
        let clone = arc.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            clone.mutate(&mut value);
            value
        });

        let mut value = 3;
        arc.mutate(&mut value);
        assert_eq!(value, 6);

        assert_eq!(handle.join().unwrap(), 10);
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
        mutator.mutate(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let clone1 = mutator.clone();
        let clone2 = mutator.clone();

        let mut value1 = 5;
        clone1.mutate(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        clone2.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = RcMutator::new(|x: &mut i32| *x *= 2);
        let second = RcMutator::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(&second);

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        first.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_into_box() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let boxed = mutator.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_into_rc() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_noop() {
        let noop = RcMutator::<i32>::noop();
        let mut value = 42;
        noop.mutate(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_clone() {
        let noop = RcMutator::<i32>::noop();
        let clone1 = noop.clone();
        let clone2 = noop.clone();

        let mut value1 = 42;
        clone1.mutate(&mut value1);
        assert_eq!(value1, 42);

        let mut value2 = 100;
        clone2.mutate(&mut value2);
        assert_eq!(value2, 100);
    }

    #[test]
    fn test_noop_chaining() {
        let noop = RcMutator::<i32>::noop();
        let double = RcMutator::new(|x: &mut i32| *x *= 2);

        let chained = noop.and_then(&double);

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 10);
    }

    // Note: RcMutator cannot be converted to ArcMutator because Rc is not
    // Send. This test has been removed.

    #[test]
    fn test_to_box() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let boxed = mutator.to_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_rc() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.to_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_to_fn() {
        let mutator = RcMutator::new(|x: &mut i32| *x += 10);
        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(mutator.to_fn());
        assert_eq!(values, vec![11, 12, 13]);
    }

    #[test]
    fn test_to_box_preserves_original() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let boxed = mutator.to_box();

        // Original still usable
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 10);

        // Boxed version also works
        let mut value2 = 3;
        boxed.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_rc_preserves_original() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        let rc = mutator.to_rc();

        // Original still usable
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 10);

        // to_rc version also works
        let mut value2 = 3;
        rc.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_to_fn_preserves_original() {
        let mutator = RcMutator::new(|x: &mut i32| *x += 10);

        // to_fn version works
        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(mutator.to_fn());
        assert_eq!(values, vec![11, 12, 13]);

        // Original still usable after to_fn (because RcMutator is Clone)
        let mut value1 = 5;
        mutator.mutate(&mut value1);
        assert_eq!(value1, 15);
    }
}

// ============================================================================
// Closure Extension Methods Tests
// ============================================================================

#[cfg(test)]
mod test_fn_mutator_ops {
    use super::*;

    #[test]
    fn test_closure_mutate() {
        let closure = |x: &mut i32| *x *= 2;
        let mut value = 5;
        closure.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_and_then() {
        let chained = (|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        chained.mutate(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_closure_into_box() {
        let closure = |x: &mut i32| *x *= 2;
        let boxed = closure.into_box();
        let mut value = 5;
        boxed.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_rc() {
        let closure = |x: &mut i32| *x *= 2;
        let rc = closure.into_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_into_arc() {
        let closure = |x: &mut i32| *x *= 2;
        let arc = closure.into_arc();
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_rc() {
        // Test non-consuming conversion to RcMutator
        // Note: Only works with cloneable closures (no mutable captures)
        let closure = |x: &mut i32| *x *= 2;
        let rc = closure.to_rc();
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_arc() {
        // Test non-consuming conversion to ArcMutator
        // Note: Only works with cloneable closures (no mutable captures)
        let closure = |x: &mut i32| *x *= 2;
        let arc = closure.to_arc();
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_closure_to_fn() {
        // Test non-consuming conversion to Fn
        // Note: Only works with cloneable closures (no mutable captures)
        let closure = |x: &mut i32| *x += 10;
        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(closure.to_fn());
        assert_eq!(values, vec![11, 12, 13]);
    }

    #[test]
    fn test_closure_to_rc_preserves_original() {
        let closure = |x: &mut i32| *x *= 2;
        let rc = closure.to_rc();

        // to_rc version works
        let mut value = 5;
        rc.mutate(&mut value);
        assert_eq!(value, 10);

        // Original closure is still usable (was copied, not moved)
        let mut value2 = 3;
        let closure_copy = closure;
        closure_copy.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_closure_to_arc_preserves_original() {
        let closure = |x: &mut i32| *x *= 2;
        let arc = closure.to_arc();

        // to_arc version works
        let mut value = 5;
        arc.mutate(&mut value);
        assert_eq!(value, 10);

        // Original closure is still usable (was copied, not moved)
        let mut value2 = 3;
        let closure_copy = closure;
        closure_copy.mutate(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_closure_to_fn_preserves_original() {
        let closure = |x: &mut i32| *x += 10;
        let fn_mutator = closure.to_fn();

        // to_fn version works
        let mut values = vec![1, 2, 3];
        values.iter_mut().for_each(fn_mutator);
        assert_eq!(values, vec![11, 12, 13]);

        // Original closure is still usable (was copied, not moved)
        let mut value = 5;
        let closure_copy = closure;
        closure_copy.mutate(&mut value);
        assert_eq!(value, 15);
    }

    #[test]
    fn test_closure_to_arc_thread_safe() {
        use std::thread;

        let closure = |x: &mut i32| *x *= 2;
        let arc = closure.to_arc();
        let clone = arc.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            clone.mutate(&mut value);
            value
        });

        let mut value = 3;
        arc.mutate(&mut value);
        assert_eq!(value, 6);

        assert_eq!(handle.join().unwrap(), 10);
    }
}

// ============================================================================
// Unified Interface Tests
// ============================================================================

#[cfg(test)]
mod test_unified_interface {
    use super::*;

    fn apply_mutator<C: Mutator<i32>>(mutator: &C, value: i32) -> i32 {
        let mut val = value;
        mutator.mutate(&mut val);
        val
    }

    #[test]
    fn test_with_box_mutator() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mutator, 5), 10);
    }

    #[test]
    fn test_with_arc_mutator() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mutator, 5), 10);
    }

    #[test]
    fn test_with_rc_mutator() {
        let mutator = RcMutator::new(|x: &mut i32| *x *= 2);
        assert_eq!(apply_mutator(&mutator, 5), 10);
    }

    #[test]
    fn test_with_closure() {
        let closure = |x: &mut i32| *x *= 2;
        assert_eq!(apply_mutator(&closure, 5), 10);
    }
}
