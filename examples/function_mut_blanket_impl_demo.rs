/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 FnMut -> FunctionMut 的 blanket implementations

use prism3_function::FunctionMut;

fn main() {
    println!("=== 测试 FnMut -> FunctionMut ===");
    test_function_mut();
}

fn test_function_mut() {
    // 测试转换为 BoxFunctionMut
    let increment = |x: &mut i32| {
        *x += 1;
        *x
    };
    let mut boxed = FunctionMut::into_box(increment);
    let mut val = 41;
    assert_eq!(boxed.apply(&mut val), 42);
    println!("✓ FnMut 可以转换为 BoxFunctionMut");

    // 测试直接使用
    let double = |x: &mut i32| *x * 2;
    let mut value = 21;
    // 直接调用不需要'static,因为没有通过 trait
    let result = double(&mut value);
    assert_eq!(result, 42);
    println!("✓ FnMut 可以直接使用");

    // 测试转换为 RcFunctionMut
    let increment2 = |x: &mut i32| {
        *x += 2;
        *x
    };
    let mut rc = FunctionMut::into_rc(increment2);
    let mut val2 = 40;
    assert_eq!(rc.apply(&mut val2), 42);
    println!("✓ into_rc() 测试通过");

    // 测试转换为 ArcFunctionMut
    let increment3 = |x: &mut i32| {
        *x += 3;
        *x
    };
    let mut arc = FunctionMut::into_arc(increment3);
    let mut val3 = 39;
    assert_eq!(arc.apply(&mut val3), 42);
    println!("✓ into_arc() 测试通过");
}

