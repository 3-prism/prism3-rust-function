/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BoxTransformerOnce, TransformerOnce};

// ============================================================================
// BoxTransformerOnce Tests - Consuming, single ownership
// ============================================================================

#[cfg(test)]
mod box_transformer_once_tests {
    use super::*;

    #[test]
    fn test_new_and_transform() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse.apply("42".to_string()), 42);
    }

    #[test]
    fn test_identity() {
        let identity = BoxTransformerOnce::<i32, i32>::identity();
        assert_eq!(identity.apply(42), 42);
    }

    #[test]
    fn test_constant() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");
    }

    #[test]
    fn test_and_then() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let double = |x: i32| x * 2;
        let composed = add_one.and_then(double);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_compose() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let add_one = |x: i32| x + 1;
        let composed = double.compose(add_one);
        assert_eq!(composed.apply(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_pipeline() {
        let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
        let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
        assert_eq!(pipeline.apply(5), 9); // ((5 + 1) * 2) - 3
    }

    #[test]
    fn test_consuming_string() {
        let into_bytes = BoxTransformerOnce::new(|s: String| s.into_bytes());
        let bytes = into_bytes.apply("hello".to_string());
        assert_eq!(bytes, vec![104, 101, 108, 108, 111]);
    }

    #[test]
    fn test_type_conversion() {
        let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
        let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
        assert_eq!(add_suffix.apply(42), "42_suffix");
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

        assert_eq!(result.apply(5), 10);
    }

    #[test]
    fn test_when_or_else_negative() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let negate = BoxTransformerOnce::new(|x: i32| -x);
        let result = double.when(is_positive).or_else(negate);

        assert_eq!(result.apply(-5), 5);
    }

    #[test]
    fn test_when_or_else_with_closure() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let result = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);

        assert_eq!(result.apply(5), 10);
        let result2 = BoxTransformerOnce::new(|x: i32| x * 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: i32| -x);
        assert_eq!(result2.apply(-5), 5);
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
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_box_to_fn() {
        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        // Test into_fn in impl<F, T, R> TransformerOnce<T, R> for F
        let double = |x: i32| x * 2;
        let func = double.into_fn();
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
        assert_eq!(composed.apply(5), "12"); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_multiple_compose() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let square = BoxTransformerOnce::new(|x: i32| x * x);
        let composed = square.compose(double).compose(add_one);
        assert_eq!(composed.apply(5), 144); // ((5 + 1) * 2)^2 = 144
    }

    #[test]
    fn test_mixed_composition() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
        let double = |x: i32| x * 2;
        let to_string = |x: i32| format!("Result: {}", x);
        let pipeline = parse.and_then(double).and_then(to_string);
        assert_eq!(pipeline.apply("21".to_string()), "Result: 42");
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
        let composed = double.and_then(|x| identity.apply(x));
        assert_eq!(composed.apply(21), 42);
    }

    #[test]
    fn test_constant_with_different_types() {
        let constant = BoxTransformerOnce::constant("hello");
        assert_eq!(constant.apply(123), "hello");

        let constant2 = BoxTransformerOnce::constant("world");
        assert_eq!(constant2.apply(456), "world");
    }

    #[test]
    fn test_with_option() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse.apply("42".to_string()), Some(42));

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>().ok());
        assert_eq!(parse2.apply("abc".to_string()), None);
    }

    #[test]
    fn test_with_result() {
        let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse.apply("42".to_string()).is_ok());

        let parse2 = BoxTransformerOnce::new(|s: String| s.parse::<i32>());
        assert!(parse2.apply("abc".to_string()).is_err());
    }

    #[test]
    fn test_with_vec() {
        let split = BoxTransformerOnce::new(|s: String| {
            s.split(',').map(|s| s.to_string()).collect::<Vec<_>>()
        });
        assert_eq!(
            split.apply("a,b,c".to_string()),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_consuming_ownership() {
        let vec = vec![1, 2, 3, 4, 5];
        let sum = BoxTransformerOnce::new(|v: Vec<i32>| v.iter().sum::<i32>());
        assert_eq!(sum.apply(vec), 15);
        // vec is consumed and cannot be used again
    }

    #[test]
    fn test_with_box() {
        let boxed = Box::new(42);
        let unbox = BoxTransformerOnce::new(|b: Box<i32>| *b);
        assert_eq!(unbox.apply(boxed), 42);
    }

    #[test]
    fn test_with_closure_capture() {
        let multiplier = 3;
        let multiply = BoxTransformerOnce::new(move |x: i32| x * multiplier);
        assert_eq!(multiply.apply(7), 21);
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
            f.apply(x)
        }

        let double = BoxTransformerOnce::new(|x: i32| x * 2);
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_closure_as_transformer_once() {
        fn apply_transformer_once<F: TransformerOnce<i32, i32>>(f: F, x: i32) -> i32 {
            f.apply(x)
        }

        let double = |x: i32| x * 2;
        assert_eq!(apply_transformer_once(double, 21), 42);
    }

    #[test]
    fn test_with_different_types() {
        fn apply_transformer_once<T, R, F: TransformerOnce<T, R>>(f: F, x: T) -> R {
            f.apply(x)
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
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20), 30);
    }

    #[test]
    fn test_box_into_fn() {
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let func = add.into_fn();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_closure_into_box() {
        let double = |x: i32| x * 2;
        let boxed = double.into_box();
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_closure_into_fn() {
        let double = |x: i32| x * 2;
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_closure_to_box_and_preserve_original() {
        // to_box borrows &self and requires Clone; non-capturing closures are Clone
        let double = |x: i32| x * 2;
        let boxed = double.to_box();
        assert_eq!(boxed.apply(21), 42);

        // 原始闭包仍然可用（to_box 未消费原对象）
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_closure_to_fn_and_preserve_original() {
        // to_fn borrows &self and requires Clone; non-capturing closures are Clone
        let double = |x: i32| x * 2;
        let func = double.to_fn();
        assert_eq!(func(14), 28);

        // 原始闭包仍然可用（to_fn 未消费原对象）
        assert_eq!(double.apply(7), 14);
    }

    #[test]
    fn test_function_pointer_into_box() {
        fn triple(x: i32) -> i32 {
            x * 3
        }
        let boxed = triple.into_box();
        assert_eq!(boxed.apply(14), 42);
    }

    #[test]
    fn test_function_pointer_into_fn() {
        fn triple(x: i32) -> i32 {
            x * 3
        }
        let func = triple.into_fn();
        assert_eq!(func(14), 42);
    }
}

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod default_implementation_tests {
    use super::*;

    // 自定义类型测试默认实现
    struct CustomTransformer {
        factor: i32,
    }

    impl TransformerOnce<i32, i32> for CustomTransformer {
        fn apply(self, input: i32) -> i32 {
            input * self.factor
        }
        // 使用默认的 into_box 和 into_fn 实现
    }

    #[test]
    fn test_custom_transformer_into_box() {
        let transformer = CustomTransformer { factor: 2 };
        let boxed = transformer.into_box();
        assert_eq!(boxed.apply(21), 42);
    }

    #[test]
    fn test_custom_transformer_into_fn() {
        let transformer = CustomTransformer { factor: 2 };
        let func = transformer.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_custom_transformer_chain() {
        let transformer1 = CustomTransformer { factor: 2 };
        let transformer2 = CustomTransformer { factor: 3 };
        let composed = transformer1.into_box().and_then(transformer2);
        assert_eq!(composed.apply(7), 42); // 7 * 2 * 3
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
        // BoxTransformerOnce::into_box() 应该直接返回自己，零成本
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let boxed = add.into_box();
        assert_eq!(boxed.apply(20), 30);
    }

    #[test]
    fn test_box_into_fn_is_zero_cost() {
        // BoxTransformerOnce::into_fn() 应该直接返回内部函数，零成本
        let add = BoxTransformerOnce::new(|x: i32| x + 10);
        let func = add.into_fn();
        assert_eq!(func(20), 30);
    }

    #[test]
    fn test_closure_into_fn_is_zero_cost() {
        // 闭包的 into_fn() 应该直接返回自己，零成本
        let double = |x: i32| x * 2;
        let func = double.into_fn();
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_chained_conversions() {
        // 测试链式转换
        let double = |x: i32| x * 2;
        let boxed = double.into_box(); // 闭包 -> Box
        let func = boxed.into_fn(); // Box -> Fn (零成本，直接返回内部函数)
        assert_eq!(func(21), 42);
    }

    #[test]
    fn test_complex_type_conversion() {
        // 测试复杂类型转换
        let parse = |s: String| s.parse::<i32>().unwrap_or(0);
        let boxed = parse.into_box();
        let composed = boxed.and_then(|x| x * 2);
        let func = composed.into_fn();
        assert_eq!(func("21".to_string()), 42);
    }
}

// ============================================================================
// Custom Type Default Implementation Tests
// ============================================================================

#[cfg(test)]
mod custom_type_default_impl_tests {
    use super::*;

    /// 自定义的可克隆的 TransformerOnce 类型
    ///
    /// 这个类型演示了如何实现 TransformerOnce trait，
    /// 并且通过实现 Clone，可以使用 to_box() 和 to_fn() 方法
    #[derive(Clone)]
    struct CustomTransformer {
        multiplier: i32,
    }

    impl TransformerOnce<i32, i32> for CustomTransformer {
        fn apply(self, input: i32) -> i32 {
            input * self.multiplier
        }
    }

    #[test]
    fn test_custom_into_box() {
        // 测试 into_box 默认实现（消费 self）
        let transformer = CustomTransformer { multiplier: 3 };
        let boxed = transformer.into_box();
        assert_eq!(boxed.apply(14), 42);
        // transformer 已经被消费，无法再使用
    }

    #[test]
    fn test_custom_into_fn() {
        // 测试 into_fn 默认实现（消费 self）
        let transformer = CustomTransformer { multiplier: 3 };
        let func = transformer.into_fn();
        assert_eq!(func(14), 42);
        // transformer 已经被消费，无法再使用
    }

    #[test]
    fn test_custom_to_box() {
        // 测试 to_box 默认实现（借用 &self，需要 Clone）
        let transformer = CustomTransformer { multiplier: 3 };
        let boxed = transformer.to_box();

        // 先使用转换后的 boxed
        assert_eq!(boxed.apply(14), 42);

        // 原始 transformer 仍然可用（因为 to_box 只是借用）
        assert_eq!(transformer.apply(10), 30);
    }

    #[test]
    fn test_custom_to_fn() {
        // 测试 to_fn 默认实现（借用 &self，需要 Clone）
        let transformer = CustomTransformer { multiplier: 3 };
        let func = transformer.to_fn();

        // 先使用转换后的函数
        assert_eq!(func(14), 42);

        // 原始 transformer 仍然可用（因为 to_fn 只是借用）
        assert_eq!(transformer.apply(10), 30);
    }

    #[test]
    fn test_custom_multiple_conversions() {
        // 测试多次转换
        let transformer = CustomTransformer { multiplier: 2 };

        // 使用 to_box 多次（不消费原对象）
        let boxed1 = transformer.to_box();
        let boxed2 = transformer.to_box();
        let func = transformer.to_fn();

        assert_eq!(boxed1.apply(5), 10);
        assert_eq!(boxed2.apply(10), 20);
        assert_eq!(func(15), 30);

        // 原始 transformer 仍然可用
        assert_eq!(transformer.apply(21), 42);
    }

    #[test]
    fn test_custom_composition_with_to_box() {
        // 测试使用 to_box 进行组合
        let double = CustomTransformer { multiplier: 2 };
        let boxed = double.to_box();

        // 组合其他转换器
        let composed = boxed.and_then(|x| x + 2);
        assert_eq!(composed.apply(20), 42); // 20 * 2 + 2 = 42

        // 原始 transformer 仍然可用
        assert_eq!(double.apply(10), 20);
    }

    #[test]
    fn test_custom_composition_with_to_fn() {
        // 测试使用 to_fn 进行组合
        let triple = CustomTransformer { multiplier: 3 };
        let func = triple.to_fn();

        // 使用函数进行转换
        let result = func(14);
        assert_eq!(result, 42);

        // 原始 transformer 仍然可用（因为 to_fn 只是借用）
        assert_eq!(triple.apply(7), 21);
    }

    /// 带有复杂状态的自定义转换器
    #[derive(Clone)]
    struct ComplexTransformer {
        prefix: String,
        suffix: String,
    }

    impl TransformerOnce<i32, String> for ComplexTransformer {
        fn apply(self, input: i32) -> String {
            format!("{}{}{}", self.prefix, input, self.suffix)
        }
    }

    #[test]
    fn test_complex_custom_to_box() {
        // 测试复杂类型的 to_box
        let transformer = ComplexTransformer {
            prefix: "Number: ".to_string(),
            suffix: "!".to_string(),
        };

        let boxed = transformer.to_box();
        assert_eq!(boxed.apply(42), "Number: 42!");

        // 原始 transformer 仍然可用（因为 to_box 只是借用）
        assert_eq!(transformer.apply(100), "Number: 100!");
    }

    #[test]
    fn test_complex_custom_to_fn() {
        // 测试复杂类型的 to_fn
        let transformer = ComplexTransformer {
            prefix: "Value: ".to_string(),
            suffix: " units".to_string(),
        };

        let func = transformer.to_fn();
        assert_eq!(func(42), "Value: 42 units");

        // 原始 transformer 仍然可用（因为 to_fn 只是借用）
        assert_eq!(transformer.apply(100), "Value: 100 units");
    }

    #[test]
    fn test_complex_custom_chain_conversions() {
        // 测试复杂链式转换
        let transformer = ComplexTransformer {
            prefix: "[".to_string(),
            suffix: "]".to_string(),
        };

        // 先用 to_box 创建一个 BoxTransformerOnce
        let boxed = transformer.to_box();

        // 然后将 BoxTransformerOnce 转换为函数
        let func = boxed.into_fn();
        assert_eq!(func(42), "[42]");

        // 原始 transformer 仍然可用（因为使用了 to_box 而不是 into_box）
        assert_eq!(transformer.apply(100), "[100]");
    }
}
