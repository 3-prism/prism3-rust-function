/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BoxMapperOnce, FnMapperOnceOps, MapperOnce};

fn main() {
    println!("=== MapperOnce 演示 ===\n");

    // 1. 基本的 BoxMapperOnce
    println!("1. 基本的 BoxMapperOnce");
    let parse = BoxMapperOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
    let result = parse.apply_once("42".to_string());
    println!("   解析 '42': {}", result);

    // 2. identity mapper
    println!("\n2. Identity Mapper");
    let identity = BoxMapperOnce::<i32, i32>::identity();
    let result = identity.apply_once(42);
    println!("   identity(42): {}", result);

    // 3. constant mapper
    println!("\n3. Constant Mapper");
    let constant = BoxMapperOnce::constant("hello");
    let result = constant.apply_once(123);
    println!("   constant(123): {}", result);

    // 4. and_then 组合
    println!("\n4. and_then 组合");
    let add_one = BoxMapperOnce::new(|x: i32| x + 1);
    let double = |x: i32| x * 2;
    let composed = add_one.and_then(double);
    let result = composed.apply_once(5);
    println!("   (5 + 1) * 2 = {}", result);

    // 5. compose 组合
    println!("\n5. compose 组合");
    let double = BoxMapperOnce::new(|x: i32| x * 2);
    let add_one = |x: i32| x + 1;
    let composed = double.compose(add_one);
    let result = composed.apply_once(5);
    println!("   double(5 + 1) = {}", result);

    // 6. 条件映射
    println!("\n6. 条件映射");
    let double = BoxMapperOnce::new(|x: i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    let result1 = conditional.apply_once(5);
    println!("   条件(5 > 0): double(5) = {}", result1);

    let double2 = BoxMapperOnce::new(|x: i32| x * 2);
    let conditional2 = double2.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    let result2 = conditional2.apply_once(-5);
    println!("   条件(-5 > 0): negate(-5) = {}", result2);

    // 7. 管道操作
    println!("\n7. 管道操作");
    let add_one = BoxMapperOnce::new(|x: i32| x + 1);
    let pipeline = add_one.and_then(|x| x * 2).and_then(|x| x - 3);
    let result = pipeline.apply_once(5);
    println!("   ((5 + 1) * 2) - 3 = {}", result);

    // 8. 闭包扩展方法
    println!("\n8. 闭包扩展方法 (FnMapperOnceOps)");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let double = |x: i32| x * 2;
    let composed = parse.and_then(double);
    let result = composed.apply_once("21".to_string());
    println!("   parse('21') * 2 = {}", result);

    // 9. 类型转换
    println!("\n9. 类型转换");
    let to_string = BoxMapperOnce::new(|x: i32| x.to_string());
    let add_suffix = to_string.and_then(|s| format!("{}_suffix", s));
    let result = add_suffix.apply_once(42);
    println!("   42.to_string() + '_suffix' = {}", result);

    // 10. 消费所有权
    println!("\n10. 消费所有权");
    let vec = vec![1, 2, 3, 4, 5];
    let sum = BoxMapperOnce::new(|v: Vec<i32>| v.iter().sum::<i32>());
    let result = sum.apply_once(vec);
    println!("   sum([1,2,3,4,5]) = {}", result);

    println!("\n=== MapperOnce 演示完成 ===");
}
