/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! 演示 FnTransformerOps 扩展 trait 的使用
//!
//! 这个示例展示了如何直接对闭包使用 and_then、compose 和 when 方法，
//! 而无需显式地将它们包装在 BoxTransformer、RcTransformer 或 ArcTransformer 中。

use prism3_function::{FnTransformerOps, Transformer};

fn main() {
    println!("=== FnTransformerOps 示例 ===\n");

    // 1. 基本的 and_then 组合
    println!("1. 基本的 and_then 组合:");
    let double = |x: i32| x * 2;
    let to_string = |x: i32| x.to_string();
    let composed = double.and_then(to_string);
    println!(
        "   double.and_then(to_string).transform(21) = {}",
        composed.transform(21)
    );
    println!();

    // 2. 链式 and_then 组合
    println!("2. 链式 and_then 组合:");
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;
    let to_string = |x: i32| x.to_string();
    let chained = add_one.and_then(double).and_then(to_string);
    println!(
        "   add_one.and_then(double).and_then(to_string).transform(5) = {}",
        chained.transform(5)
    ); // (5 + 1) * 2 = 12
    println!();

    // 3. compose 反向组合
    println!("3. compose 反向组合:");
    let double = |x: i32| x * 2;
    let add_one = |x: i32| x + 1;
    let composed = double.compose(add_one);
    println!(
        "   double.compose(add_one).transform(5) = {}",
        composed.transform(5)
    ); // (5 + 1) * 2 = 12
    println!();

    // 4. 条件转换 when
    println!("4. 条件转换 when:");
    let double = |x: i32| x * 2;
    let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    println!("   double.when(x > 0).or_else(negate):");
    println!("     transform(5) = {}", conditional.transform(5)); // 10
    println!("     transform(-5) = {}", conditional.transform(-5)); // 5
    println!();

    // 5. 复杂的组合
    println!("5. 复杂的组合:");
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;
    let triple = |x: i32| x * 3;
    let to_string = |x: i32| x.to_string();

    let complex = add_one
        .and_then(double.when(|x: &i32| *x > 5).or_else(triple))
        .and_then(to_string);

    println!("   add_one.and_then(double.when(x > 5).or_else(triple)).and_then(to_string):");
    println!("     transform(1) = {}", complex.transform(1)); // (1 + 1) = 2 <= 5, so 2 * 3 = 6
    println!("     transform(5) = {}", complex.transform(5)); // (5 + 1) = 6 > 5, so 6 * 2 = 12
    println!("     transform(10) = {}", complex.transform(10)); // (10 + 1) = 11 > 5, so 11 * 2 = 22
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
    fn double_fn(x: i32) -> i32 {
        x * 2
    }
    fn add_one_fn(x: i32) -> i32 {
        x + 1
    }
    let fn_composed = double_fn.and_then(add_one_fn);
    println!(
        "   double_fn.and_then(add_one_fn).transform(5) = {}",
        fn_composed.transform(5)
    ); // 5 * 2 + 1 = 11
    println!();

    // 9. 多条件转换
    println!("9. 多条件转换:");
    let abs = |x: i32| x.abs();
    let double = |x: i32| x * 2;
    let transformer = abs.when(|x: &i32| *x < 0).or_else(double);
    println!("   abs.when(x < 0).or_else(double):");
    println!("     transform(-5) = {}", transformer.transform(-5)); // abs(-5) = 5
    println!("     transform(5) = {}", transformer.transform(5)); // 5 * 2 = 10
    println!("     transform(0) = {}", transformer.transform(0)); // 0 * 2 = 0
    println!();

    println!("=== 示例结束 ===");
}
