/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for BiTransformerOnce trait and implementations

use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};

// ============================================================================
// Tests for BiTransformerOnce trait
// ============================================================================

#[cfg(test)]
mod trait_tests {
    use super::*;

    #[test]
    fn test_blanket_impl_with_closure() {
        let add = |x: i32, y: i32| x + y;
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_blanket_impl_with_function() {
        fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }
        assert_eq!(multiply.apply(6, 7), 42);
    }

    #[test]
    fn test_blanket_impl_with_consuming_closure() {
        let owned_x = String::from("hello");
        let owned_y = String::from("world");
        let concat = |x: String, y: String| format!("{} {}", x, y);
        assert_eq!(concat.apply(owned_x, owned_y), "hello world");
    }

    #[test]
    fn test_into_box() {
        let add = |x: i32, y: i32| x + y;
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20, 22), 42);
    }

    #[test]
    fn test_into_fn() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let f = add.into_fn();
        assert_eq!(f(20, 22), 42);
    }

    #[test]
    fn test_blanket_impl_into_fn() {
        // Test into_fn in impl<F, T, U, R> BiTransformerOnce<T, U, R> for F
        let add = |x: i32, y: i32| x + y;
        let f = add.into_fn();
        assert_eq!(f(20, 22), 42);
    }

    #[test]
    fn test_blanket_impl_into_fn_with_string() {
        // Test closure's into_fn method
        let concat = |x: String, y: String| format!("{}-{}", x, y);
        let f = concat.into_fn();
        assert_eq!(f("hello".to_string(), "world".to_string()), "hello-world");
    }

    #[test]
    fn test_blanket_impl_to_box() {
        // Test closure's to_box method (requires Clone)
        #[derive(Clone)]
        struct AddOp;

        impl AddOp {
            fn call(&self, x: i32, y: i32) -> i32 {
                x + y
            }
        }

        let add_op = AddOp;
        let add = move |x: i32, y: i32| add_op.call(x, y);
        let boxed = add.to_box();
        assert_eq!(boxed.apply(20, 22), 42);
        // Original closure still usable
        assert_eq!(add(10, 5), 15);
    }

    #[test]
    fn test_blanket_impl_to_fn() {
        // Test closure's to_fn method (requires Clone)
        #[derive(Clone)]
        struct MultiplyOp(i32);

        impl MultiplyOp {
            fn call(&self, x: i32, y: i32) -> i32 {
                (x + y) * self.0
            }
        }

        let op = MultiplyOp(2);
        let multiply = move |x: i32, y: i32| op.call(x, y);
        let f = multiply.to_fn();
        assert_eq!(f(3, 5), 16); // (3 + 5) * 2
                                 // Original closure still usable
        assert_eq!(multiply(10, 5), 30); // (10 + 5) * 2
    }

    #[test]
    fn test_blanket_impl_to_box_with_string() {
        // Test closure's to_box method with String
        #[derive(Clone)]
        struct ConcatOp {
            separator: String,
        }

        impl ConcatOp {
            fn call(&self, x: String, y: String) -> String {
                format!("{}{}{}", x, self.separator, y)
            }
        }

        let op = ConcatOp {
            separator: "-".to_string(),
        };
        let concat = move |x: String, y: String| op.call(x, y);
        let boxed = concat.to_box();
        assert_eq!(
            boxed.apply("hello".to_string(), "world".to_string()),
            "hello-world"
        );
        // Original closure still usable
        assert_eq!(concat("foo".to_string(), "bar".to_string()), "foo-bar");
    }

    #[test]
    fn test_blanket_impl_to_fn_with_captured_value() {
        // Test closure's to_fn method with captured value
        #[derive(Clone)]
        struct Calculator {
            base: i32,
        }

        impl Calculator {
            fn calculate(&self, x: i32, y: i32) -> i32 {
                (x + y) + self.base
            }
        }

        let calc = Calculator { base: 100 };
        let add_with_base = move |x: i32, y: i32| calc.calculate(x, y);
        let f = add_with_base.to_fn();
        assert_eq!(f(20, 22), 142); // 20 + 22 + 100
                                    // Original closure still usable
        assert_eq!(add_with_base(10, 5), 115); // 10 + 5 + 100
    }
}

// ============================================================================
// Tests for BiTransformerOnce trait default implementations (to_box, to_fn)
// ============================================================================

#[cfg(test)]
mod trait_default_impl_tests {
    use super::*;

    // Create a cloneable custom type that implements BiTransformerOnce
    #[derive(Clone)]
    struct CustomAddTransformer {
        offset: i32,
    }

    impl BiTransformerOnce<i32, i32, i32> for CustomAddTransformer {
        fn apply(self, first: i32, second: i32) -> i32 {
            first + second + self.offset
        }
    }

    #[test]
    fn test_trait_default_to_box() {
        // Test BiTransformerOnce trait's default to_box implementation
        let transformer = CustomAddTransformer { offset: 10 };
        let boxed = transformer.to_box();
        assert_eq!(boxed.apply(20, 22), 52); // 20 + 22 + 10
                                             // Original transformer still usable
        assert_eq!(transformer.apply(5, 3), 18); // 5 + 3 + 10
    }

    #[test]
    fn test_trait_default_to_fn() {
        // Test BiTransformerOnce trait's default to_fn implementation
        let transformer = CustomAddTransformer { offset: 100 };
        let f = transformer.to_fn();
        assert_eq!(f(20, 22), 142); // 20 + 22 + 100
                                    // Original transformer still usable
        assert_eq!(transformer.apply(10, 5), 115); // 10 + 5 + 100
    }

    #[derive(Clone)]
    struct CustomConcatTransformer {
        separator: String,
    }

    impl BiTransformerOnce<String, String, String> for CustomConcatTransformer {
        fn apply(self, first: String, second: String) -> String {
            format!("{}{}{}", first, self.separator, second)
        }
    }

    #[test]
    fn test_trait_default_to_box_with_string() {
        // Test BiTransformerOnce trait's default to_box with String type
        let transformer = CustomConcatTransformer {
            separator: " | ".to_string(),
        };
        let boxed = transformer.to_box();
        assert_eq!(
            boxed.apply("hello".to_string(), "world".to_string()),
            "hello | world"
        );
        // Original transformer still usable
        assert_eq!(
            transformer.apply("foo".to_string(), "bar".to_string()),
            "foo | bar"
        );
    }

    #[test]
    fn test_trait_default_to_fn_with_string() {
        // Test BiTransformerOnce trait's default to_fn with String type
        let transformer = CustomConcatTransformer {
            separator: "->".to_string(),
        };
        let f = transformer.to_fn();
        assert_eq!(f("start".to_string(), "end".to_string()), "start->end");
        // Original transformer still usable
        assert_eq!(transformer.apply("a".to_string(), "b".to_string()), "a->b");
    }

    #[derive(Clone)]
    struct CustomMultiplyTransformer {
        multiplier: f64,
    }

    impl BiTransformerOnce<f64, f64, f64> for CustomMultiplyTransformer {
        fn apply(self, first: f64, second: f64) -> f64 {
            (first + second) * self.multiplier
        }
    }

    #[test]
    fn test_trait_default_to_box_with_float() {
        // Test BiTransformerOnce trait's default to_box with float type
        let transformer = CustomMultiplyTransformer { multiplier: 2.5 };
        let boxed = transformer.to_box();
        let result = boxed.apply(3.0, 5.0);
        assert!((result - 20.0).abs() < 1e-10); // (3.0 + 5.0) * 2.5 = 20.0
                                                // Original transformer still usable
        let result2 = transformer.apply(2.0, 4.0);
        assert!((result2 - 15.0).abs() < 1e-10); // (2.0 + 4.0) * 2.5 = 15.0
    }

    #[test]
    fn test_trait_default_to_fn_with_float() {
        // Test BiTransformerOnce trait's default to_fn with float type
        let transformer = CustomMultiplyTransformer { multiplier: 3.0 };
        let f = transformer.to_fn();
        let result = f(4.0, 6.0);
        assert!((result - 30.0).abs() < 1e-10); // (4.0 + 6.0) * 3.0 = 30.0
                                                // Original transformer still usable
        let result2 = transformer.apply(1.0, 2.0);
        assert!((result2 - 9.0).abs() < 1e-10); // (1.0 + 2.0) * 3.0 = 9.0
    }
}

// ============================================================================
// Tests for BoxBiTransformerOnce
// ============================================================================

#[cfg(test)]
mod box_bi_transformer_once_tests {
    use super::*;

    #[test]
    fn test_new() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(20, 22), 42);
    }

    #[test]
    fn test_new_with_string() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
        assert_eq!(
            concat.apply("hello".to_string(), "world".to_string()),
            "hello world"
        );
    }

    #[test]
    fn test_constant() {
        let constant = BoxBiTransformerOnce::constant("hello");
        assert_eq!(constant.apply(123, 456), "hello");
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxBiTransformerOnce::constant(42);
        assert_eq!(constant.apply("foo", "bar"), 42);
    }

    #[test]
    fn test_transform_consumes_inputs() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let s1 = String::from("hello");
        let s2 = String::from("world");
        let result = concat.apply(s1, s2);
        assert_eq!(result, "hello-world");
        // s1 and s2 are moved and cannot be used here
    }

    #[test]
    fn test_into_box_zero_cost() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20, 22), 42);
    }

    #[test]
    fn test_into_fn_conversion() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let f = add.into_fn();
        assert_eq!(f(20, 22), 42);
    }

    #[test]
    fn test_and_then_with_closure() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let composed = add.and_then(double);
        assert_eq!(composed.apply(3, 5), 16); // (3 + 5) * 2
    }

    #[test]
    fn test_and_then_with_to_string() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let to_string = |x: i32| x.to_string();
        let composed = add.and_then(to_string);
        assert_eq!(composed.apply(20, 22), "42");
    }

    #[test]
    fn test_and_then_chain_multiple() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let double = |x: i32| x * 2;
        let to_string = |x: i32| format!("Result: {}", x);
        let composed = add.and_then(double).and_then(to_string);
        assert_eq!(composed.apply(3, 5), "Result: 16");
    }

    #[test]
    fn test_and_then_with_string_transformation() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
        let uppercase = |s: String| s.to_uppercase();
        let composed = concat.and_then(uppercase);
        assert_eq!(
            composed.apply("hello".to_string(), "world".to_string()),
            "HELLO WORLD"
        );
    }

    #[test]
    fn test_and_then_type_conversion() {
        let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let to_float = |x: i32| x as f64 / 2.0;
        let composed = multiply.and_then(to_float);
        assert!((composed.apply(6, 7) - 21.0).abs() < 1e-10);
    }
}

// ============================================================================
// Tests for BoxBiTransformerOnce::when and conditional logic
// ============================================================================

#[cfg(test)]
mod conditional_tests {
    use super::*;

    #[test]
    fn test_when_with_or_else_condition_true() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);
        assert_eq!(conditional.apply(5, 3), 8); // add
    }

    #[test]
    fn test_when_with_or_else_condition_false() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);
        assert_eq!(conditional.apply(-5, 3), -15); // multiply
    }

    #[test]
    fn test_when_with_closure_or_else() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);
        assert_eq!(conditional.apply(5, 3), 8); // add
    }

    #[test]
    fn test_when_with_closure_or_else_false() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(|x: i32, y: i32| x * y);
        assert_eq!(conditional.apply(-5, 3), -15); // multiply
    }

    #[test]
    fn test_when_with_complex_predicate() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let reverse_concat =
            BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", y, x));
        let conditional = concat
            .when(|x: &String, y: &String| x.len() > y.len())
            .or_else(reverse_concat);

        assert_eq!(
            conditional.apply("hello".to_string(), "hi".to_string()),
            "hello-hi"
        );
    }

    #[test]
    fn test_when_with_complex_predicate_false() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let reverse_concat =
            BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", y, x));
        let conditional = concat
            .when(|x: &String, y: &String| x.len() > y.len())
            .or_else(reverse_concat);

        assert_eq!(
            conditional.apply("hi".to_string(), "hello".to_string()),
            "hello-hi"
        );
    }

    #[test]
    fn test_when_both_inputs_zero() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let constant = BoxBiTransformerOnce::constant(0);
        let conditional = add
            .when(|x: &i32, y: &i32| *x != 0 || *y != 0)
            .or_else(constant);
        assert_eq!(conditional.apply(0, 0), 0); // constant
    }

    #[test]
    fn test_when_one_input_zero() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        let constant = BoxBiTransformerOnce::constant(0);
        let conditional = add
            .when(|x: &i32, y: &i32| *x != 0 || *y != 0)
            .or_else(constant);
        assert_eq!(conditional.apply(5, 0), 5); // add
    }
}

// ============================================================================
// Tests for different data types
// ============================================================================

#[cfg(test)]
mod type_tests {
    use super::*;

    #[test]
    fn test_with_integers() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(10, 20), 30);
    }

    #[test]
    fn test_with_floats() {
        let multiply = BoxBiTransformerOnce::new(|x: f64, y: f64| x * y);
        assert!((multiply.apply(3.5, 2.0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_with_strings() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}{}", x, y));
        assert_eq!(
            concat.apply("hello".to_string(), "world".to_string()),
            "helloworld"
        );
    }

    #[test]
    fn test_with_mixed_types() {
        let format_pair =
            BoxBiTransformerOnce::new(|x: i32, y: String| format!("number: {}, text: {}", x, y));
        assert_eq!(
            format_pair.apply(42, "hello".to_string()),
            "number: 42, text: hello"
        );
    }

    #[test]
    fn test_with_vectors() {
        let merge = BoxBiTransformerOnce::new(|mut x: Vec<i32>, y: Vec<i32>| {
            x.extend(y);
            x
        });
        assert_eq!(merge.apply(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_with_options() {
        let combine = BoxBiTransformerOnce::new(|x: Option<i32>, y: Option<i32>| match (x, y) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        });
        assert_eq!(combine.apply(Some(5), Some(3)), Some(8));
    }

    #[test]
    fn test_with_tuples() {
        let swap = BoxBiTransformerOnce::new(|x: (i32, String), y: (String, i32)| {
            ((y.1, x.1), (x.0, y.0))
        });
        let result = swap.apply((42, "hello".to_string()), ("world".to_string(), 99));
        assert_eq!(
            result,
            ((99, "hello".to_string()), (42, "world".to_string()))
        );
    }
}

// ============================================================================
// Tests for edge cases
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_with_empty_strings() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}{}", x, y));
        assert_eq!(concat.apply(String::new(), String::new()), String::new());
    }

    #[test]
    fn test_with_zero_values() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(0, 0), 0);
    }

    #[test]
    fn test_with_negative_values() {
        let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
        assert_eq!(add.apply(-5, -3), -8);
    }

    #[test]
    fn test_with_large_values() {
        let add = BoxBiTransformerOnce::new(|x: i64, y: i64| x + y);
        assert_eq!(add.apply(1_000_000_000, 2_000_000_000), 3_000_000_000);
    }

    #[test]
    fn test_constant_ignores_inputs() {
        let constant = BoxBiTransformerOnce::constant(42);
        assert_eq!(constant.apply(999, 888), 42);
    }

    #[test]
    fn test_with_unicode_strings() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}{}", x, y));
        assert_eq!(
            concat.apply("Hello".to_string(), "World".to_string()),
            "HelloWorld"
        );
    }
}

// ============================================================================
// Tests for complex transformations
// ============================================================================

#[cfg(test)]
mod complex_transformation_tests {
    use super::*;

    #[test]
    fn test_nested_structure_transformation() {
        let merge_nested =
            BoxBiTransformerOnce::new(|x: Vec<Vec<i32>>, y: Vec<Vec<i32>>| -> Vec<Vec<i32>> {
                let mut result = x;
                result.extend(y);
                result
            });
        assert_eq!(
            merge_nested.apply(vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]),
            vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]]
        );
    }

    #[test]
    fn test_transformation_with_calculation() {
        let calculate = BoxBiTransformerOnce::new(|x: i32, y: i32| {
            let sum = x + y;
            let product = x * y;
            (sum, product)
        });
        assert_eq!(calculate.apply(5, 3), (8, 15));
    }

    #[test]
    fn test_transformation_with_string_manipulation() {
        let process = BoxBiTransformerOnce::new(|x: String, y: String| {
            format!("{} {} {}", x.to_uppercase(), "and", y.to_lowercase())
        });
        assert_eq!(
            process.apply("Hello".to_string(), "WORLD".to_string()),
            "HELLO and world"
        );
    }

    #[test]
    fn test_conditional_with_complex_logic() {
        let complex_add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y + 10);
        let complex_multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y - 5);
        let conditional = complex_add
            .when(|x: &i32, y: &i32| (*x + *y) % 2 == 0)
            .or_else(complex_multiply);

        assert_eq!(conditional.apply(4, 6), 20); // (4 + 6) is even, so add: 4 + 6 + 10 = 20
    }

    #[test]
    fn test_conditional_with_complex_logic_odd() {
        let complex_add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y + 10);
        let complex_multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y - 5);
        let conditional = complex_add
            .when(|x: &i32, y: &i32| (*x + *y) % 2 == 0)
            .or_else(complex_multiply);

        assert_eq!(conditional.apply(3, 4), 7); // (3 + 4) is odd, so multiply: 3 * 4 - 5 = 7
    }
}

// ============================================================================
// Tests for ownership and consumption
// ============================================================================

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn test_consumes_owned_values() {
        let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{}-{}", x, y));
        let s1 = String::from("hello");
        let s2 = String::from("world");
        let result = concat.apply(s1, s2);
        assert_eq!(result, "hello-world");
        // s1 and s2 are consumed and cannot be used here
    }

    #[test]
    fn test_consumes_vectors() {
        let merge = BoxBiTransformerOnce::new(|mut x: Vec<i32>, y: Vec<i32>| {
            x.extend(y);
            x
        });
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5, 6];
        let result = merge.apply(v1, v2);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
        // v1 and v2 are consumed
    }

    #[test]
    fn test_closure_captures_and_consumes() {
        let prefix = String::from("Result: ");
        let concat =
            BoxBiTransformerOnce::new(move |x: String, y: String| format!("{}{}-{}", prefix, x, y));
        let result = concat.apply("hello".to_string(), "world".to_string());
        assert_eq!(result, "Result: hello-world");
        // prefix is moved into closure
    }
}
