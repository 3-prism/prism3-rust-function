/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 Fn -> Transformer 的 blanket implementations

use prism3_function::Transformer;

fn main() {
    println!("=== 测试 Fn -> Transformer ===");
    test_transformer();
}

fn test_transformer() {
    // 测试函数指针
    fn double(x: i32) -> i32 {
        x * 2
    }
    assert_eq!(double.transform(21), 42);
    println!("✓ 函数指针测试通过: double.transform(21) = 42");

    // 测试闭包
    let triple = |x: i32| x * 3;
    assert_eq!(triple.transform(14), 42);
    println!("✓ 闭包测试通过: triple.transform(14) = 42");

    // 测试转换为 BoxTransformer
    let quad = |x: i32| x * 4;
    let boxed = Transformer::into_box(quad);
    assert_eq!(boxed.transform(10), 40);
    println!("✓ into_box() 测试通过");

    // 测试转换为 RcTransformer
    let times_five = |x: i32| x * 5;
    let rc = Transformer::into_rc(times_five);
    assert_eq!(rc.transform(8), 40);
    println!("✓ into_rc() 测试通过");

    // 测试转换为 ArcTransformer
    let times_six = |x: i32| x * 6;
    let arc = Transformer::into_arc(times_six);
    assert_eq!(arc.transform(7), 42);
    println!("✓ into_arc() 测试通过");
}
