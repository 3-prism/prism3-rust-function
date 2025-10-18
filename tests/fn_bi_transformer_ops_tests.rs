/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BiTransformer, FnBiTransformerOps};

#[cfg(test)]
mod fn_bi_transformer_ops_tests {
    use super::*;

    #[test]
    fn test_closure_and_then() {
        // 测试闭包的 and_then 方法
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let composed = add.and_then(double);
        assert_eq!(composed.transform(3, 5), 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_closure_and_then_with_type_conversion() {
        // 测试类型转换的 and_then
        let add = |x: i32, y: i32| x + y;
        let to_string = |x: i32| x.to_string();

        let composed = add.and_then(to_string);
        assert_eq!(composed.transform(20, 22), "42");
    }

    #[test]
    fn test_closure_when_with_or_else() {
        // 测试闭包的 when 方法
        let add = |x: i32, y: i32| x + y;
        let multiply = |x: i32, y: i32| x * y;

        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply);

        assert_eq!(conditional.transform(5, 3), 8); // 条件满足，执行加法
        assert_eq!(conditional.transform(-5, 3), -15); // 条件不满足，执行乘法
    }

    #[test]
    fn test_closure_when_with_single_condition() {
        // 测试单个条件的 when
        let add = |x: i32, y: i32| x + y;
        let subtract = |x: i32, y: i32| x - y;

        let conditional = add.when(|x: &i32, _y: &i32| *x > 0).or_else(subtract);

        assert_eq!(conditional.transform(10, 3), 13); // x > 0，执行加法
        assert_eq!(conditional.transform(-10, 3), -13); // x <= 0，执行减法
    }

    #[test]
    fn test_function_pointer_and_then() {
        // 测试函数指针的 and_then
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        fn double(x: i32) -> i32 {
            x * 2
        }

        let composed = add.and_then(double);
        assert_eq!(composed.transform(3, 5), 16);
    }

    #[test]
    fn test_function_pointer_when() {
        // 测试函数指针的 when
        fn add(x: i32, y: i32) -> i32 {
            x + y
        }
        fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }
        fn both_positive(x: &i32, y: &i32) -> bool {
            *x > 0 && *y > 0
        }

        let conditional = add.when(both_positive).or_else(multiply);

        assert_eq!(conditional.transform(5, 3), 8);
        assert_eq!(conditional.transform(-5, 3), -15);
    }

    #[test]
    fn test_chained_and_then() {
        // 测试链式 and_then - 注意：第一次 and_then 返回 BoxBiTransformer，
        // 它没有 and_then 方法，所以需要分步进行
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let step1 = add.and_then(double);
        let result = step1.transform(3, 5);
        assert_eq!(result, 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_and_then_with_string_types() {
        // 测试字符串类型的组合
        let concat = |x: String, y: String| format!("{}{}", x, y);
        let uppercase = |s: String| s.to_uppercase();

        let composed = concat.and_then(uppercase);
        assert_eq!(
            composed.transform("hello".to_string(), "world".to_string()),
            "HELLOWORLD"
        );
    }

    #[test]
    fn test_when_with_complex_predicate() {
        // 测试复杂谓词
        let add = |x: i32, y: i32| x + y;
        let multiply = |x: i32, y: i32| x * y;

        let conditional = add
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0 && (*x + *y) < 20)
            .or_else(multiply);

        assert_eq!(conditional.transform(5, 3), 8); // 满足条件
        assert_eq!(conditional.transform(15, 10), 150); // 不满足条件（和 >= 20）
        assert_eq!(conditional.transform(-5, 3), -15); // 不满足条件（x <= 0）
    }

    #[test]
    fn test_multiple_operations() {
        // 测试多个操作的组合
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        // 先进行 and_then 组合
        let composed = add.and_then(double);
        assert_eq!(composed.transform(3, 5), 16); // (3 + 5) * 2 = 16

        // 测试另一个组合
        let multiply = |x: i32, y: i32| x * y;
        let triple = |x: i32| x * 3;
        let composed2 = multiply.and_then(triple);
        assert_eq!(composed2.transform(2, 3), 18); // (2 * 3) * 3 = 18
    }
}
