/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};
use std::thread;

// ============================================================================
// BoxTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(double.transform(21), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(double.transform(21), 42);
        assert_eq!(double.transform(42), 84);
        assert_eq!(double.transform(10), 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformer::<i32, i32>::identity();
        assert_eq!(identity.transform(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformer::constant("hello");
        assert_eq!(constant.transform(123), "hello");
        assert_eq!(constant.transform(456), "hello");
    }

    #[test]
    fn test_with_string() {
        let len = BoxTransformer::new(|s: String| s.len());
        let text = "hello".to_string();
        assert_eq!(len.transform(text), 5);
        // Note: text is consumed by transform
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let multiply = BoxTransformer::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.transform(7), 21);
    }

    #[test]
    fn test_and_then() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);
        assert_eq!(composed.transform(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let composed = double.compose(add_one);
        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }
}

// ============================================================================
// ArcTransformer Tests - Immutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_transformer_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        assert_eq!(double.transform(21), 42);
    }

    #[test]
    fn test_clone() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.transform(21), 42);
        assert_eq!(cloned.transform(21), 42);
    }

    #[test]
    fn test_thread_safe() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        let handle = thread::spawn(move || cloned.transform(21));

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(double.transform(21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = ArcTransformer::<i32, i32>::identity();
        assert_eq!(identity.transform(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcTransformer::constant("hello");
        assert_eq!(constant.transform(123), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let square = ArcTransformer::new(|x: i32| x * x);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let sq = square.clone();
                thread::spawn(move || sq.transform(i))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(results, vec![0, 1, 4, 9]);
    }

    #[test]
    fn test_and_then() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let to_string = ArcTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);

        // Original double transformer still usable
        assert_eq!(double.transform(21), 42);
        assert_eq!(composed.transform(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let add_one = ArcTransformer::new(|x: i32| x + 1);
        let composed = double.compose(add_one);

        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }
}

// ============================================================================
// RcTransformer Tests - Immutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_transformer_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let double = RcTransformer::new(|x: i32| x * 2);
        assert_eq!(double.transform(21), 42);
    }

    #[test]
    fn test_clone() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.transform(21), 42);
        assert_eq!(cloned.transform(21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = RcTransformer::<i32, i32>::identity();
        assert_eq!(identity.transform(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcTransformer::constant("hello");
        assert_eq!(constant.transform(123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcTransformer::new(|s: String| s.to_uppercase());

        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.transform("hello".to_string()), "HELLO");
        assert_eq!(func1.transform("world".to_string()), "WORLD");
        assert_eq!(func2.transform("rust".to_string()), "RUST");
    }

    #[test]
    fn test_and_then() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let to_string = RcTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);

        // Original double transformer still usable
        assert_eq!(double.transform(21), 42);
        assert_eq!(composed.transform(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let composed = double.compose(add_one);

        assert_eq!(composed.transform(5), 12); // (5 + 1) * 2
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_transformer_trait() {
        fn apply_transformer<F: Transformer<i32, i32>>(f: &F, x: i32) -> i32 {
            f.transform(x)
        }

        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(apply_transformer(&double, 21), 42);
    }
}
