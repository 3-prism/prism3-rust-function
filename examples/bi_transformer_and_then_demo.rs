/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # BiTransformer and_then 方法演示
//!
//! 演示 BoxBiTransformer、ArcBiTransformer 和 RcBiTransformer 的 and_then 方法使用
//!
//! # 作者
//!
//! 胡海星

use prism3_function::{ArcBiTransformer, BiTransformer, BoxBiTransformer, RcBiTransformer};

fn main() {
    println!("=== BiTransformer and_then 方法演示 ===\n");

    // 1. BoxBiTransformer::and_then - 基本用法
    println!("1. BoxBiTransformer::and_then - 基本用法");
    let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    let double = |x: i32| x * 2;
    let composed = add.and_then(double);
    println!("   (3 + 5) * 2 = {}", composed.transform(3, 5));
    println!();

    // 2. BoxBiTransformer::and_then - 链式调用
    println!("2. BoxBiTransformer::and_then - 链式调用");
    let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
    let add_ten = |x: i32| x + 10;
    let to_string = |x: i32| format!("结果是: {}", x);
    let pipeline = multiply.and_then(add_ten).and_then(to_string);
    println!("   (6 * 7) + 10 = {}", pipeline.transform(6, 7));
    println!();

    // 3. ArcBiTransformer::and_then - 共享所有权
    println!("3. ArcBiTransformer::and_then - 共享所有权");
    let add_arc = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    let triple = |x: i32| x * 3;
    let composed_arc = add_arc.and_then(triple);

    // 原始 bi-transformer 仍然可用
    println!("   原始: 20 + 22 = {}", add_arc.transform(20, 22));
    println!("   组合: (5 + 3) * 3 = {}", composed_arc.transform(5, 3));
    println!();

    // 4. ArcBiTransformer::and_then - 可克隆
    println!("4. ArcBiTransformer::and_then - 可克隆");
    let subtract = ArcBiTransformer::new(|x: i32, y: i32| x - y);
    let abs = |x: i32| x.abs();
    let composed_abs = subtract.and_then(abs);
    let cloned = composed_abs.clone();

    println!("   原始: |10 - 15| = {}", composed_abs.transform(10, 15));
    println!("   克隆: |15 - 10| = {}", cloned.transform(15, 10));
    println!();

    // 5. RcBiTransformer::and_then - 单线程共享
    println!("5. RcBiTransformer::and_then - 单线程共享");
    let divide = RcBiTransformer::new(|x: i32, y: i32| x / y);
    let square = |x: i32| x * x;
    let composed_rc = divide.and_then(square);

    println!("   原始: 20 / 4 = {}", divide.transform(20, 4));
    println!("   组合: (20 / 4)² = {}", composed_rc.transform(20, 4));
    println!();

    // 6. 类型转换示例
    println!("6. 类型转换示例");
    let concat = BoxBiTransformer::new(|s1: String, s2: String| format!("{} {}", s1, s2));
    let to_uppercase = |s: String| s.to_uppercase();
    let get_length = |s: String| s.len();

    let uppercase_pipeline = concat.and_then(to_uppercase);
    println!(
        "   \"hello\" + \"world\" -> 大写: {}",
        uppercase_pipeline.transform("hello".to_string(), "world".to_string())
    );

    let concat2 = BoxBiTransformer::new(|s1: String, s2: String| format!("{} {}", s1, s2));
    let length_pipeline = concat2.and_then(get_length);
    println!(
        "   \"hello\" + \"world\" -> 长度: {}",
        length_pipeline.transform("hello".to_string(), "world".to_string())
    );
    println!();

    // 7. 实际应用：计算器
    println!("7. 实际应用：计算器");
    let calculate = BoxBiTransformer::new(|a: f64, b: f64| a + b);
    let round = |x: f64| x.round();
    let to_int = |x: f64| x as i32;

    let calculator = calculate.and_then(round).and_then(to_int);
    println!(
        "   3.7 + 4.8 -> 四舍五入 -> 整数: {}",
        calculator.transform(3.7, 4.8)
    );
    println!();

    // 8. 错误处理示例
    println!("8. 错误处理示例");
    let safe_divide = BoxBiTransformer::new(|x: i32, y: i32| -> Result<i32, String> {
        if y == 0 {
            Err("除数不能为零".to_string())
        } else {
            Ok(x / y)
        }
    });

    let format_result = |res: Result<i32, String>| match res {
        Ok(v) => format!("成功: {}", v),
        Err(e) => format!("错误: {}", e),
    };

    let safe_calculator = safe_divide.and_then(format_result);
    println!("   10 / 2 = {}", safe_calculator.transform(10, 2));
    println!("   10 / 0 = {}", safe_calculator.transform(10, 0));
    println!();

    // 9. 复杂数据结构
    println!("9. 复杂数据结构");
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    let create_point = BoxBiTransformer::new(|x: i32, y: i32| Point { x, y });
    let distance_from_origin = |p: Point| ((p.x * p.x + p.y * p.y) as f64).sqrt();
    let format_distance = |d: f64| format!("{:.2}", d);

    let point_processor = create_point
        .and_then(distance_from_origin)
        .and_then(format_distance);
    println!(
        "   点(3, 4)到原点的距离: {}",
        point_processor.transform(3, 4)
    );
    println!();

    // 10. 与 when 结合使用
    println!("10. 与 when 结合使用");
    let add_when = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply_when = BoxBiTransformer::new(|x: i32, y: i32| x * y);

    let conditional = add_when
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply_when);

    let double_result = |x: i32| x * 2;
    let final_transformer = conditional.and_then(double_result);

    println!(
        "   正数相加再翻倍: (5 + 3) * 2 = {}",
        final_transformer.transform(5, 3)
    );
    println!(
        "   负数相乘再翻倍: (-5 * 3) * 2 = {}",
        final_transformer.transform(-5, 3)
    );

    println!("\n=== 演示完成 ===");
}
