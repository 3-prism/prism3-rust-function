/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

// ============================================================================
// BoxTransformer Tests - Immutable, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_tests {
    use prism3_function::{BoxTransformer, Transformer};

    #[test]
    fn test_new_and_apply() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_multiple_calls() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
        assert_eq!(double.apply(42), 84);
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformer::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
    }

    #[test]
    fn test_with_string() {
        let len = BoxTransformer::new(|s: String| s.len());
        let text = "hello".to_string();
        assert_eq!(len.apply(text), 5);
        // Note: text is consumed by transform
    }

    #[test]
    fn test_captured_variable() {
        let multiplier = 3;
        let multiply = BoxTransformer::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.apply(7), 21);
    }

    #[test]
    fn test_and_then() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);
        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let composed = double.compose(add_one);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }
}

// ============================================================================
// ArcTransformer Tests - Immutable, thread-safe
// ============================================================================

#[cfg(test)]
mod arc_transformer_tests {
    use prism3_function::{ArcTransformer, Transformer};
    use std::thread;

    #[test]
    fn test_new_and_apply() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_clone() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(21), 42);
        assert_eq!(cloned.apply(21), 42);
    }

    #[test]
    fn test_thread_safe() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        let handle = thread::spawn(move || cloned.apply(21));

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = ArcTransformer::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = ArcTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_multiple_threads() {
        let square = ArcTransformer::new(|x: i32| x * x);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let sq = square.clone();
                thread::spawn(move || sq.apply(i))
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
        assert_eq!(double.apply(21), 42);
        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let add_one = ArcTransformer::new(|x: i32| x + 1);
        let composed = double.compose(add_one);

        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }
}

// ============================================================================
// RcTransformer Tests - Immutable, single-threaded
// ============================================================================

#[cfg(test)]
mod rc_transformer_tests {
    use prism3_function::{RcTransformer, Transformer};

    #[test]
    fn test_new_and_apply() {
        let double = RcTransformer::new(|x: i32| x * 2);
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_clone() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let cloned = double.clone();

        assert_eq!(double.apply(21), 42);
        assert_eq!(cloned.apply(21), 42);
    }

    #[test]
    fn test_identity() {
        let identity = RcTransformer::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = RcTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_shared_usage() {
        let to_upper = RcTransformer::new(|s: String| s.to_uppercase());

        let func1 = to_upper.clone();
        let func2 = to_upper.clone();

        assert_eq!(to_upper.apply("hello".to_string()), "HELLO");
        assert_eq!(func1.apply("world".to_string()), "WORLD");
        assert_eq!(func2.apply("rust".to_string()), "RUST");
    }

    #[test]
    fn test_and_then() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let to_string = RcTransformer::new(|x: i32| x.to_string());
        let composed = double.and_then(to_string);

        // Original double transformer still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_compose() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let composed = double.compose(add_one);

        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }
}

// ============================================================================
// Conditional Transformer Tests
// ============================================================================

#[cfg(test)]
mod box_conditional_tests {
    use prism3_function::{BoxPredicate, BoxTransformer, Transformer};

    #[test]
    fn test_when_or_else() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformer::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
        assert_eq!(result.apply(0), 0);
    }
}

#[cfg(test)]
mod arc_conditional_tests {
    use prism3_function::{ArcPredicate, ArcTransformer, Transformer};

    #[test]
    fn test_when_or_else() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let negate = ArcTransformer::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
        assert_eq!(result.apply(0), 0);
    }

    #[test]
    fn test_conditional_clone() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let conditional = double.when(|x: &i32| *x > 0);
        let cloned = conditional.clone();

        let result1 = conditional.or_else(|x: i32| -x);
        let result2 = cloned.or_else(|x: i32| -x);

        assert_eq!(result1.apply(5), 10);
        assert_eq!(result2.apply(5), 10);
        assert_eq!(result1.apply(-5), 5);
        assert_eq!(result2.apply(-5), 5);
    }
}

#[cfg(test)]
mod rc_conditional_tests {
    use prism3_function::{RcPredicate, RcTransformer, Transformer};

    #[test]
    fn test_when_or_else() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let negate = RcTransformer::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        assert_eq!(result.apply(-5), 5);
        assert_eq!(result.apply(0), 0);
    }

    #[test]
    fn test_conditional_clone() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let conditional = double.when(|x: &i32| *x > 0);
        let cloned = conditional.clone();

        let result1 = conditional.or_else(|x: i32| -x);
        let result2 = cloned.or_else(|x: i32| -x);

        assert_eq!(result1.apply(5), 10);
        assert_eq!(result2.apply(5), 10);
        assert_eq!(result1.apply(-5), 5);
        assert_eq!(result2.apply(-5), 5);
    }
}

// ============================================================================
// Conversion Tests
// ============================================================================

#[cfg(test)]
mod conversion_tests {
    use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};

    #[test]
    fn test_closure_to_box() {
        let double = |x: i32| x * 2;
        let boxed = double.into_box();
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_closure_to_arc() {
        let double = |x: i32| x * 2;
        let arc = double.into_arc();
        assert_eq!(arc.apply(21), 42);
    }

    #[test]
    fn test_closure_to_rc() {
        let double = |x: i32| x * 2;
        let rc = double.into_rc();
        assert_eq!(rc.apply(21), 42);
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
        assert_eq!(rc.apply(21), 42);
    }

    #[test]
    fn test_arc_to_box() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box();
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_arc_to_rc() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let rc = double.into_rc();
        assert_eq!(rc.apply(21), 42);
    }

    #[test]
    fn test_rc_to_box() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box();
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        // Test into_fn in impl<F, T, R> Transformer<T, R> for F
        let double = |x: i32| x * 2;
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }
}

// ============================================================================
// Non-consuming Conversion Tests (to_xxx methods)
// ============================================================================

#[cfg(test)]
mod to_conversion_tests {
    use prism3_function::{ArcTransformer, RcTransformer, Transformer};
    use std::thread;

    // ArcTransformer to_xxx tests
    #[test]
    fn test_arc_to_box() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let boxed = double.to_box();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_arc_to_rc() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let rc = double.to_rc();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(rc.apply(21), 42);
    }

    #[test]
    fn test_arc_to_arc() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let arc2 = double.to_arc();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(arc2.apply(21), 42);
    }

    #[test]
    fn test_arc_to_fn() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let func = double.to_fn();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(func(21), 42);
    }

    // RcTransformer to_xxx tests
    #[test]
    fn test_rc_to_box() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let boxed = double.to_box();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_rc_to_rc() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let rc2 = double.to_rc();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(rc2.apply(21), 42);
    }

    #[test]
    fn test_rc_to_fn() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let func = double.to_fn();

        // Original still usable
        assert_eq!(double.apply(21), 42);
        assert_eq!(func(21), 42);
    }

    // Test to_xxx with composition
    #[test]
    fn test_arc_to_box_with_composition() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let to_string = ArcTransformer::new(|x: i32| x.to_string());

        let boxed = double.to_box();
        let composed = boxed.and_then(to_string);

        assert_eq!(composed.apply(21), "42");
    }

    #[test]
    fn test_rc_to_box_with_composition() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let to_string = RcTransformer::new(|x: i32| x.to_string());

        let boxed = double.to_box();
        let composed = boxed.and_then(to_string);

        assert_eq!(composed.apply(21), "42");
    }

    // Test multiple conversions
    #[test]
    fn test_arc_multiple_to_conversions() {
        let double = ArcTransformer::new(|x: i32| x * 2);

        let boxed = double.to_box();
        let rc = double.to_rc();
        let func = double.to_fn();

        // All still work
        assert_eq!(double.apply(21), 42);
        assert_eq!(boxed.apply(21), 42);
        assert_eq!(rc.apply(21), 42);
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_rc_multiple_to_conversions() {
        let double = RcTransformer::new(|x: i32| x * 2);

        let boxed = double.to_box();
        let rc2 = double.to_rc();
        let func = double.to_fn();

        // All still work
        assert_eq!(double.apply(21), 42);
        assert_eq!(boxed.apply(21), 42);
        assert_eq!(rc2.apply(21), 42);
        assert_eq!(func(21), 42);
    }

    // Test with different types
    #[test]
    fn test_arc_to_box_with_string() {
        let len = ArcTransformer::new(|s: String| s.len());
        let boxed = len.to_box();

        assert_eq!(len.apply("hello".to_string()), 5);
        assert_eq!(boxed.apply("world".to_string()), 5);
    }

    #[test]
    fn test_rc_to_fn_with_string() {
        let upper = RcTransformer::new(|s: String| s.to_uppercase());
        let func = upper.to_fn();

        assert_eq!(upper.apply("hello".to_string()), "HELLO");
        assert_eq!(func("world".to_string()), "WORLD");
    }

    // Test thread safety with Arc - clone first to get owned values
    #[test]
    fn test_arc_to_fn_thread_safe() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let double1 = double.clone();
        let double2 = double.clone();

        let handle1 = thread::spawn(move || {
            let func = double1.to_fn();
            func(21)
        });
        let handle2 = thread::spawn(move || {
            let func = double2.to_fn();
            func(10)
        });

        assert_eq!(handle1.join().unwrap(), 42);
        assert_eq!(handle2.join().unwrap(), 20);

        // Original still usable
        assert_eq!(double.apply(5), 10);
    }

    // Test that to_xxx creates independent copies
    #[test]
    fn test_arc_to_conversions_are_independent() {
        let double = ArcTransformer::new(|x: i32| x * 2);

        let boxed1 = double.to_box();
        let boxed2 = double.to_box();

        // Both work independently
        assert_eq!(boxed1.apply(21), 42);
        assert_eq!(boxed2.apply(10), 20);
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use prism3_function::{BoxTransformer, Transformer};

    #[test]
    fn test_transformer_trait() {
        fn apply_transformer<F: Transformer<i32, i32>>(f: &F, x: i32) -> i32 {
            f.apply(x)
        }

        let double = BoxTransformer::new(|x: i32| x * 2);
        assert_eq!(apply_transformer(&double, 21), 42);
    }

    #[test]
    fn test_closure_as_transformer() {
        fn apply_transformer<F: Transformer<i32, i32>>(f: &F, x: i32) -> i32 {
            f.apply(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_transformer(&double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_transformer<T, R, F: Transformer<T, R>>(f: &F, x: T) -> R {
            f.apply(x)
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
    use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};

    #[test]
    fn test_multiple_and_then() {
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.apply(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = BoxTransformer::new(|x: i32| x + 1);
        let double = BoxTransformer::new(|x: i32| x * 2);
        let square = BoxTransformer::new(|x: i32| x * x);
        let composed = square.compose(double).compose(add_one);
        assert_eq!(composed.apply(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_arc_multiple_and_then() {
        let add_one = ArcTransformer::new(|x: i32| x + 1);
        let double = ArcTransformer::new(|x: i32| x * 2);
        let to_string = ArcTransformer::new(|x: i32| x.to_string());
        let composed = add_one.and_then(double.clone()).and_then(to_string.clone());
        assert_eq!(composed.apply(5), "12");
        // Original transformers still usable
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);
    }

    #[test]
    fn test_rc_multiple_compose() {
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let double = RcTransformer::new(|x: i32| x * 2);
        let square = RcTransformer::new(|x: i32| x * x);
        let composed = square.compose(double.clone()).compose(add_one.clone());
        assert_eq!(composed.apply(5), 144);
        // Original transformers still usable
        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);
        assert_eq!(square.apply(5), 25);
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use prism3_function::{ArcTransformer, BoxTransformer, Transformer};

    #[test]
    fn test_identity_composition() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let identity = BoxTransformer::<i32, i32>::identity();
        let composed = double.and_then(identity);
        assert_eq!(composed.apply(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformer::constant("hello");
        assert_eq!(constant.apply(123), "hello");
        assert_eq!(constant.apply(456), "hello");
        assert_eq!(constant.apply(789), "hello");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformer::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.apply("42".to_string()), Some(42));
        assert_eq!(parse.apply("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformer::new(|s: String| s.parse::<i32>());
        assert!(parse.apply("42".to_string()).is_ok());
        assert!(parse.apply("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformer::new(|s: String| {
            s.split(',').map(|s| s.to_string()).collect::<Vec<_>>()
        });
        assert_eq!(
            split.apply("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_arc_with_large_data() {
        let process = ArcTransformer::new(|v: Vec<i32>| v.iter().sum::<i32>());
        let data = (1..=100).collect::<Vec<_>>();
        assert_eq!(process.apply(data), 5050);
    }
}

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod default_implementation_tests {
    use prism3_function::{BoxTransformer, Transformer};
    use std::thread;

    // A custom transformer that only implements the core `transform`
    // method, relying on default implementations for all `into_xxx`
    // methods.
    struct CustomTransformer {
        multiplier: i32,
    }

    impl Transformer<i32, i32> for CustomTransformer {
        fn apply(&self, input: i32) -> i32 {
            input * self.multiplier
        }
    }

    #[test]
    fn test_custom_into_box_uses_default() {
        let custom = CustomTransformer { multiplier: 3 };
        let boxed = custom.into_box();

        // Test that the BoxTransformer works correctly with the
        // default implementation
        assert_eq!(boxed.apply(7), 21);
        assert_eq!(boxed.apply(10), 30);
    }

    #[test]
    fn test_custom_into_rc_uses_default() {
        let custom = CustomTransformer { multiplier: 5 };
        let rc = custom.into_rc();

        // Test that the RcTransformer works correctly with the
        // default implementation
        assert_eq!(rc.apply(4), 20);

        // Test that cloning works
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.apply(6), 30);
    }

    #[test]
    fn test_custom_into_fn_uses_default() {
        let custom = CustomTransformer { multiplier: 7 };
        let func = custom.into_fn();

        // Test that the closure works correctly with the default
        // implementation
        assert_eq!(func(3), 21);
        assert_eq!(func(5), 35);
    }

    #[test]
    fn test_custom_chained_conversions() {
        let custom = CustomTransformer { multiplier: 2 };

        // Convert to Box, then convert to Rc
        let boxed = custom.into_box();
        assert_eq!(boxed.apply(10), 20);

        // Create another custom transformer for the next test
        let custom2 = CustomTransformer { multiplier: 4 };
        let rc = custom2.into_rc();
        let boxed2 = rc.into_box();
        assert_eq!(boxed2.apply(5), 20);
    }

    #[test]
    fn test_custom_with_different_types() {
        // A custom transformer that converts i32 to String
        struct IntToString;

        impl Transformer<i32, String> for IntToString {
            fn apply(&self, input: i32) -> String {
                format!("Number: {}", input)
            }
        }

        let custom = IntToString;
        let boxed = custom.into_box();

        assert_eq!(boxed.apply(42), "Number: 42");
        assert_eq!(boxed.apply(100), "Number: 100");
    }

    #[test]
    fn test_custom_composition_with_default() {
        let custom = CustomTransformer { multiplier: 3 };
        let boxed = custom.into_box();

        // Compose with another transformer
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let composed = boxed.and_then(to_string);

        assert_eq!(composed.apply(7), "21");
    }

    #[test]
    fn test_custom_into_arc_uses_default() {
        // A thread-safe custom transformer that implements Send + Sync
        #[derive(Clone)]
        struct ThreadSafeTransformer {
            multiplier: i32,
        }

        impl Transformer<i32, i32> for ThreadSafeTransformer {
            fn apply(&self, input: i32) -> i32 {
                input * self.multiplier
            }
        }

        let custom = ThreadSafeTransformer { multiplier: 4 };
        let arc = custom.into_arc();

        // Test that the ArcTransformer works correctly with the
        // default implementation
        assert_eq!(arc.apply(5), 20);

        // Test that cloning works
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.apply(10), 40);

        // Test thread safety
        let arc2 = arc.clone();
        let handle = thread::spawn(move || arc2.apply(7));
        assert_eq!(handle.join().unwrap(), 28);
        assert_eq!(arc.apply(3), 12);
    }
}

// ============================================================================
// Specialized into_fn Implementation Tests
// ============================================================================

#[cfg(test)]
mod specialized_into_fn_tests {
    use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};
    use std::thread;

    #[test]
    fn test_box_transformer_into_fn_optimized() {
        // Test that BoxTransformer::into_fn uses the optimized
        // implementation that unwraps the Box directly
        let double = BoxTransformer::new(|x: i32| x * 2);
        let func = double.into_fn();

        assert_eq!(func(21), 42);
        assert_eq!(func(10), 20);
        assert_eq!(func(0), 0);
    }

    #[test]
    fn test_box_transformer_into_fn_with_closure_capture() {
        // Test that the optimized implementation works with closures
        // that capture variables
        let multiplier = 3;
        let multiply = BoxTransformer::new(move |x: i32| x * multiplier);
        let func = multiply.into_fn();

        assert_eq!(func(7), 21);
        assert_eq!(func(10), 30);
    }

    #[test]
    fn test_box_transformer_into_fn_type_conversion() {
        // Test that the optimized implementation works with type
        // conversions
        let to_string = BoxTransformer::new(|x: i32| x.to_string());
        let func = to_string.into_fn();

        assert_eq!(func(42), "42");
        assert_eq!(func(100), "100");
    }

    #[test]
    fn test_arc_transformer_into_fn_optimized() {
        // Test that ArcTransformer::into_fn uses the optimized
        // implementation
        let double = ArcTransformer::new(|x: i32| x * 2);
        let func = double.into_fn();

        assert_eq!(func(21), 42);
        assert_eq!(func(10), 20);
    }

    #[test]
    fn test_arc_transformer_into_fn_clone_and_convert() {
        // Test that we can clone an ArcTransformer and convert the
        // clone to a function
        let double = ArcTransformer::new(|x: i32| x * 2);
        let double_clone = double.clone();

        let func = double_clone.into_fn();
        assert_eq!(func(21), 42);

        // Original still usable
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_arc_transformer_into_fn_thread_safety() {
        // Test that the converted function maintains thread safety
        let square = ArcTransformer::new(|x: i32| x * x);
        let func = square.into_fn();

        let handle = thread::spawn(move || func(7));
        assert_eq!(handle.join().unwrap(), 49);
    }

    #[test]
    fn test_rc_transformer_into_fn_optimized() {
        // Test that RcTransformer::into_fn uses the optimized
        // implementation
        let triple = RcTransformer::new(|x: i32| x * 3);
        let func = triple.into_fn();

        assert_eq!(func(7), 21);
        assert_eq!(func(10), 30);
    }

    #[test]
    fn test_rc_transformer_into_fn_clone_and_convert() {
        // Test that we can clone an RcTransformer and convert the
        // clone to a function
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let add_one_clone = add_one.clone();

        let func = add_one_clone.into_fn();
        assert_eq!(func(41), 42);

        // Original still usable
        assert_eq!(add_one.apply(99), 100);
    }

    #[test]
    fn test_rc_transformer_into_fn_with_shared_state() {
        // Test that multiple clones can be converted to functions
        let negate = RcTransformer::new(|x: i32| -x);
        let clone1 = negate.clone();
        let clone2 = negate.clone();

        let func1 = clone1.into_fn();
        let func2 = clone2.into_fn();

        assert_eq!(func1(42), -42);
        assert_eq!(func2(100), -100);
        assert_eq!(negate.apply(7), -7);
    }

    #[test]
    fn test_closure_into_fn_zero_cost() {
        // Test that closure::into_fn is a zero-cost abstraction that
        // returns the closure itself
        let double = |x: i32| x * 2;
        let func = double.into_fn();

        assert_eq!(func(21), 42);
        assert_eq!(func(10), 20);
    }

    #[test]
    fn test_closure_into_fn_with_capture() {
        // Test that closure::into_fn works with captured variables
        let base = 10;
        let add_base = move |x: i32| x + base;
        let func = add_base.into_fn();

        assert_eq!(func(5), 15);
        assert_eq!(func(32), 42);
    }

    #[test]
    fn test_closure_into_fn_composition() {
        // Test that closures converted to functions can be composed
        let double = |x: i32| x * 2;
        let add_one = |x: i32| x + 1;

        let func1 = double.into_fn();
        let func2 = add_one.into_fn();

        assert_eq!(func2(func1(5)), 11); // (5 * 2) + 1
    }

    #[test]
    fn test_function_pointer_into_fn() {
        // Test that function pointers work with into_fn
        fn square(x: i32) -> i32 {
            x * x
        }

        let func = square.into_fn();
        assert_eq!(func(5), 25);
        assert_eq!(func(7), 49);
    }

    #[test]
    fn test_into_fn_multiple_calls() {
        // Test that the returned function can be called multiple
        // times
        let factorial = BoxTransformer::new(|n: u32| {
            (1..=n).product::<u32>()
        });
        let func = factorial.into_fn();

        assert_eq!(func(0), 1);
        assert_eq!(func(1), 1);
        assert_eq!(func(5), 120);
        assert_eq!(func(5), 120); // Call again with same value
    }

    #[test]
    fn test_into_fn_with_complex_types() {
        // Test into_fn with more complex input/output types
        let parse = ArcTransformer::new(|s: String| s.parse::<i32>());
        let func = parse.into_fn();

        assert_eq!(func("42".to_string()), Ok(42));
        assert!(func("abc".to_string()).is_err());
    }

    #[test]
    fn test_into_fn_preserves_behavior() {
        // Test that into_fn preserves the exact behavior of the
        // original transformer
        let transformer = RcTransformer::new(|x: i32| {
            if x > 0 {
                x * 2
            } else {
                x * 3
            }
        });

        let original_result1 = transformer.apply(5);
        let original_result2 = transformer.apply(-5);

        let func = transformer.into_fn();

        assert_eq!(func(5), original_result1);
        assert_eq!(func(-5), original_result2);
    }
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[cfg(test)]
mod type_conversion_tests {
    use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};

    #[test]
    fn test_box_into_box() {
        let add = BoxTransformer::new(|x: i32| x + 10);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20), 30);
    }

    #[test]
    fn test_box_into_rc() {
        let add = BoxTransformer::new(|x: i32| x + 10);
        let rc = add.into_rc();
        assert_eq!(rc.apply(20), 30);
    }

    #[test]
    fn test_arc_into_arc() {
        let add = ArcTransformer::new(|x: i32| x + 10);
        let arc = add.into_arc();
        assert_eq!(arc.apply(20), 30);
    }

    #[test]
    fn test_arc_into_fn() {
        let add = ArcTransformer::new(|x: i32| x + 10);
        let func = add.into_fn();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_rc_into_rc() {
        let add = RcTransformer::new(|x: i32| x + 10);
        let rc = add.into_rc();
        assert_eq!(rc.apply(20), 30);
    }

    #[test]
    fn test_rc_into_fn() {
        let add = RcTransformer::new(|x: i32| x + 10);
        let func = add.into_fn();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_box_into_fn() {
        let add = BoxTransformer::new(|x: i32| x + 10);
        let func = add.into_fn();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_arc_into_box() {
        let add = ArcTransformer::new(|x: i32| x + 10);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20), 30);
    }

    #[test]
    fn test_arc_into_rc() {
        let add = ArcTransformer::new(|x: i32| x + 10);
        let rc = add.into_rc();
        assert_eq!(rc.apply(20), 30);
    }

    #[test]
    fn test_rc_into_box() {
        let add = RcTransformer::new(|x: i32| x + 10);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20), 30);
    }

    #[test]
    fn test_arc_constant_with_clone() {
        let constant = ArcTransformer::constant(42);
        assert_eq!(constant.apply(1), 42);
        assert_eq!(constant.apply(2), 42);
        assert_eq!(constant.apply(3), 42);
    }

    #[test]
    fn test_rc_constant_with_clone() {
        let constant = RcTransformer::constant("test");
        assert_eq!(constant.apply(1), "test");
        assert_eq!(constant.apply(2), "test");
        assert_eq!(constant.apply(3), "test");
    }
}

// ============================================================================
// Transformer Default Implementation Tests - to_xxx() methods
// ============================================================================

#[cfg(test)]
mod transformer_default_to_methods_tests {
    use prism3_function::{ArcTransformer, RcTransformer, Transformer};
    use std::thread;

    // ========================================================================
    // ArcTransformer::to_box() Tests
    // ========================================================================

    #[test]
    fn test_arc_to_box_basic() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let boxed = double.to_box();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的 BoxTransformer 也可用
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_arc_to_box_multiple_conversions() {
        let triple = ArcTransformer::new(|x: i32| x * 3);
        let boxed1 = triple.to_box();
        let boxed2 = triple.to_box();

        // 多次转换都可以工作
        assert_eq!(boxed1.apply(7), 21);
        assert_eq!(boxed2.apply(7), 21);
        assert_eq!(triple.apply(7), 21);
    }

    #[test]
    fn test_arc_to_box_with_string() {
        let length = ArcTransformer::new(|s: String| s.len());
        let boxed = length.to_box();

        assert_eq!(length.apply("hello".to_string()), 5);
        assert_eq!(boxed.apply("world".to_string()), 5);
    }

    #[test]
    fn test_arc_to_box_with_captured_variable() {
        let multiplier = 5;
        let multiply = ArcTransformer::new(move |x: i32| x * multiplier);
        let boxed = multiply.to_box();

        assert_eq!(multiply.apply(8), 40);
        assert_eq!(boxed.apply(8), 40);
    }

    // ========================================================================
    // ArcTransformer::to_rc() Tests
    // ========================================================================

    #[test]
    fn test_arc_to_rc_basic() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let rc = double.to_rc();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的 RcTransformer 也可用
        assert_eq!(rc.apply(21), 42);
    }

    #[test]
    fn test_arc_to_rc_multiple_conversions() {
        let add_ten = ArcTransformer::new(|x: i32| x + 10);
        let rc1 = add_ten.to_rc();
        let rc2 = add_ten.to_rc();

        assert_eq!(rc1.apply(5), 15);
        assert_eq!(rc2.apply(5), 15);
        assert_eq!(add_ten.apply(5), 15);
    }

    #[test]
    fn test_arc_to_rc_clone_conversion() {
        let negate = ArcTransformer::new(|x: i32| -x);
        let rc = negate.to_rc();
        let rc_clone = rc.clone();

        assert_eq!(negate.apply(42), -42);
        assert_eq!(rc.apply(42), -42);
        assert_eq!(rc_clone.apply(42), -42);
    }

    // ========================================================================
    // ArcTransformer::to_arc() Tests
    // ========================================================================

    #[test]
    fn test_arc_to_arc_basic() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let arc = double.to_arc();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的 ArcTransformer 也可用
        assert_eq!(arc.apply(21), 42);
    }

    #[test]
    fn test_arc_to_arc_is_clone() {
        let add_one = ArcTransformer::new(|x: i32| x + 1);
        let arc = add_one.to_arc();

        // to_arc() 应该等价于 clone()
        assert_eq!(add_one.apply(41), 42);
        assert_eq!(arc.apply(41), 42);
    }

    #[test]
    fn test_arc_to_arc_thread_safe() {
        let increment = ArcTransformer::new(|x: i32| x + 1);
        let arc = increment.to_arc();

        let handle = thread::spawn(move || {
            arc.apply(99)
        });

        assert_eq!(handle.join().unwrap(), 100);
        assert_eq!(increment.apply(41), 42);
    }

    // ========================================================================
    // ArcTransformer::to_fn() Tests
    // ========================================================================

    #[test]
    fn test_arc_to_fn_basic() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let func = double.to_fn();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的函数也可用
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let square = ArcTransformer::new(|x: i32| x * x);
        let func = square.to_fn();

        assert_eq!(func(5), 25);
        assert_eq!(func(7), 49);
        assert_eq!(square.apply(3), 9);
    }

    #[test]
    fn test_arc_to_fn_with_closure_composition() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let func = double.to_fn();

        let add_ten = |x: i32| x + 10;
        let result = add_ten(func(5)); // (5 * 2) + 10
        assert_eq!(result, 20);
    }

    // ========================================================================
    // RcTransformer::to_box() Tests
    // ========================================================================

    #[test]
    fn test_rc_to_box_basic() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let boxed = double.to_box();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的 BoxTransformer 也可用
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_rc_to_box_multiple_conversions() {
        let subtract = RcTransformer::new(|x: i32| x - 5);
        let boxed1 = subtract.to_box();
        let boxed2 = subtract.to_box();

        assert_eq!(boxed1.apply(15), 10);
        assert_eq!(boxed2.apply(15), 10);
        assert_eq!(subtract.apply(15), 10);
    }

    #[test]
    fn test_rc_to_box_with_clone() {
        let negate = RcTransformer::new(|x: i32| -x);
        let rc_clone = negate.clone();
        let boxed = rc_clone.to_box();

        assert_eq!(negate.apply(42), -42);
        assert_eq!(rc_clone.apply(42), -42);
        assert_eq!(boxed.apply(42), -42);
    }

    // ========================================================================
    // RcTransformer::to_rc() Tests
    // ========================================================================

    #[test]
    fn test_rc_to_rc_basic() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let rc = double.to_rc();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的 RcTransformer 也可用
        assert_eq!(rc.apply(21), 42);
    }

    #[test]
    fn test_rc_to_rc_is_clone() {
        let add_one = RcTransformer::new(|x: i32| x + 1);
        let rc = add_one.to_rc();

        // to_rc() 应该等价于 clone()
        assert_eq!(add_one.apply(41), 42);
        assert_eq!(rc.apply(41), 42);
    }

    #[test]
    fn test_rc_to_rc_multiple_clones() {
        let triple = RcTransformer::new(|x: i32| x * 3);
        let rc1 = triple.to_rc();
        let rc2 = triple.to_rc();
        let rc1_clone = rc1.clone();

        assert_eq!(triple.apply(7), 21);
        assert_eq!(rc1.apply(7), 21);
        assert_eq!(rc2.apply(7), 21);
        assert_eq!(rc1_clone.apply(7), 21);
    }

    // ========================================================================
    // RcTransformer::to_fn() Tests
    // ========================================================================

    #[test]
    fn test_rc_to_fn_basic() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let func = double.to_fn();

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(21), 42);
        // 转换后的函数也可用
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_rc_to_fn_multiple_calls() {
        let abs = RcTransformer::new(|x: i32| x.abs());
        let func = abs.to_fn();

        assert_eq!(func(-5), 5);
        assert_eq!(func(5), 5);
        assert_eq!(abs.apply(-10), 10);
    }

    #[test]
    fn test_rc_to_fn_with_captured_state() {
        let offset = 100;
        let add_offset = RcTransformer::new(move |x: i32| x + offset);
        let func = add_offset.to_fn();

        assert_eq!(func(42), 142);
        assert_eq!(add_offset.apply(42), 142);
    }

    // ========================================================================
    // Cross-type conversion tests
    // ========================================================================

    #[test]
    fn test_arc_to_box_to_fn_chain() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let boxed = double.to_box();
        let func = boxed.into_fn();

        assert_eq!(func(21), 42);
        // 原始 ArcTransformer 仍然可用
        assert_eq!(double.apply(21), 42);
    }

    #[test]
    fn test_arc_to_rc_to_box_chain() {
        let triple = ArcTransformer::new(|x: i32| x * 3);
        let rc = triple.to_rc();
        let boxed = rc.to_box();

        assert_eq!(boxed.apply(7), 21);
        assert_eq!(rc.apply(7), 21);
        assert_eq!(triple.apply(7), 21);
    }

    #[test]
    fn test_rc_to_box_composition() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;

        let boxed = double.to_box();
        let composed = boxed.and_then(add_one);

        assert_eq!(composed.apply(5), 11); // (5 * 2) + 1
        assert_eq!(double.apply(5), 10);
    }

    // ========================================================================
    // Complex type tests
    // ========================================================================

    #[test]
    fn test_arc_to_box_with_option() {
        let parse = ArcTransformer::new(|s: String| s.parse::<i32>().ok());
        let boxed = parse.to_box();

        assert_eq!(parse.apply("42".to_string()), Some(42));
        assert_eq!(boxed.apply("42".to_string()), Some(42));
        assert_eq!(boxed.apply("abc".to_string()), None);
    }

    #[test]
    fn test_rc_to_fn_with_result() {
        let divide = RcTransformer::new(|x: i32| {
            if x == 0 {
                Err("Division by zero")
            } else {
                Ok(100 / x)
            }
        });
        let func = divide.to_fn();

        assert_eq!(func(10), Ok(10));
        assert_eq!(func(0), Err("Division by zero"));
        assert_eq!(divide.apply(5), Ok(20));
    }

    #[test]
    fn test_arc_to_rc_with_vec() {
        let sort = ArcTransformer::new(|mut v: Vec<i32>| {
            v.sort();
            v
        });
        let rc = sort.to_rc();

        let input1 = vec![3, 1, 4, 1, 5, 9];
        let input2 = vec![3, 1, 4, 1, 5, 9];

        assert_eq!(sort.apply(input1), vec![1, 1, 3, 4, 5, 9]);
        assert_eq!(rc.apply(input2), vec![1, 1, 3, 4, 5, 9]);
    }

    // ========================================================================
    // Identity and constant tests
    // ========================================================================

    #[test]
    fn test_arc_identity_to_box() {
        let identity = ArcTransformer::<i32, i32>::identity();
        let boxed = identity.to_box();

        assert_eq!(identity.apply(42), 42);
        assert_eq!(boxed.apply(42), 42);
    }

    #[test]
    fn test_rc_constant_to_fn() {
        let constant = RcTransformer::constant("fixed");
        let func = constant.to_fn();

        assert_eq!(func(1), "fixed");
        assert_eq!(func(2), "fixed");
        assert_eq!(constant.apply(3), "fixed");
    }

    #[test]
    fn test_arc_constant_to_rc() {
        let constant = ArcTransformer::constant(123);
        let rc = constant.to_rc();

        assert_eq!(constant.apply(999), 123);
        assert_eq!(rc.apply(999), 123);
    }
}

// ============================================================================
// Custom Type with Default to_xxx Implementation Tests
// ============================================================================

#[cfg(test)]
mod custom_transformer_to_methods_tests {
    use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};
    use std::thread;

    /// 自定义 Transformer 实现，用于测试默认的 to_xxx 方法
    /// 这是一个简单的乘法 transformer
    #[derive(Clone)]
    struct MultiplyTransformer {
        multiplier: i32,
    }

    impl Transformer<i32, i32> for MultiplyTransformer {
        fn apply(&self, input: i32) -> i32 {
            input * self.multiplier
        }
    }

    /// 线程安全的自定义 Transformer 实现
    #[derive(Clone)]
    struct ThreadSafeMultiplyTransformer {
        multiplier: i32,
    }

    // 手动实现 Send + Sync
    unsafe impl Send for ThreadSafeMultiplyTransformer {}
    unsafe impl Sync for ThreadSafeMultiplyTransformer {}

    impl Transformer<i32, i32> for ThreadSafeMultiplyTransformer {
        fn apply(&self, input: i32) -> i32 {
            input * self.multiplier
        }
    }

    // ========================================================================
    // Custom Type to_box() Tests
    // ========================================================================

    #[test]
    fn test_custom_to_box_basic() {
        let multiply = MultiplyTransformer { multiplier: 3 };
        let boxed = multiply.to_box();

        // 原始 transformer 仍然可用
        assert_eq!(multiply.apply(7), 21);
        // 转换后的 BoxTransformer 也可用
        assert_eq!(boxed.apply(7), 21);
    }

    #[test]
    fn test_custom_to_box_multiple_conversions() {
        let multiply = MultiplyTransformer { multiplier: 5 };
        let boxed1 = multiply.to_box();
        let boxed2 = multiply.to_box();

        // 多次转换都可以工作
        assert_eq!(boxed1.apply(4), 20);
        assert_eq!(boxed2.apply(4), 20);
        assert_eq!(multiply.apply(4), 20);
    }

    #[test]
    fn test_custom_to_box_with_composition() {
        let multiply = MultiplyTransformer { multiplier: 2 };
        let boxed = multiply.to_box();

        // 与其他 transformer 组合
        let add_ten = BoxTransformer::new(|x: i32| x + 10);
        let composed = boxed.and_then(add_ten);

        assert_eq!(composed.apply(5), 20); // (5 * 2) + 10
    }

    // ========================================================================
    // Custom Type to_rc() Tests
    // ========================================================================

    #[test]
    fn test_custom_to_rc_basic() {
        let multiply = MultiplyTransformer { multiplier: 4 };
        let rc = multiply.to_rc();

        // 原始 transformer 仍然可用
        assert_eq!(multiply.apply(5), 20);
        // 转换后的 RcTransformer 也可用
        assert_eq!(rc.apply(5), 20);
    }

    #[test]
    fn test_custom_to_rc_multiple_conversions() {
        let multiply = MultiplyTransformer { multiplier: 7 };
        let rc1 = multiply.to_rc();
        let rc2 = multiply.to_rc();

        assert_eq!(rc1.apply(3), 21);
        assert_eq!(rc2.apply(3), 21);
        assert_eq!(multiply.apply(3), 21);
    }

    #[test]
    fn test_custom_to_rc_clone_and_use() {
        let multiply = MultiplyTransformer { multiplier: 6 };
        let rc = multiply.to_rc();
        let rc_clone = rc.clone();

        assert_eq!(multiply.apply(4), 24);
        assert_eq!(rc.apply(4), 24);
        assert_eq!(rc_clone.apply(4), 24);
    }

    #[test]
    fn test_custom_to_rc_with_composition() {
        let multiply = MultiplyTransformer { multiplier: 3 };
        let rc = multiply.to_rc();

        // 与其他 transformer 组合
        let square = RcTransformer::new(|x: i32| x * x);
        let composed = rc.and_then(square);

        assert_eq!(composed.apply(5), 225); // (5 * 3)^2 = 225
    }

    // ========================================================================
    // Custom Type to_arc() Tests (Thread-Safe)
    // ========================================================================

    #[test]
    fn test_custom_to_arc_basic() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 5 };
        let arc = multiply.to_arc();

        // 原始 transformer 仍然可用
        assert_eq!(multiply.apply(8), 40);
        // 转换后的 ArcTransformer 也可用
        assert_eq!(arc.apply(8), 40);
    }

    #[test]
    fn test_custom_to_arc_multiple_conversions() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 9 };
        let arc1 = multiply.to_arc();
        let arc2 = multiply.to_arc();

        assert_eq!(arc1.apply(2), 18);
        assert_eq!(arc2.apply(2), 18);
        assert_eq!(multiply.apply(2), 18);
    }

    #[test]
    fn test_custom_to_arc_clone_and_use() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 10 };
        let arc = multiply.to_arc();
        let arc_clone = arc.clone();

        assert_eq!(multiply.apply(3), 30);
        assert_eq!(arc.apply(3), 30);
        assert_eq!(arc_clone.apply(3), 30);
    }

    #[test]
    fn test_custom_to_arc_thread_safe() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 7 };
        let arc = multiply.to_arc();

        let handle = thread::spawn(move || {
            arc.apply(6)
        });

        assert_eq!(handle.join().unwrap(), 42);
        assert_eq!(multiply.apply(6), 42);
    }

    #[test]
    fn test_custom_to_arc_with_composition() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 4 };
        let arc = multiply.to_arc();

        // 与其他 transformer 组合
        let double = ArcTransformer::new(|x: i32| x * 2);
        let composed = arc.and_then(double);

        assert_eq!(composed.apply(5), 40); // (5 * 4) * 2 = 40
    }

    // ========================================================================
    // Custom Type to_fn() Tests
    // ========================================================================

    #[test]
    fn test_custom_to_fn_basic() {
        let multiply = MultiplyTransformer { multiplier: 8 };
        let func = multiply.to_fn();

        // 原始 transformer 仍然可用
        assert_eq!(multiply.apply(5), 40);
        // 转换后的函数也可用
        assert_eq!(func(5), 40);
    }

    #[test]
    fn test_custom_to_fn_multiple_calls() {
        let multiply = MultiplyTransformer { multiplier: 6 };
        let func = multiply.to_fn();

        assert_eq!(func(3), 18);
        assert_eq!(func(7), 42);
        assert_eq!(multiply.apply(10), 60);
    }

    #[test]
    fn test_custom_to_fn_with_closure_composition() {
        let multiply = MultiplyTransformer { multiplier: 5 };
        let func = multiply.to_fn();

        let add_five = |x: i32| x + 5;
        let result = add_five(func(4)); // (4 * 5) + 5 = 25
        assert_eq!(result, 25);
    }

    #[test]
    fn test_custom_to_fn_multiple_conversions() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 9 };
        let func1 = multiply.to_fn();
        let func2 = multiply.to_fn();

        // 多次转换都可以工作
        assert_eq!(func1(5), 45);
        assert_eq!(func2(5), 45);
        assert_eq!(multiply.apply(5), 45);
    }

    // ========================================================================
    // Cross-type conversion tests
    // ========================================================================

    #[test]
    fn test_custom_to_box_to_rc_chain() {
        let multiply = MultiplyTransformer { multiplier: 3 };
        let boxed = multiply.to_box();
        let rc = boxed.into_rc();

        assert_eq!(rc.apply(7), 21);
        // 原始自定义 transformer 仍然可用
        assert_eq!(multiply.apply(7), 21);
    }

    #[test]
    fn test_custom_to_rc_to_box_chain() {
        let multiply = MultiplyTransformer { multiplier: 4 };
        let rc = multiply.to_rc();
        let boxed = rc.to_box();

        assert_eq!(boxed.apply(6), 24);
        assert_eq!(rc.apply(6), 24);
        assert_eq!(multiply.apply(6), 24);
    }

    #[test]
    fn test_custom_to_arc_to_rc_chain() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 5 };
        let arc = multiply.to_arc();
        let rc = arc.to_rc();

        assert_eq!(rc.apply(8), 40);
        assert_eq!(arc.apply(8), 40);
        assert_eq!(multiply.apply(8), 40);
    }

    #[test]
    fn test_custom_multiple_to_conversions() {
        let multiply = ThreadSafeMultiplyTransformer { multiplier: 2 };

        let boxed = multiply.to_box();
        let rc = multiply.to_rc();
        let arc = multiply.to_arc();
        let func = multiply.to_fn();

        // 所有转换后的类型都可以正常工作
        assert_eq!(multiply.apply(10), 20);
        assert_eq!(boxed.apply(10), 20);
        assert_eq!(rc.apply(10), 20);
        assert_eq!(arc.apply(10), 20);
        assert_eq!(func(10), 20);
    }

    // ========================================================================
    // Different type transformation tests
    // ========================================================================

    /// 自定义 Transformer: i32 -> String
    #[derive(Clone)]
    struct IntToStringTransformer {
        prefix: String,
    }

    impl Transformer<i32, String> for IntToStringTransformer {
        fn apply(&self, input: i32) -> String {
            format!("{}{}", self.prefix, input)
        }
    }

    #[test]
    fn test_custom_different_types_to_box() {
        let transformer = IntToStringTransformer {
            prefix: "Number: ".to_string(),
        };
        let boxed = transformer.to_box();

        assert_eq!(transformer.apply(42), "Number: 42");
        assert_eq!(boxed.apply(42), "Number: 42");
    }

    #[test]
    fn test_custom_different_types_to_rc() {
        let transformer = IntToStringTransformer {
            prefix: "Value: ".to_string(),
        };
        let rc = transformer.to_rc();

        assert_eq!(transformer.apply(100), "Value: 100");
        assert_eq!(rc.apply(100), "Value: 100");
    }

    #[test]
    fn test_custom_different_types_to_fn() {
        let transformer = IntToStringTransformer {
            prefix: "Result: ".to_string(),
        };
        let func = transformer.to_fn();

        assert_eq!(transformer.apply(999), "Result: 999");
        assert_eq!(func(999), "Result: 999");
    }

    // ========================================================================
    // Complex state tests
    // ========================================================================

    /// 带有复杂状态的自定义 Transformer
    #[derive(Clone)]
    struct StatefulTransformer {
        base: i32,
        multiplier: i32,
        offset: i32,
    }

    impl Transformer<i32, i32> for StatefulTransformer {
        fn apply(&self, input: i32) -> i32 {
            (input + self.base) * self.multiplier + self.offset
        }
    }

    #[test]
    fn test_stateful_to_box() {
        let transformer = StatefulTransformer {
            base: 5,
            multiplier: 3,
            offset: 10,
        };
        let boxed = transformer.to_box();

        // (10 + 5) * 3 + 10 = 55
        assert_eq!(transformer.apply(10), 55);
        assert_eq!(boxed.apply(10), 55);
    }

    #[test]
    fn test_stateful_to_rc() {
        let transformer = StatefulTransformer {
            base: 2,
            multiplier: 4,
            offset: 1,
        };
        let rc = transformer.to_rc();

        // (5 + 2) * 4 + 1 = 29
        assert_eq!(transformer.apply(5), 29);
        assert_eq!(rc.apply(5), 29);
    }

    #[test]
    fn test_stateful_to_fn() {
        let transformer = StatefulTransformer {
            base: 1,
            multiplier: 2,
            offset: 3,
        };
        let func = transformer.to_fn();

        // (7 + 1) * 2 + 3 = 19
        assert_eq!(transformer.apply(7), 19);
        assert_eq!(func(7), 19);
    }

    #[test]
    fn test_stateful_all_conversions() {
        let transformer = StatefulTransformer {
            base: 3,
            multiplier: 2,
            offset: 5,
        };

        let boxed = transformer.to_box();
        let rc = transformer.to_rc();
        let func = transformer.to_fn();

        // (6 + 3) * 2 + 5 = 23
        let expected = 23;
        assert_eq!(transformer.apply(6), expected);
        assert_eq!(boxed.apply(6), expected);
        assert_eq!(rc.apply(6), expected);
        assert_eq!(func(6), expected);
    }
}
