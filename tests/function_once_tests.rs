/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFunctionOnce, BoxFunctionOnce, FunctionOnce, RcFunctionOnce};
use std::thread;

// ============================================================================
// BoxFunctionOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_function_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = BoxFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxFunctionOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxFunctionOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxFunctionOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
        let pipeline = add_one
            .and_then(|x| x * 2)
            .and_then(|x| x - 3);
        assert_eq!(pipeline.apply(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxFunctionOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.apply("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxFunctionOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply(42), "42_suffix");
    }
}

// ============================================================================
// ArcFunctionOnce Tests - Consuming, thread-safe, reusable
// ============================================================================

#[cfg(test)]
mod arc_function_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = ArcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_clone_and_reuse() {
        let parse = ArcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        let cloned = parse.clone();

        // Can use both (but each consumes its input)
        assert_eq!(parse.apply("42".to_string()), 42);
        assert_eq!(cloned.apply("21".to_string()), 21);
    }

    #[test]
    fn test_identity() {
        let identity = ArcFunctionOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcFunctionOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_thread_safe() {
        let parse = ArcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        let cloned = parse.clone();

        let handle = thread::spawn(move || {
            cloned.apply("42".to_string())
        });

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(parse.apply("21".to_string()), 21);
    }
}

// ============================================================================
// RcFunctionOnce Tests - Consuming, single-threaded, reusable
// ============================================================================

#[cfg(test)]
mod rc_function_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = RcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_clone_and_reuse() {
        let parse = RcFunctionOnce::new(|s: String| {
            s.parse::<i32>().unwrap_or(0)
        });

        let cloned = parse.clone();

        assert_eq!(parse.apply("42".to_string()), 42);
        assert_eq!(cloned.apply("21".to_string()), 21);
    }

    #[test]
    fn test_identity() {
        let identity = RcFunctionOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcFunctionOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcFunctionOnce::new(|s: String| s.to_uppercase());

        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.apply("hello".to_string()), "HELLO");
        assert_eq!(func1.apply("world".to_string()), "WORLD");
        assert_eq!(func2.apply("rust".to_string()), "RUST");
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_function_once_trait() {
        fn apply_function_once<F: FunctionOnce<i32, i32>>(
            f: F,
            x: i32,
        ) -> i32 {
            f.apply(x)
        }

        let double = BoxFunctionOnce::new(|x: i32| x * 2);
        assert_eq!(apply_function_once(double, 21), 42);
    }
}

