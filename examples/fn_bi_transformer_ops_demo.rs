/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # FnBiTransformerOps 演示
//!
//! 展示如何使用 `FnBiTransformerOps` trait 为闭包提供 `and_then` 和 `when` 方法。

use prism3_function::{BiTransformer, FnBiTransformerOps};

fn main() {
    println!("=== FnBiTransformerOps 演示 ===\n");

    // 示例 1: 基本的 and_then 组合
    println!("1. 基本的 and_then 组合:");
    let add = |x: i32, y: i32| x + y;
    let double = |x: i32| x * 2;

    let composed = add.and_then(double);
    let result = composed.transform(3, 5);
    println!("   (3 + 5) * 2 = {}", result);
    println!();

    // 示例 2: 类型转换的 and_then
    println!("2. 类型转换的 and_then:");
    let multiply = |x: i32, y: i32| x * y;
    let to_string = |x: i32| format!("结果是: {}", x);

    let composed = multiply.and_then(to_string);
    let result = composed.transform(6, 7);
    println!("   6 * 7 = {}", result);
    println!();

    // 示例 3: 条件执行 - when
    println!("3. 条件执行 - when:");
    let add = |x: i32, y: i32| x + y;
    let multiply = |x: i32, y: i32| x * y;

    let conditional = add
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply);

    println!("   当两个数都为正时执行加法，否则执行乘法:");
    println!("   conditional(5, 3) = {}", conditional.transform(5, 3));
    println!("   conditional(-5, 3) = {}", conditional.transform(-5, 3));
    println!();

    // 示例 4: 复杂的条件逻辑
    println!("4. 复杂的条件逻辑:");
    let add = |x: i32, y: i32| x + y;
    let subtract = |x: i32, y: i32| x - y;

    let conditional = add
        .when(|x: &i32, y: &i32| (*x + *y) < 100)
        .or_else(subtract);

    println!("   当和小于100时执行加法，否则执行减法:");
    println!("   conditional(30, 40) = {}", conditional.transform(30, 40));
    println!("   conditional(60, 50) = {}", conditional.transform(60, 50));
    println!();

    // 示例 5: 字符串操作
    println!("5. 字符串操作:");
    let concat = |x: String, y: String| format!("{}-{}", x, y);
    let uppercase = |s: String| s.to_uppercase();

    let composed = concat.and_then(uppercase);
    let result = composed.transform("hello".to_string(), "world".to_string());
    println!("   concat + uppercase: {}", result);
    println!();

    // 示例 6: 函数指针也可以使用
    println!("6. 函数指针也可以使用:");
    fn add_fn(x: i32, y: i32) -> i32 {
        x + y
    }
    fn triple(x: i32) -> i32 {
        x * 3
    }

    let composed = add_fn.and_then(triple);
    let result = composed.transform(4, 6);
    println!("   (4 + 6) * 3 = {}", result);
    println!();

    // 示例 7: 实际应用 - 计算器
    println!("7. 实际应用 - 简单计算器:");
    let calculate = |x: i32, y: i32| x + y;
    let format_result = |result: i32| {
        if result >= 0 {
            format!("✓ 结果: {}", result)
        } else {
            format!("✗ 负数结果: {}", result)
        }
    };

    let calculator = calculate.and_then(format_result);
    println!("   10 + 5 = {}", calculator.transform(10, 5));
    println!("   -10 + 3 = {}", calculator.transform(-10, 3));
    println!();

    // 示例 8: 组合多个操作
    println!("8. 组合多个操作:");
    let add = |x: i32, y: i32| x + y;

    // 先计算和，然后根据是否为偶数选择不同的格式化方式
    let sum_and_format = add.and_then(|n| {
        if n % 2 == 0 {
            format!("{} 是偶数", n)
        } else {
            format!("{} 是奇数", n)
        }
    });

    println!("   3 + 5 = {}", sum_and_format.transform(3, 5));
    println!("   4 + 6 = {}", sum_and_format.transform(4, 6));

    println!("\n=== 演示完成 ===");
}
