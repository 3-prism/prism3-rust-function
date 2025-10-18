/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! 演示 FnTransformerOnceOps 扩展 trait 的使用
//!
//! 这个示例展示了如何直接对 FnOnce 闭包使用 and_then、compose 和 when 方法，
//! 而无需显式地将它们包装在 BoxTransformerOnce 中。

use prism3_function::{FnTransformerOnceOps, TransformerOnce};

fn main() {
    println!("=== FnTransformerOnceOps 示例 ===\n");

    // 1. 基本的 and_then 组合
    println!("1. 基本的 and_then 组合:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let double = |x: i32| x * 2;
    let composed = parse.and_then(double);
    println!(
        "   parse.and_then(double).transform(\"21\") = {}",
        composed.transform("21".to_string())
    );
    println!();

    // 2. 链式 and_then 组合
    println!("2. 链式 and_then 组合:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;
    let chained = parse.and_then(add_one).and_then(double);
    println!(
        "   parse.and_then(add_one).and_then(double).transform(\"5\") = {}",
        chained.transform("5".to_string())
    ); // (5 + 1) * 2 = 12
    println!();

    // 3. compose 反向组合
    println!("3. compose 反向组合:");
    let double = |x: i32| x * 2;
    let to_string = |x: i32| x.to_string();
    let composed = to_string.compose(double);
    println!(
        "   to_string.compose(double).transform(21) = {}",
        composed.transform(21)
    ); // (21 * 2).to_string() = "42"
    println!();

    // 4. 条件转换 when
    println!("4. 条件转换 when:");
    let double = |x: i32| x * 2;
    let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    println!("   double.when(x > 0).or_else(negate):");
    println!("     transform(5) = {}", conditional.transform(5)); // 10

    let double2 = |x: i32| x * 2;
    let conditional2 = double2.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    println!("     transform(-5) = {}", conditional2.transform(-5)); // 5
    println!();

    // 5. 复杂的组合
    println!("5. 复杂的组合:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let double = |x: i32| x * 2;
    let triple = |x: i32| x * 3;
    let to_string = |x: i32| x.to_string();

    let complex = parse
        .and_then(double.when(|x: &i32| *x > 5).or_else(triple))
        .and_then(to_string);

    println!("   parse.and_then(double.when(x > 5).or_else(triple)).and_then(to_string):");
    println!(
        "     transform(\"3\") = {}",
        complex.transform("3".to_string())
    ); // 3 <= 5, so 3 * 3 = 9

    let parse2 = |s: String| s.parse::<i32>().unwrap_or(0);
    let double2 = |x: i32| x * 2;
    let triple2 = |x: i32| x * 3;
    let to_string2 = |x: i32| x.to_string();
    let complex2 = parse2
        .and_then(double2.when(|x: &i32| *x > 5).or_else(triple2))
        .and_then(to_string2);
    println!(
        "     transform(\"10\") = {}",
        complex2.transform("10".to_string())
    ); // 10 > 5, so 10 * 2 = 20
    println!();

    // 6. 类型转换
    println!("6. 类型转换:");
    let to_string = |x: i32| x.to_string();
    let get_length = |s: String| s.len();
    let length_transformer = to_string.and_then(get_length);
    println!(
        "   to_string.and_then(get_length).transform(12345) = {}",
        length_transformer.transform(12345)
    ); // 5
    println!();

    // 7. 捕获环境的闭包
    println!("7. 捕获环境的闭包:");
    let multiplier = 3;
    let multiply = move |x: i32| x * multiplier;
    let add_ten = |x: i32| x + 10;
    let with_capture = multiply.and_then(add_ten);
    println!(
        "   multiply(3).and_then(add_ten).transform(5) = {}",
        with_capture.transform(5)
    ); // 5 * 3 + 10 = 25
    println!();

    // 8. 函数指针
    println!("8. 函数指针:");
    fn parse_fn(s: String) -> i32 {
        s.parse().unwrap_or(0)
    }
    fn double_fn(x: i32) -> i32 {
        x * 2
    }
    let fn_composed = parse_fn.and_then(double_fn);
    println!(
        "   parse_fn.and_then(double_fn).transform(\"21\") = {}",
        fn_composed.transform("21".to_string())
    ); // 42
    println!();

    // 9. 消费所有权的字符串操作
    println!("9. 消费所有权的字符串操作:");
    let owned = String::from("hello");
    let append = move |s: String| format!("{} {}", s, owned);
    let uppercase = |s: String| s.to_uppercase();
    let composed = append.and_then(uppercase);
    println!(
        "   append.and_then(uppercase).transform(\"world\") = {}",
        composed.transform("world".to_string())
    ); // "WORLD HELLO"
    println!();

    // 10. 解析和验证
    println!("10. 解析和验证:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let validate = |x: i32| if x > 0 { x } else { 1 };
    let composed = parse.and_then(validate);
    println!(
        "   parse.and_then(validate).transform(\"42\") = {}",
        composed.transform("42".to_string())
    ); // 42

    let parse2 = |s: String| s.parse::<i32>().unwrap_or(0);
    let validate2 = |x: i32| if x > 0 { x } else { 1 };
    let composed2 = parse2.and_then(validate2);
    println!(
        "   parse.and_then(validate).transform(\"-5\") = {}",
        composed2.transform("-5".to_string())
    ); // 1
    println!();

    println!("=== 示例结束 ===");
}
