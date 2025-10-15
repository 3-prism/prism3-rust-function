/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFunction, BoxFunction, Function, RcFunction};
use std::thread;

// ============================================================================
// BoxFunction Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = BoxFunction::new(|x: &i32| x * 2);
        let value = 21;
        assert_eq!(double.apply(&value), 42);
        assert_eq!(value, 21); // Value is still usable
    }

    #[test]
    fn test_multiple_calls() {
        let double = BoxFunction::new(|x: &i32| x * 2);
        assert_eq!(double.apply(&21), 42);
        assert_eq!(double.apply(&42), 84);
        assert_eq!(double.apply(&10), 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(&42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxFunction::constant("hello");
        assert_eq!(constant.apply(&123), "hello");
        assert_eq!(constant.apply(&456), "hello");
    }

    #[test]
    fn test_with_string() {
        let len = BoxFunction::new(|s: &String| s.len());
        let text = "hello".to_string();
        assert_eq!(len.apply(&text), 5);
        assert_eq!(text, "hello"); // String is still usable
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let multiply = BoxFunction::new(move |x: &i32| x * multiplier);
        assert_eq!(multiply.apply(&7), 21);
    }
}

// ============================================================================
// ArcFunction Tests - Immutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = ArcFunction::new(|x: &i32| x * 2);
        let value = 21;
        assert_eq!(double.apply(&value), 42);
    }

    #[test]
    fn test_clone() {
        let double = ArcFunction::new(|x: &i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(&21), 42);
        assert_eq!(cloned.apply(&21), 42);
    }

    #[test]
    fn test_thread_safe() {
        let double = ArcFunction::new(|x: &i32| x * 2);
        let cloned = double.clone();

        let handle = thread::spawn(move || cloned.apply(&21));

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(double.apply(&21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = ArcFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(&42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcFunction::constant("hello");
        assert_eq!(constant.apply(&123), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let square = ArcFunction::new(|x: &i32| x * x);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let sq = square.clone();
                thread::spawn(move || sq.apply(&i))
            })
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        assert_eq!(results, vec![0, 1, 4, 9]);
    }
}

// ============================================================================
// RcFunction Tests - Immutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_function_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let double = RcFunction::new(|x: &i32| x * 2);
        let value = 21;
        assert_eq!(double.apply(&value), 42);
    }

    #[test]
    fn test_clone() {
        let double = RcFunction::new(|x: &i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(&21), 42);
        assert_eq!(cloned.apply(&21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = RcFunction::<i32, i32>::identity();
        assert_eq!(identity.apply(&42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcFunction::constant("hello");
        assert_eq!(constant.apply(&123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcFunction::new(|s: &String| s.to_uppercase());

        let text = "hello".to_string();
        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.apply(&text), "HELLO");
        assert_eq!(func1.apply(&text), "HELLO");
        assert_eq!(func2.apply(&text), "HELLO");
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_function_trait() {
        fn apply_function<F: Function<i32, i32>>(f: &F, x: &i32) -> i32 {
            f.apply(x)
        }

        let double = BoxFunction::new(|x: &i32| x * 2);
        assert_eq!(apply_function(&double, &21), 42);
    }
}
