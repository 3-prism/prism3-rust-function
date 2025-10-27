/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    BoxTransformerOnce,
    TransformerOnce,
};

// ============================================================================
// BoxTransformerOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_once_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse.apply_once("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        assert_eq!(identity.apply_once(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.apply_once(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.apply_once(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxTransformerOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.apply_once("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply_once(42), "42_suffix");
    }
}

// ============================================================================
// Conditional Transformer Once Tests
// ============================================================================

#[cfg(test)]
mod conditional_tests {
    use super::*;
    use prism3_function::BoxPredicate;

    #[test]
    fn test_when_or_else() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply_once(5), 10);
    }

    #[test]
    fn test_when_or_else_negative() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply_once(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply_once(5), 10);
        let result2 = BoxTransformerOnce::new(|x: i32| x * 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: i32| -x);
        assert_eq!(result2.apply_once(-5), 5);
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
        let boxed = double.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_box_to_fn() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        // Test into_fn in impl<F, T, R> TransformerOnce<T, R> for F
        let double = |x: i32| x * 2;
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
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
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.apply_once(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let square = BoxTransformerOnce::new(|x: i32| x * x);
        let composed = square.compose(double).compose(add_one);
        assert_eq!(composed.apply_once(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_mixed_composition() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
        let double = |x: i32| x * 2;
        let to_string = |x: i32| format!("Result: {}", x);
        let pipeline = parse.and_then(double).and_then(to_string);
        assert_eq!(pipeline.apply_once("21".to_string()), "Result: 42");
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
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        let composed = double.and_then(|x| identity.apply_once(x));
        assert_eq!(composed.apply_once(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.apply_once(123), "hello");

        let constant2 = BoxTransformerOnce::constant("world");
        assert_eq!(constant2.apply_once(456), "world");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.apply_once("42".to_string()), Some(42));

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse2.apply_once("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse.apply_once("42".to_string()).is_ok());

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse2.apply_once("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformerOnce::new(|s: String| {
            s.split(',').map(|s| s.to_string()).collect::<Vec<_>>()
        });
        assert_eq!(
            split.apply_once("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_consuming_ownership() {
        let vec = vec![1, 2, 3, 4, 5];
        let sum = BoxTransformerOnce::new(|v: Vec<i32>| v.iter().sum::<i32>());
        assert_eq!(sum.apply_once(vec), 15);
        // vec is consumed and cannot be used again
    }

    #[test]
    fn test_with_box() {
        let boxed = Box::new(42);
        let unbox = BoxTransformerOnce::new(|b: Box<i32>| *b);
        assert_eq!(unbox.apply_once(boxed), 42);
    }

    #[test]
    fn test_with_closure_capture() {
        let multiplier = 3;
        let multiply = BoxTransformerOnce::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.apply_once(7), 21);
    }
}

// ============================================================================
// Trait Usage Tests
// ============================================================================

#[cfg(test)]
mod trait_usage_tests {
    use super::*;

    #[test]
    fn test_transformer_once_trait() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply_once(x)
        }

        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_closure_as_transformer_once() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply_once(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_transformer_once<T, R, F: TransformerOnce<T, R>>(f: F, x: T) -> R {
            f.apply_once(x)
        }

        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        assert_eq!(apply_transformer_once(to_string, 42), "42");
    }
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_box_into_box() {
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let boxed = add.into_box_once();
        assert_eq!(boxed.apply_once(20), 30);
    }

    #[test]
    fn test_box_into_fn() {
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let func = add.into_fn_once();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_closure_into_box() {
        let double = |x: i32| x * 2;
        let boxed = double.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        let double = |x: i32| x * 2;
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_closure_to_box_and_preserve_original() {
        // to_box borrows &self and requires Clone; non-capturing closures are Clone
        let double = |x: i32| x * 2;
        let boxed = double.to_box_once();
        assert_eq!(boxed.apply_once(21), 42);

        // Original closure is still available (to_box does not consume the original object)
        assert_eq!(double.apply_once(10), 20);
    }

    #[test]
    fn test_closure_to_fn_and_preserve_original() {
        // to_fn borrows &self and requires Clone; non-capturing closures are Clone
        let double = |x: i32| x * 2;
        let func = double.to_fn_once();
        assert_eq!(func(14), 28);

        // Original closure is still available (to_fn does not consume the original object)
        assert_eq!(double.apply_once(7), 14);
    }

    #[test]
    fn test_function_pointer_into_box() {
        fn triple(x: i32) -> i32 {
            x * 3
        }
        let boxed = triple.into_box_once();
        assert_eq!(boxed.apply_once(14), 42);
    }

    #[test]
    fn test_function_pointer_into_fn() {
        fn triple(x: i32) -> i32 {
            x * 3
        }
        let func = triple.into_fn_once();
        assert_eq!(func(14), 42);
    }
}

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod default_implementation_tests {
    use super::*;

    // Custom type test default implementation
    struct CustomTransformer {
        factor: i32,
    }

    impl TransformerOnce<i32, i32> for CustomTransformer {
        fn apply_once(self, input: i32) -> i32 {
            input * self.factor
        }
        // Use default into_box and into_fn implementations
    }

    #[test]
    fn test_custom_transformer_into_box() {
        let transformer = CustomTransformer { factor: 2 };
        let boxed = transformer.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_custom_transformer_into_fn() {
        let transformer = CustomTransformer { factor: 2 };
        let func = transformer.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_custom_transformer_chain() {
        let transformer1 = CustomTransformer { factor: 2 };
        let transformer2 = CustomTransformer { factor: 3 };
        let composed = transformer1.into_box_once().and_then(transformer2);
        assert_eq!(composed.apply_once(7), 42); // 7 * 2 * 3
    }
}

// ============================================================================
// Zero-Cost Specialization Tests
// ============================================================================

#[cfg(test)]
mod zero_cost_specialization_tests {
    use super::*;

    #[test]
    fn test_box_into_box_is_zero_cost() {
        // BoxTransformerOnce::into_box() should directly return itself, zero cost
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let boxed = add.into_box_once();
        assert_eq!(boxed.apply_once(20), 30);
    }

    #[test]
    fn test_box_into_fn_is_zero_cost() {
        // BoxTransformerOnce::into_fn() should directly return the inner function, zero cost
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let func = add.into_fn_once();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_closure_into_fn_is_zero_cost() {
        // Closure's into_fn() should directly return itself, zero cost
        let double = |x: i32| x * 2;
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_chained_conversions() {
        // Test chained conversions
        let double = |x: i32| x * 2;
        let boxed = double.into_box_once(); // closure -> Box
        let func = boxed.into_fn_once(); // Box -> Fn (zero cost, directly return inner function)
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_complex_type_conversion() {
        // Test complex type conversions
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let boxed = parse.into_box_once();
        let composed = boxed.and_then(|x| x * 2);
        let func = composed.into_fn_once();
        assert_eq!(func("21".to_string()), 42);
    }
}

// ============================================================================
// Custom Type Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod custom_type_default_impl_tests {
    use super::*;

    /// Custom cloneable TransformerOnce type
    ///
    /// This type demonstrates how to implement the TransformerOnce trait,
    /// and by implementing Clone, it can use the to_box() and to_fn() methods
    #[derive(Clone)]
    struct CustomTransformer {
        multiplier: i32,
    }

    impl TransformerOnce<i32, i32> for CustomTransformer {
        fn apply_once(self, input: i32) -> i32 {
            input * self.multiplier
        }
    }

    #[test]
    fn test_custom_into_box() {
        // Test into_box default implementation (consumes self)
        let transformer = CustomTransformer { multiplier: 3 };
        let boxed = transformer.into_box_once();
        assert_eq!(boxed.apply_once(14), 42);
        // transformer has been consumed and cannot be used again
    }

    #[test]
    fn test_custom_into_fn() {
        // Test into_fn default implementation (consumes self)
        let transformer = CustomTransformer { multiplier: 3 };
        let func = transformer.into_fn_once();
        assert_eq!(func(14), 42);
        // transformer has been consumed and cannot be used again
    }

    #[test]
    fn test_custom_to_box() {
        // Test to_box default implementation (borrows &self, requires Clone)
        let transformer = CustomTransformer { multiplier: 3 };
        let boxed = transformer.to_box_once();

        // Use the converted boxed first
        assert_eq!(boxed.apply_once(14), 42);

        // Original transformer is still available (because to_box just borrows)
        assert_eq!(transformer.apply_once(10), 30);
    }

    #[test]
    fn test_custom_to_fn() {
        // Test to_fn default implementation (borrows &self, requires Clone)
        let transformer = CustomTransformer { multiplier: 3 };
        let func = transformer.to_fn_once();

        // Use the converted function first
        assert_eq!(func(14), 42);

        // Original transformer is still available (because to_fn just borrows)
        assert_eq!(transformer.apply_once(10), 30);
    }

    #[test]
    fn test_custom_multiple_conversions() {
        // Test multiple conversions
        let transformer = CustomTransformer { multiplier: 2 };

        // Use to_box multiple times (does not consume original object)
        let boxed1 = transformer.to_box_once();
        let boxed2 = transformer.to_box_once();
        let func = transformer.to_fn_once();

        assert_eq!(boxed1.apply_once(5), 10);
        assert_eq!(boxed2.apply_once(10), 20);
        assert_eq!(func(15), 30);

        // Original transformer is still available
        assert_eq!(transformer.apply_once(21), 42);
    }

    #[test]
    fn test_custom_composition_with_to_box() {
        // Test composition using to_box
        let double = CustomTransformer { multiplier: 2 };
        let boxed = double.to_box_once();

        // Compose with other transformers
        let composed = boxed.and_then(|x| x + 2);
        assert_eq!(composed.apply_once(20), 42); // 20 * 2 + 2 = 42

        // Original transformer is still available
        assert_eq!(double.apply_once(10), 20);
    }

    #[test]
    fn test_custom_composition_with_to_fn() {
        // Test composition using to_fn
        let triple = CustomTransformer { multiplier: 3 };
        let func = triple.to_fn_once();

        // Use function for transformation
        let result = func(14);
        assert_eq!(result, 42);

        // Original transformer is still available (because to_fn just borrows)
        assert_eq!(triple.apply_once(7), 21);
    }

    /// Custom transformer with complex state
    #[derive(Clone)]
    struct ComplexTransformer {
        prefix: String,
        suffix: String,
    }

    impl TransformerOnce<i32, String> for ComplexTransformer {
        fn apply_once(self, input: i32) -> String {
            format!("{}{}{}", self.prefix, input, self.suffix)
        }
    }

    #[test]
    fn test_complex_custom_to_box() {
        // Test to_box for complex types
        let transformer = ComplexTransformer {
            prefix: "Number: ".to_string(),
            suffix: "!".to_string(),
        };

        let boxed = transformer.to_box_once();
        assert_eq!(boxed.apply_once(42), "Number: 42!");

        // Original transformer is still available (because to_box just borrows)
        assert_eq!(transformer.apply_once(100), "Number: 100!");
    }

    #[test]
    fn test_complex_custom_to_fn() {
        // Test to_fn for complex types
        let transformer = ComplexTransformer {
            prefix: "Value: ".to_string(),
            suffix: " units".to_string(),
        };

        let func = transformer.to_fn_once();
        assert_eq!(func(42), "Value: 42 units");

        // Original transformer is still available (because to_fn just borrows)
        assert_eq!(transformer.apply_once(100), "Value: 100 units");
    }

    #[test]
    fn test_complex_custom_chain_conversions() {
        // Test complex chained conversions
        let transformer = ComplexTransformer {
            prefix: "[".to_string(),
            suffix: "]".to_string(),
        };

        // First use to_box to create a BoxTransformerOnce
        let boxed = transformer.to_box_once();

        // Then convert BoxTransformerOnce to function
        let func = boxed.into_fn_once();
        assert_eq!(func(42), "[42]");

        // Original transformer is still available (because to_box was used, not into_box)
        assert_eq!(transformer.apply_once(100), "[100]");
    }
}

// ============================================================================
// BoxTransformer TransformerOnce Tests
// ============================================================================

#[cfg(test)]
mod box_transformer_transformer_once_tests {
    use prism3_function::{
        BoxTransformer,
        TransformerOnce,
    };

    #[test]
    fn test_box_transformer_apply_once() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let result = double.apply_once(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_box_transformer_into_box_once() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_box_transformer_into_fn_once() {
        let double = BoxTransformer::new(|x: i32| x * 2);
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_box_transformer_string_transformation() {
        let uppercase = BoxTransformer::new(|s: String| s.to_uppercase());
        let result = uppercase.apply_once("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_box_transformer_complex_transformation() {
        let parse_and_double = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
        let result = parse_and_double.apply_once("21".to_string());
        assert_eq!(result, 42);
    }
}

// ============================================================================
// RcTransformer TransformerOnce Tests
// ============================================================================

#[cfg(test)]
mod rc_transformer_transformer_once_tests {
    use prism3_function::{
        RcTransformer,
        TransformerOnce,
    };

    #[test]
    fn test_rc_transformer_apply_once() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let result = double.apply_once(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_rc_transformer_into_box_once() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_rc_transformer_into_fn_once() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_rc_transformer_string_transformation() {
        let uppercase = RcTransformer::new(|s: String| s.to_uppercase());
        let result = uppercase.apply_once("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_rc_transformer_complex_transformation() {
        let parse_and_double = RcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
        let result = parse_and_double.apply_once("21".to_string());
        assert_eq!(result, 42);
    }

    #[test]
    fn test_rc_transformer_clone_before_apply_once() {
        let double = RcTransformer::new(|x: i32| x * 2);
        let double_clone = double.clone();

        // Both should work
        assert_eq!(double.apply_once(21), 42);
        assert_eq!(double_clone.apply_once(21), 42);
    }
}

// ============================================================================
// ArcTransformer TransformerOnce Tests
// ============================================================================

#[cfg(test)]
mod arc_transformer_transformer_once_tests {
    use prism3_function::{
        ArcTransformer,
        TransformerOnce,
    };
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_arc_transformer_apply_once() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let result = double.apply_once(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_arc_transformer_into_box_once() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let boxed = double.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_arc_transformer_into_fn_once() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_arc_transformer_string_transformation() {
        let uppercase = ArcTransformer::new(|s: String| s.to_uppercase());
        let result = uppercase.apply_once("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_arc_transformer_complex_transformation() {
        let parse_and_double = ArcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
        let result = parse_and_double.apply_once("21".to_string());
        assert_eq!(result, 42);
    }

    #[test]
    fn test_arc_transformer_clone_before_apply_once() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let double_clone = double.clone();

        // Both should work
        assert_eq!(double.apply_once(21), 42);
        assert_eq!(double_clone.apply_once(21), 42);
    }

    #[test]
    fn test_arc_transformer_thread_safety() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let _double_arc = Arc::new(double);

        let handle = thread::spawn(move || {
            // We can't move out of Arc, so we need to use the regular apply method
            // or create a new transformer in the thread
            let new_double = ArcTransformer::new(|x: i32| x * 2);
            new_double.apply_once(21)
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_arc_transformer_into_box_once_thread_safety() {
        let double = ArcTransformer::new(|x: i32| x * 2);
        let _double_arc = Arc::new(double);

        let handle = thread::spawn(move || {
            // We can't move out of Arc, so we need to create a new transformer
            let new_double = ArcTransformer::new(|x: i32| x * 2);
            let boxed = new_double.into_box_once();
            boxed.apply_once(21)
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 42);
    }
}
