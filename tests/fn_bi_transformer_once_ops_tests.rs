/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BiTransformerOnce, FnBiTransformerOnceOps};

#[cfg(test)]
mod fn_bi_transformer_once_ops_tests {
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

        // 需要重新创建，因为是 FnOnce
        let add2 = |x: i32, y: i32| x + y;
        let multiply2 = |x: i32, y: i32| x * y;
        let conditional2 = add2
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
            .or_else(multiply2);
        assert_eq!(conditional2.transform(-5, 3), -15); // 条件不满足，执行乘法
    }

    #[test]
    fn test_closure_when_with_single_condition() {
        // 测试单个条件的 when
        let add = |x: i32, y: i32| x + y;
        let subtract = |x: i32, y: i32| x - y;

        let conditional = add.when(|x: &i32, _y: &i32| *x > 0).or_else(subtract);
        assert_eq!(conditional.transform(10, 3), 13); // x > 0，执行加法

        let add2 = |x: i32, y: i32| x + y;
        let subtract2 = |x: i32, y: i32| x - y;
        let conditional2 = add2.when(|x: &i32, _y: &i32| *x > 0).or_else(subtract2);
        assert_eq!(conditional2.transform(-10, 3), -13); // x <= 0，执行减法
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

        let conditional2 = add.when(both_positive).or_else(multiply);
        assert_eq!(conditional2.transform(-5, 3), -15);
    }

    #[test]
    fn test_chained_and_then() {
        // 测试链式 and_then - 注意：第一次 and_then 返回 BoxBiTransformerOnce，
        // 它没有 and_then 方法，所以需要分步进行
        let add = |x: i32, y: i32| x + y;
        let double = |x: i32| x * 2;

        let step1 = add.and_then(double);
        let result = step1.transform(3, 5);
        assert_eq!(result, 16); // (3 + 5) * 2 = 16
    }

    #[test]
    fn test_and_then_with_consuming_closure() {
        // 测试消费闭包的 and_then
        let owned_value = String::from("prefix-");
        let concat = move |x: String, y: String| format!("{}{}{}", owned_value, x, y);
        let uppercase = |s: String| s.to_uppercase();

        let composed = concat.and_then(uppercase);
        assert_eq!(
            composed.transform("hello".to_string(), "world".to_string()),
            "PREFIX-HELLOWORLD"
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

        let add2 = |x: i32, y: i32| x + y;
        let multiply2 = |x: i32, y: i32| x * y;
        let conditional2 = add2
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0 && (*x + *y) < 20)
            .or_else(multiply2);
        assert_eq!(conditional2.transform(15, 10), 150); // 不满足条件（和 >= 20）

        let add3 = |x: i32, y: i32| x + y;
        let multiply3 = |x: i32, y: i32| x * y;
        let conditional3 = add3
            .when(|x: &i32, y: &i32| *x > 0 && *y > 0 && (*x + *y) < 20)
            .or_else(multiply3);
        assert_eq!(conditional3.transform(-5, 3), -15); // 不满足条件（x <= 0）
    }
}
