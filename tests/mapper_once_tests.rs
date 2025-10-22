/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BoxMapperOnce, MapperOnce};

// ============================================================================
// BoxMapperOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_mapper_once_tests {
    use super::*;

    #[test]
    fn test_new_and_apply() {
        let parse = BoxMapperOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse.apply_once("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxMapperOnce::<i32, i32>::identity();
        assert_eq!(identity.apply_once(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxMapperOnce::constant("hello");
        assert_eq!(constant.apply_once(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxMapperOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxMapperOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.apply_once(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxMapperOnce::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.apply_once(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxMapperOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.apply_once("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxMapperOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply_once(42), "42_suffix");
    }
}

// ============================================================================
// Conditional Mapper Once Tests
// ============================================================================

#[cfg(test)]
mod conditional_tests {
    use super::*;
    use prism3_function::BoxPredicate;

    #[test]
    fn test_when_or_else() {
        let double = BoxMapperOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxMapperOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply_once(5), 10);
    }

    #[test]
    fn test_when_or_else_negative() {
        let double = BoxMapperOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxMapperOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply_once(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxMapperOnce::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply_once(5), 10);
        let result2 = BoxMapperOnce::new(|x: i32| x * 2)
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
        let double = BoxMapperOnce::new(|x: i32| x * 2);
        let func = double.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        // Test into_fn in impl<F, T, R> MapperOnce<T, R> for F
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
        let add_one = BoxMapperOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| x.to_string();
        let composed = add_one.and_then(double).and_then(to_string);
        assert_eq!(composed.apply_once(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let square = BoxMapperOnce::new(|x: i32| x * x);
        let composed = square.compose(double).compose(add_one);
        assert_eq!(composed.apply_once(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_mixed_composition() {
        let parse = BoxMapperOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
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
        let double = BoxMapperOnce::new(|x: i32| x * 2);
        let identity = BoxMapperOnce::<i32, i32>::identity();
        let composed = double.and_then(|x| identity.apply_once(x));
        assert_eq!(composed.apply_once(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxMapperOnce::constant("hello");
        assert_eq!(constant.apply_once(123), "hello");

        let constant2 = BoxMapperOnce::constant("world");
        assert_eq!(constant2.apply_once(456), "world");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxMapperOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.apply_once("42".to_string()), Some(42));

        let parse2 = BoxMapperOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse2.apply_once("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxMapperOnce::new(|s: String| s.parse::<i32>());
        assert!(parse.apply_once("42".to_string()).is_ok());

        let parse2 = BoxMapperOnce::new(|s: String| s.parse::<i32>());
        assert!(parse2.apply_once("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split =
            BoxMapperOnce::new(|s: String| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(
            split.apply_once("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_consuming_ownership() {
        let vec = vec![1, 2, 3, 4, 5];
        let sum = BoxMapperOnce::new(|v: Vec<i32>| v.iter().sum::<i32>());
        assert_eq!(sum.apply_once(vec), 15);
        // vec is consumed and cannot be used again
    }

    #[test]
    fn test_with_box() {
        let boxed = Box::new(42);
        let unbox = BoxMapperOnce::new(|b: Box<i32>| *b);
        assert_eq!(unbox.apply_once(boxed), 42);
    }

    #[test]
    fn test_with_closure_capture() {
        let multiplier = 3;
        let multiply = BoxMapperOnce::new(move |x: i32| x * multiplier);
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
    fn test_mapper_once_trait() {
        fn apply_mapper_once<F: MapperOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply_once(x)
        }

        let double = BoxMapperOnce::new(|x: i32| x * 2);
        assert_eq!(apply_mapper_once(double, 21), 42);
    }

    #[test]
    fn test_closure_as_mapper_once() {
        fn apply_mapper_once<F: MapperOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply_once(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_mapper_once(double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_mapper_once<T, R, F: MapperOnce<T, R>>(f: F, x: T) -> R {
            f.apply_once(x)
        }

        let to_string = BoxMapperOnce::new(|x: i32| x.to_string());
        assert_eq!(apply_mapper_once(to_string, 42), "42");
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
        let add = BoxMapperOnce::new(|x: i32| x + 10);
        let boxed = add.into_box_once();
        assert_eq!(boxed.apply_once(20), 30);
    }

    #[test]
    fn test_box_into_fn() {
        let add = BoxMapperOnce::new(|x: i32| x + 10);
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
    struct CustomMapper {
        factor: i32,
    }

    impl MapperOnce<i32, i32> for CustomMapper {
        fn apply_once(self, input: i32) -> i32 {
            input * self.factor
        }
        // Use default into_box_once and into_fn_once implementations
    }

    #[test]
    fn test_custom_mapper_into_box() {
        let mapper = CustomMapper { factor: 2 };
        let boxed = mapper.into_box_once();
        assert_eq!(boxed.apply_once(21), 42);
    }

    #[test]
    fn test_custom_mapper_into_fn() {
        let mapper = CustomMapper { factor: 2 };
        let func = mapper.into_fn_once();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_custom_mapper_chain() {
        let mapper1 = CustomMapper { factor: 2 };
        let mapper2 = CustomMapper { factor: 3 };
        let composed = mapper1.into_box_once().and_then(mapper2);
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
        // BoxMapperOnce::into_box() should directly return itself, zero cost
        let add = BoxMapperOnce::new(|x: i32| x + 10);
        let boxed = add.into_box_once();
        assert_eq!(boxed.apply_once(20), 30);
    }

    #[test]
    fn test_box_into_fn_is_zero_cost() {
        // BoxMapperOnce::into_fn() should directly return the inner function, zero cost
        let add = BoxMapperOnce::new(|x: i32| x + 10);
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

    /// Custom cloneable MapperOnce type
    ///
    /// This type demonstrates how to implement the MapperOnce trait,
    /// and by implementing Clone, it can use the to_box() and to_fn() methods
    #[derive(Clone)]
    struct CustomMapper {
        multiplier: i32,
    }

    impl MapperOnce<i32, i32> for CustomMapper {
        fn apply_once(self, input: i32) -> i32 {
            input * self.multiplier
        }
    }

    #[test]
    fn test_custom_into_box() {
        // Test into_box default implementation (consumes self)
        let mapper = CustomMapper { multiplier: 3 };
        let boxed = mapper.into_box_once();
        assert_eq!(boxed.apply_once(14), 42);
        // mapper has been consumed and cannot be used again
    }

    #[test]
    fn test_custom_into_fn() {
        // Test into_fn default implementation (consumes self)
        let mapper = CustomMapper { multiplier: 3 };
        let func = mapper.into_fn_once();
        assert_eq!(func(14), 42);
        // mapper has been consumed and cannot be used again
    }

    #[test]
    fn test_custom_to_box() {
        // Test to_box default implementation (borrows &self, requires Clone)
        let mapper = CustomMapper { multiplier: 3 };
        let boxed = mapper.to_box_once();

        // Use the converted boxed first
        assert_eq!(boxed.apply_once(14), 42);

        // Original mapper is still available (because to_box just borrows)
        assert_eq!(mapper.apply_once(10), 30);
    }

    #[test]
    fn test_custom_to_fn() {
        // Test to_fn default implementation (borrows &self, requires Clone)
        let mapper = CustomMapper { multiplier: 3 };
        let func = mapper.to_fn_once();

        // Use the converted function first
        assert_eq!(func(14), 42);

        // Original mapper is still available (because to_fn just borrows)
        assert_eq!(mapper.apply_once(10), 30);
    }

    #[test]
    fn test_custom_multiple_conversions() {
        // Test multiple conversions
        let mapper = CustomMapper { multiplier: 2 };

        // Use to_box multiple times (does not consume original object)
        let boxed1 = mapper.to_box_once();
        let boxed2 = mapper.to_box_once();
        let func = mapper.to_fn_once();

        assert_eq!(boxed1.apply_once(5), 10);
        assert_eq!(boxed2.apply_once(10), 20);
        assert_eq!(func(15), 30);

        // Original mapper is still available
        assert_eq!(mapper.apply_once(21), 42);
    }

    #[test]
    fn test_custom_composition_with_to_box() {
        // Test composition using to_box
        let double = CustomMapper { multiplier: 2 };
        let boxed = double.to_box_once();

        // Compose with other mappers
        let composed = boxed.and_then(|x| x + 2);
        assert_eq!(composed.apply_once(20), 42); // 20 * 2 + 2 = 42

        // Original mapper is still available
        assert_eq!(double.apply_once(10), 20);
    }

    #[test]
    fn test_custom_composition_with_to_fn() {
        // Test composition using to_fn
        let triple = CustomMapper { multiplier: 3 };
        let func = triple.to_fn_once();

        // Use function for transformation
        let result = func(14);
        assert_eq!(result, 42);

        // Original mapper is still available (because to_fn just borrows)
        assert_eq!(triple.apply_once(7), 21);
    }

    /// Custom mapper with complex state
    #[derive(Clone)]
    struct ComplexMapper {
        prefix: String,
        suffix: String,
    }

    impl MapperOnce<i32, String> for ComplexMapper {
        fn apply_once(self, input: i32) -> String {
            format!("{}{}{}", self.prefix, input, self.suffix)
        }
    }

    #[test]
    fn test_complex_custom_to_box() {
        // Test to_box for complex types
        let mapper = ComplexMapper {
            prefix: "Number: ".to_string(),
            suffix: "!".to_string(),
        };

        let boxed = mapper.to_box_once();
        assert_eq!(boxed.apply_once(42), "Number: 42!");

        // Original mapper is still available (because to_box just borrows)
        assert_eq!(mapper.apply_once(100), "Number: 100!");
    }

    #[test]
    fn test_complex_custom_to_fn() {
        // Test to_fn for complex types
        let mapper = ComplexMapper {
            prefix: "Value: ".to_string(),
            suffix: " units".to_string(),
        };

        let func = mapper.to_fn_once();
        assert_eq!(func(42), "Value: 42 units");

        // Original mapper is still available (because to_fn just borrows)
        assert_eq!(mapper.apply_once(100), "Value: 100 units");
    }

    #[test]
    fn test_complex_custom_chain_conversions() {
        // Test complex chained conversions
        let mapper = ComplexMapper {
            prefix: "[".to_string(),
            suffix: "]".to_string(),
        };

        // First use to_box to create a BoxMapperOnce
        let boxed = mapper.to_box_once();

        // Then convert BoxMapperOnce to function
        let func = boxed.into_fn_once();
        assert_eq!(func(42), "[42]");

        // Original mapper is still available (because to_box was used, not into_box)
        assert_eq!(mapper.apply_once(100), "[100]");
    }
}

// ============================================================================
// Stateful Mapper Tests
// ============================================================================

#[cfg(test)]
mod stateful_mapper_tests {
    use super::*;

    #[test]
    fn test_stateful_mapper_with_captured_state() {
        // Test mapper that consumes captured state
        let prefix = "Value: ".to_string();
        let mapper = BoxMapperOnce::new(move |x: i32| format!("{}{}", prefix, x));
        assert_eq!(mapper.apply_once(42), "Value: 42");
        // prefix is moved and consumed
    }

    #[test]
    fn test_stateful_mapper_chain() {
        // Test chaining with stateful mappers
        let prefix = "Result: ".to_string();
        let suffix = "!".to_string();

        let add_prefix = BoxMapperOnce::new(move |x: i32| format!("{}{}", prefix, x));
        let add_suffix = move |s: String| format!("{}{}", s, suffix);

        let composed = add_prefix.and_then(add_suffix);
        assert_eq!(composed.apply_once(42), "Result: 42!");
    }

    #[test]
    fn test_mapper_consuming_vec() {
        // Test mapper that consumes a Vec
        let vec = vec![1, 2, 3];
        let sum_mapper = BoxMapperOnce::new(|v: Vec<i32>| v.into_iter().sum::<i32>());
        assert_eq!(sum_mapper.apply_once(vec), 6);
    }

    #[test]
    fn test_mapper_with_box_state() {
        // Test mapper that works with boxed values
        let boxed = Box::new(42);
        let unbox_and_double = BoxMapperOnce::new(|b: Box<i32>| *b * 2);
        assert_eq!(unbox_and_double.apply_once(boxed), 84);
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_result_mapper() {
        let parse_mapper =
            BoxMapperOnce::new(|s: String| -> Result<i32, std::num::ParseIntError> {
                s.parse::<i32>()
            });

        let result = parse_mapper.apply_once("42".to_string());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_mapper_error() {
        let parse_mapper =
            BoxMapperOnce::new(|s: String| -> Result<i32, std::num::ParseIntError> {
                s.parse::<i32>()
            });

        let result = parse_mapper.apply_once("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_option_mapper() {
        let parse_mapper = BoxMapperOnce::new(|s: String| s.parse::<i32>().ok());

        let result = parse_mapper.apply_once("42".to_string());
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_option_mapper_none() {
        let parse_mapper = BoxMapperOnce::new(|s: String| s.parse::<i32>().ok());

        let result = parse_mapper.apply_once("invalid".to_string());
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_chain() {
        let parse = BoxMapperOnce::new(|s: String| s.parse::<i32>().ok());
        let double = |opt: Option<i32>| opt.map(|x| x * 2);

        let composed = parse.and_then(double);
        assert_eq!(composed.apply_once("21".to_string()), Some(42));
    }
}
