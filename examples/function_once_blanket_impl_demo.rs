/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 FnOnce -> FunctionOnce 的 blanket implementations

use prism3_function::FunctionOnce;

fn main() {
    println!("=== 测试 FnOnce -> FunctionOnce ===");
    test_function_once();
}

fn test_function_once() {
    // 测试函数指针
    fn parse(s: String) -> i32 {
        s.parse().unwrap_or(0)
    }
    assert_eq!(parse.apply("42".to_string()), 42);
    println!("✓ 函数指针测试通过: parse.apply(\"42\") = 42");

    // 测试消耗所有权的闭包
    let owned_value = String::from("hello");
    let consume = |s: String| format!("{} world", s);
    assert_eq!(consume.apply(owned_value), "hello world");
    println!("✓ 消耗所有权的闭包测试通过");

    // 测试转换为 BoxFunctionOnce
    let transform = |s: String| s.to_uppercase();
    let boxed = transform.into_box();
    assert_eq!(boxed.apply("hello".to_string()), "HELLO");
    println!("✓ into_box() 测试通过");

    // 测试 into_rc
    let transform2 = |s: String| s.len();
    let rc = transform2.into_rc();
    assert_eq!(rc.apply("hello".to_string()), 5);
    println!("✓ into_rc() 测试通过");

    // 测试 into_arc
    let transform3 = |s: String| s.chars().count();
    let arc = transform3.into_arc();
    assert_eq!(arc.apply("hello".to_string()), 5);
    println!("✓ into_arc() 测试通过");
}

