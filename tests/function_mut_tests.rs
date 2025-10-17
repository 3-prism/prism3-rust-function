/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFunctionMut, BoxFunctionMut, FunctionMut, RcFunctionMut};

// ============================================================================
// BoxFunctionMut Tests - Mutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_function_mut_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let mut double_in_place = BoxFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(double_in_place.apply(&mut value), 42);
        assert_eq!(value, 42); // Value was modified
    }

    #[test]
    fn test_multiple_calls() {
        let mut increment = BoxFunctionMut::new(|x: &mut i32| {
            *x += 1;
            *x
        });

        let mut value = 0;
        assert_eq!(increment.apply(&mut value), 1);
        assert_eq!(increment.apply(&mut value), 2);
        assert_eq!(increment.apply(&mut value), 3);
        assert_eq!(value, 3);
    }

    #[test]
    fn test_stateful_function() {
        let mut count = 0;
        let mut counter = BoxFunctionMut::new(move |x: &mut i32| {
            count += 1;
            *x += count;
            (*x, count)
        });

        let mut value = 10;
        assert_eq!(counter.apply(&mut value), (11, 1));
        assert_eq!(counter.apply(&mut value), (13, 2));
        assert_eq!(counter.apply(&mut value), (16, 3));
    }

    #[test]
    fn test_with_string() {
        let mut append = BoxFunctionMut::new(|s: &mut String| {
            s.push_str("!");
            s.len()
        });

        let mut text = "hello".to_string();
        assert_eq!(append.apply(&mut text), 6);
        assert_eq!(text, "hello!");
    }
}

// ============================================================================
// ArcFunctionMut Tests - Mutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_function_mut_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let mut double_in_place = ArcFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(double_in_place.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_clone() {
        let mut increment = ArcFunctionMut::new(|x: &mut i32| {
            *x += 1;
            *x
        });

        let mut cloned = increment.clone();

        let mut val1 = 0;
        let mut val2 = 10;

        assert_eq!(increment.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 11);
    }

    #[test]
    fn test_stateful_shared() {
        let mut count = 0;
        let mut counter = ArcFunctionMut::new(move |x: &mut i32| {
            count += 1;
            *x += count;
            count
        });

        let mut cloned = counter.clone();

        let mut val1 = 0;
        let mut val2 = 0;

        // Both share the same internal state via Mutex
        assert_eq!(counter.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 2);
        assert_eq!(counter.apply(&mut val1), 3);
    }
}

// ============================================================================
// RcFunctionMut Tests - Mutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_function_mut_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let mut double_in_place = RcFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(double_in_place.apply(&mut value), 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_clone() {
        let mut increment = RcFunctionMut::new(|x: &mut i32| {
            *x += 1;
            *x
        });

        let mut cloned = increment.clone();

        let mut val1 = 0;
        let mut val2 = 10;

        assert_eq!(increment.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 11);
    }

    #[test]
    fn test_stateful_shared() {
        let mut count = 0;
        let mut counter = RcFunctionMut::new(move |x: &mut i32| {
            count += 1;
            *x += count;
            count
        });

        let mut cloned = counter.clone();

        let mut val1 = 0;
        let mut val2 = 0;

        // Both share the same internal state via RefCell
        assert_eq!(counter.apply(&mut val1), 1);
        assert_eq!(cloned.apply(&mut val2), 2);
        assert_eq!(counter.apply(&mut val1), 3);
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_function_mut_trait() {
        fn apply_function_mut<F: FunctionMut<i32, i32>>(f: &mut F, x: &mut i32) -> i32 {
            f.apply(x)
        }

        let mut double = BoxFunctionMut::new(|x: &mut i32| {
            *x *= 2;
            *x
        });

        let mut value = 21;
        assert_eq!(apply_function_mut(&mut double, &mut value), 42);
    }
}
