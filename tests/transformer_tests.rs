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
// Conditional Transformer Tests
// ============================================================================

#[cfg(test)]
mod box_conditional_tests {
    use super::*;
    use prism3_function::BoxPredicate;

    #[test]
    fn test_when_or_else() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformer::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.transform(5), 10);
        assert_eq!(result.transform(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.transform(5), 10);
        assert_eq!(result.transform(-5), 5);
        assert_eq!(result.transform(0), 0);
    }
}

#[cfg(test)]
mod arc_conditional_tests {
    use super::*;
    use prism3_function::ArcPredicate;

    #[test]
    fn test_when_or_else() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let negate = ArcTransformer::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.transform(5), 10);
        assert_eq!(result.transform(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.transform(5), 10);
        assert_eq!(result.transform(-5), 5);
        assert_eq!(result.transform(0), 0);
    }
}

#[cfg(test)]
mod rc_conditional_tests {
    use super::*;
    use prism3_function::RcPredicate;

    #[test]
    fn test_when_or_else() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let negate = RcTransformer::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.transform(5), 10);
        assert_eq!(result.transform(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.transform(5), 10);
        assert_eq!(result.transform(-5), 5);
        assert_eq!(result.transform(0), 0);
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_closure_to_box() {
        let double = |x: i32| x * 2;
        let boxed = double.into_box();
        assert_eq!(boxed.transform(21), 42);
    }

    #[test]
    fn test_closure_to_arc() {
        let double = |x: i32| x * 2;
        let arc = double.into_arc();
        assert_eq!(arc.transform(21), 42);
    }

    #[test]
    fn test_closure_to_rc() {
        let double = |x: i32| x * 2;
        let rc = double.into_rc();
        assert_eq!(rc.transform(21), 42);
    }

    #[test]
    fn test_box_to_fn() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_arc_to_fn() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_rc_to_fn() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_box_to_rc() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let rc = double.into_rc();
        assert_eq!(rc.transform(21), 42);
    }

    #[test]
    fn test_arc_to_box() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box();
        assert_eq!(boxed.transform(21), 42);
    }

    #[test]
    fn test_arc_to_rc() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let rc = double.into_rc();
        assert_eq!(rc.transform(21), 42);
    }

    #[test]
    fn test_rc_to_box() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box();
        assert_eq!(boxed.transform(21), 42);
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

    #[test]
    fn test_closure_as_transformer() {
        fn apply_transformer<F: Transformer<i32, i32>>(f: &F, x: i32) -> i32 {
            f.transform(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_transformer(&double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_transformer<T, R, F: Transformer<T, R>>(f: &F, x: T) -> R {
            f.transform(x)
        }

        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        assert_eq!(apply_transformer(&to_string, 42), "42");
    }
}

// ============================================================================
// Complex Composition Tests
// ============================================================================

#[cfg(test)]
mod complex_composition_tests {
    use super::*;

    #[test]
    fn test_multiple_and_then() {
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.transform(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let square = BoxTransformer::new(|x: i32| x * x);
        let composed = square.compose(double).compose(add_one);
        assert_eq!(composed.transform(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_arc_multiple_and_then() {
        let add_one = ArcTransformer::new(|x: i32| x + 1);
        let double = ArcTransformer::new(|x: i32| x * 2);
        let to_string = ArcTransformer::new(|x: i32| x.to_string());
        let composed = add_one.and_then(double.clone()).and_then(to_string.clone());
        assert_eq!(composed.transform(5), "12");
        // Original transformers still usable
        assert_eq!(add_one.transform(5), 6);
        assert_eq!(double.transform(5), 10);
    }

    #[test]
    fn test_rc_multiple_compose() {
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let double = RcTransformer::new(|x: i32| x * 2);
        let square = RcTransformer::new(|x: i32| x * x);
        let composed = square.compose(double.clone()).compose(add_one.clone());
        assert_eq!(composed.transform(5), 144);
        // Original transformers still usable
        assert_eq!(add_one.transform(5), 6);
        assert_eq!(double.transform(5), 10);
        assert_eq!(square.transform(5), 25);
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_identity_composition() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let identity = BoxTransformer::<i32, i32>::identity();
        let composed = double.and_then(identity);
        assert_eq!(composed.transform(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformer::constant("hello");
        assert_eq!(constant.transform(123), "hello");
        assert_eq!(constant.transform(456), "hello");
        assert_eq!(constant.transform(789), "hello");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformer::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.transform("42".to_string()), Some(42));
        assert_eq!(parse.transform("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformer::new(|s: String| s.parse::<i32>());
        assert!(parse.transform("42".to_string()).is_ok());
        assert!(parse.transform("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformer::new(|s: String| {
            s.split(',').map(|s| s.to_string()).collect::<Vec<_>>()
        });
        assert_eq!(
            split.transform("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_arc_with_large_data() {
        let process = ArcTransformer::new(|v: Vec<i32>| v.iter().sum::<i32>());
        let data = (1..=100).collect::<Vec<_>>();
        assert_eq!(process.transform(data), 5050);
    }
}
