/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 Fn -> Function 的 blanket implementations

use prism3_function::Function;

fn main() {
    println!("=== 测试 Fn -> Function ===");
    test_function();
}

fn test_function() {
    // 测试函数指针
    fn double(x: &i32) -> i32 {
        x * 2
    }
    assert_eq!(double.apply(&21), 42);
    println!("✓ 函数指针测试通过: double.apply(&21) = 42");

    // 测试闭包
    let triple = |x: &i32| x * 3;
    assert_eq!(triple.apply(&14), 42);
    println!("✓ 闭包测试通过: triple.apply(&14) = 42");

    // 测试转换为 BoxFunction
    let quad = |x: &i32| x * 4;
    let boxed = Function::into_box(quad);
    assert_eq!(boxed.apply(&10), 40);
    println!("✓ into_box() 测试通过");

    // 测试转换为 RcFunction
    let times_five = |x: &i32| x * 5;
    let rc = Function::into_rc(times_five);
    assert_eq!(rc.apply(&8), 40);
    println!("✓ into_rc() 测试通过");

    // 测试转换为 ArcFunction
    let times_six = |x: &i32| x * 6;
    let arc = Function::into_arc(times_six);
    assert_eq!(arc.apply(&7), 42);
    println!("✓ into_arc() 测试通过");
}
