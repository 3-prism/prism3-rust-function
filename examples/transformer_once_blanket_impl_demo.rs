/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 FnOnce -> TransformerOnce 的 blanket implementations

use prism3_function::TransformerOnce;

fn main() {
    println!("=== 测试 FnOnce -> TransformerOnce ===");
    test_transformer_once();
}

fn test_transformer_once() {
    // 测试函数指针
    fn parse(s: String) -> i32 {
        s.parse().unwrap_or(0)
    }
    assert_eq!(parse.transform("42".to_string()), 42);
    println!("✓ 函数指针测试通过: parse.transform(\"42\") = 42");

    // 测试消耗所有权的闭包
    let owned_value = String::from("hello");
    let consume = |s: String| format!("{} world", s);
    assert_eq!(consume.transform(owned_value), "hello world");
    println!("✓ 消耗所有权的闭包测试通过");

    // 测试转换为 BoxTransformerOnce
    let transform = |s: String| s.to_uppercase();
    let boxed = transform.into_box();
    assert_eq!(boxed.transform("hello".to_string()), "HELLO");
    println!("✓ into_box() 测试通过");

    // 测试 into_fn
    let transform2 = |s: String| s.len();
    let func = transform2.into_fn();
    assert_eq!(func("hello".to_string()), 5);
    println!("✓ into_fn() 测试通过");
}
