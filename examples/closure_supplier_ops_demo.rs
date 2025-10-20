/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! 演示闭包 Supplier 操作的示例
//!
//! 本示例展示如何在闭包上直接使用 `map`、`filter`、`zip` 和 `memoize` 操作。

use prism3_function::{FnSupplierOps, Supplier};

fn main() {
    println!("=== 闭包 Supplier 操作演示 ===\n");

    // 1. FnMut 闭包使用 map
    println!("1. FnMut 闭包使用 map:");
    let mut counter = 0;
    let mut mapped = (move || {
        counter += 1;
        counter
    })
    .map(|x| x * 2);

    println!("   第一次调用: {}", mapped.get());
    println!("   第二次调用: {}\n", mapped.get());

    // 2. FnMut 闭包使用 filter
    println!("2. FnMut 闭包使用 filter:");
    let mut counter2 = 0;
    let mut filtered = (move || {
        counter2 += 1;
        counter2
    })
    .filter(|x| x % 2 == 0);

    println!("   第一次调用 (奇数): {:?}", filtered.get());
    println!("   第二次调用 (偶数): {:?}\n", filtered.get());

    // 3. FnMut 闭包使用 memoize
    println!("3. FnMut 闭包使用 memoize:");
    let mut call_count = 0;
    let mut memoized = (move || {
        call_count += 1;
        println!("   底层函数被调用了 {} 次", call_count);
        42
    })
    .memoize();

    println!("   第一次调用: {}", memoized.get());
    println!("   第二次调用: {}", memoized.get());
    println!("   第三次调用: {}\n", memoized.get());

    // 4. Fn 闭包使用 map (Fn 也实现了 FnMut，所以可以使用 FnSupplierOps)
    println!("4. Fn 闭包使用 map:");
    let mut mapped_readonly = (|| 10).map(|x| x * 3).map(|x| x + 5);
    println!("   结果: {}\n", mapped_readonly.get());

    // 5. Fn 闭包使用 filter (Fn 也实现了 FnMut，所以可以使用 FnSupplierOps)
    println!("5. Fn 闭包使用 filter:");
    let mut filtered_readonly = (|| 42).filter(|x| x % 2 == 0);
    println!("   过滤偶数: {:?}\n", filtered_readonly.get());

    // 6. 链式操作
    println!("6. 链式操作:");
    let mut counter3 = 0;
    let mut chained = (move || {
        counter3 += 1;
        counter3
    })
    .map(|x| x * 2)
    .filter(|x| *x > 5)
    .map(|opt: Option<i32>| opt.unwrap_or(0));

    println!("   第一次调用: {}", chained.get()); // 2, filtered out
    println!("   第二次调用: {}", chained.get()); // 4, filtered out
    println!("   第三次调用: {}", chained.get()); // 6, passed
    println!("   第四次调用: {}\n", chained.get()); // 8, passed

    println!("=== 演示完成 ===");
}
