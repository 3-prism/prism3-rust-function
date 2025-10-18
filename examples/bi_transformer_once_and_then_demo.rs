/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! BoxBiTransformerOnce and_then 方法示例
//!
//! 演示如何使用 BoxBiTransformerOnce 的 and_then 方法进行链式组合。

use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};

fn main() {
    println!("=== BoxBiTransformerOnce and_then 方法示例 ===\n");

    // 示例 1: 基本的 and_then 使用
    println!("示例 1: 基本的 and_then 使用");
    let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let double = |x: i32| x * 2;
    let composed = add.and_then(double);
    let result = composed.transform(3, 5);
    println!("  (3 + 5) * 2 = {}", result);
    assert_eq!(result, 16);
    println!();

    // 示例 2: 类型转换
    println!("示例 2: 类型转换");
    let add2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let to_string = |x: i32| x.to_string();
    let composed2 = add2.and_then(to_string);
    let result2 = composed2.transform(20, 22);
    println!("  (20 + 22).to_string() = \"{}\"", result2);
    assert_eq!(result2, "42");
    println!();

    // 示例 3: 多级链式组合
    println!("示例 3: 多级链式组合");
    let add3 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let double3 = |x: i32| x * 2;
    let to_string3 = |x: i32| format!("Result: {}", x);
    let composed3 = add3.and_then(double3).and_then(to_string3);
    let result3 = composed3.transform(3, 5);
    println!("  (3 + 5) * 2 -> \"{}\"", result3);
    assert_eq!(result3, "Result: 16");
    println!();

    // 示例 4: 字符串操作
    println!("示例 4: 字符串操作");
    let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
    let uppercase = |s: String| s.to_uppercase();
    let composed4 = concat.and_then(uppercase);
    let result4 = composed4.transform("hello".to_string(), "world".to_string());
    println!("  \"hello\" + \"world\" -> uppercase = \"{}\"", result4);
    assert_eq!(result4, "HELLO WORLD");
    println!();

    // 示例 5: 数学计算链
    println!("示例 5: 数学计算链");
    let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    let to_float = |x: i32| x as f64 / 2.0;
    let composed5 = multiply.and_then(to_float);
    let result5 = composed5.transform(6, 7);
    println!("  (6 * 7) / 2.0 = {}", result5);
    assert!((result5 - 21.0).abs() < 1e-10);
    println!();

    // 示例 6: 复杂的业务逻辑
    println!("示例 6: 复杂的业务逻辑");
    let calculate_total =
        BoxBiTransformerOnce::new(|price: f64, quantity: i32| price * quantity as f64);
    let apply_discount = |total: f64| {
        if total > 100.0 {
            total * 0.9 // 10% 折扣
        } else {
            total
        }
    };
    let format_price = |total: f64| format!("${:.2}", total);
    let composed6 = calculate_total
        .and_then(apply_discount)
        .and_then(format_price);
    let result6 = composed6.transform(15.5, 8);
    println!("  价格: $15.5, 数量: 8");
    println!("  总价(含折扣): {}", result6);
    assert_eq!(result6, "$111.60");
    println!();

    println!("=== 所有示例执行成功! ===");
}
