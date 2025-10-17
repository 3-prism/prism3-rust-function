/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Mutator 类型演示
//!
//! 本示例演示了 Mutator 的三种实现（BoxMutator、ArcMutator、RcMutator）
//! 以及它们的各种使用方式。
//!
//! Mutator 用于修改值，与只读的 Consumer 不同。

use prism3_function::{ArcMutator, BoxMutator, FnMutatorOps, Mutator, RcMutator};
use std::thread;

fn main() {
    println!("=== Mutator 示例 ===\n");

    // ========================================================================
    // 示例 1: BoxMutator 基本使用
    // ========================================================================
    println!("示例 1: BoxMutator 基本使用");
    println!("{}", "-".repeat(50));

    let mut mutator = BoxMutator::new(|x: &mut i32| {
        *x *= 2;
    });
    let mut value = 5;
    println!("初始值: {}", value);
    mutator.accept(&mut value);
    println!("执行 BoxMutator 后: {}\n", value);

    // ========================================================================
    // 示例 2: BoxMutator 方法链
    // ========================================================================
    println!("示例 2: BoxMutator 方法链");
    println!("{}", "-".repeat(50));

    let mut chained = BoxMutator::new(|x: &mut i32| {
        *x *= 2; // 乘以2
    })
    .and_then(|x: &mut i32| {
        *x += 10; // 加10
    })
    .and_then(|x: &mut i32| {
        *x = *x * *x; // 平方
    });

    let mut value = 5;
    println!("初始值: {}", value);
    chained.accept(&mut value);
    println!("结果: {} (5 * 2 + 10 = 20, 20 * 20 = 400)\n", value);

    // ========================================================================
    // 示例 3: 闭包扩展方法
    // ========================================================================
    println!("示例 3: 闭包直接使用扩展方法");
    println!("{}", "-".repeat(50));

    let mut closure_chain = (|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

    let mut value = 5;
    println!("初始值: {}", value);
    closure_chain.accept(&mut value);
    println!("结果: {} (5 * 2 + 10 = 20)\n", value);

    // ========================================================================
    // 示例 4: BoxMutator 工厂方法
    // ========================================================================
    println!("示例 4: BoxMutator 工厂方法");
    println!("{}", "-".repeat(50));

    // noop
    let mut noop = BoxMutator::<i32>::noop();
    let mut value = 42;
    println!("noop 前: {}", value);
    noop.accept(&mut value);
    println!("noop 后: {} (未改变)\n", value);

    // ========================================================================
    // 示例 5: 条件 Mutator
    // ========================================================================
    println!("示例 5: 条件 Mutator");
    println!("{}", "-".repeat(50));

    // if_then
    let mut increment_if_positive = BoxMutator::if_then(|x: &i32| *x > 0, |x: &mut i32| *x += 1);

    let mut positive = 5;
    let mut negative = -5;
    println!(
        "if_then 前 - positive: {}, negative: {}",
        positive, negative
    );
    increment_if_positive.accept(&mut positive);
    increment_if_positive.accept(&mut negative);
    println!(
        "if_then 后 - positive: {}, negative: {}\n",
        positive, negative
    );

    // if_then_else
    let mut adjust = BoxMutator::if_then_else(
        |x: &i32| *x > 0,
        |x: &mut i32| *x *= 2,
        |x: &mut i32| *x = -*x,
    );

    let mut positive = 10;
    let mut negative = -10;
    println!(
        "if_then_else 前 - positive: {}, negative: {}",
        positive, negative
    );
    adjust.accept(&mut positive);
    adjust.accept(&mut negative);
    println!(
        "if_then_else 后 - positive: {}, negative: {}\n",
        positive, negative
    );

    // ========================================================================
    // 示例 6: ArcMutator - 多线程共享
    // ========================================================================
    println!("示例 6: ArcMutator - 多线程共享");
    println!("{}", "-".repeat(50));

    let shared = ArcMutator::new(|x: &mut i32| *x *= 2);

    // 克隆用于另一个线程
    let shared_clone = shared.clone();
    let handle = thread::spawn(move || {
        let mut value = 5;
        let mut mutator = shared_clone;
        mutator.accept(&mut value);
        println!("线程中: 5 * 2 = {}", value);
        value
    });

    // 主线程使用
    let mut value = 3;
    let mut mutator = shared;
    mutator.accept(&mut value);
    println!("主线程: 3 * 2 = {}", value);

    let thread_result = handle.join().unwrap();
    println!("线程结果: {}\n", thread_result);

    // ========================================================================
    // 示例 7: ArcMutator 组合（不消耗原始 mutator）
    // ========================================================================
    println!("示例 7: ArcMutator 组合（借用 &self）");
    println!("{}", "-".repeat(50));

    let double = ArcMutator::new(|x: &mut i32| *x *= 2);
    let add_ten = ArcMutator::new(|x: &mut i32| *x += 10);

    // 组合不消耗原始 mutator
    let pipeline1 = double.and_then(&add_ten);
    let pipeline2 = add_ten.and_then(&double);

    let mut value1 = 5;
    let mut p1 = pipeline1;
    p1.accept(&mut value1);
    println!("pipeline1 (double then add): 5 -> {}", value1);

    let mut value2 = 5;
    let mut p2 = pipeline2;
    p2.accept(&mut value2);
    println!("pipeline2 (add then double): 5 -> {}", value2);

    // double 和 add_ten 仍然可用
    let mut value3 = 10;
    let mut d = double;
    d.accept(&mut value3);
    println!("原始 double 仍可用: 10 -> {}\n", value3);

    // ========================================================================
    // 示例 8: RcMutator - 单线程共享
    // ========================================================================
    println!("示例 8: RcMutator - 单线程共享");
    println!("{}", "-".repeat(50));

    let rc_mutator = RcMutator::new(|x: &mut i32| *x *= 2);

    // 克隆多个副本
    let clone1 = rc_mutator.clone();
    let clone2 = rc_mutator.clone();

    let mut value1 = 5;
    let mut c1 = clone1;
    c1.accept(&mut value1);
    println!("clone1: 5 -> {}", value1);

    let mut value2 = 3;
    let mut c2 = clone2;
    c2.accept(&mut value2);
    println!("clone2: 3 -> {}", value2);

    let mut value3 = 7;
    let mut c3 = rc_mutator;
    c3.accept(&mut value3);
    println!("原始: 7 -> {}\n", value3);

    // ========================================================================
    // 示例 9: RcMutator 组合（借用 &self）
    // ========================================================================
    println!("示例 9: RcMutator 组合（借用 &self）");
    println!("{}", "-".repeat(50));

    let double = RcMutator::new(|x: &mut i32| *x *= 2);
    let add_ten = RcMutator::new(|x: &mut i32| *x += 10);

    let pipeline1 = double.and_then(&add_ten);
    let pipeline2 = add_ten.and_then(&double);

    let mut value1 = 5;
    let mut p1 = pipeline1;
    p1.accept(&mut value1);
    println!("pipeline1 (double then add): 5 -> {}", value1);

    let mut value2 = 5;
    let mut p2 = pipeline2;
    p2.accept(&mut value2);
    println!("pipeline2 (add then double): 5 -> {}\n", value2);

    // ========================================================================
    // 示例 10: 统一的 Mutator trait
    // ========================================================================
    println!("示例 10: 统一的 Mutator trait");
    println!("{}", "-".repeat(50));

    fn apply_to_all<M: Mutator<i32>>(mutator: &mut M, values: &mut [i32]) {
        for value in values.iter_mut() {
            mutator.accept(value);
        }
    }

    let mut values1 = vec![1, 2, 3, 4, 5];
    let mut box_mut = BoxMutator::new(|x: &mut i32| *x *= 2);
    println!("使用 BoxMutator: {:?}", values1);
    apply_to_all(&mut box_mut, &mut values1);
    println!("结果: {:?}", values1);

    let mut values2 = vec![1, 2, 3, 4, 5];
    let mut arc_mut = ArcMutator::new(|x: &mut i32| *x *= 2);
    println!("使用 ArcMutator: {:?}", values2);
    apply_to_all(&mut arc_mut, &mut values2);
    println!("结果: {:?}", values2);

    let mut values3 = vec![1, 2, 3, 4, 5];
    let mut rc_mut = RcMutator::new(|x: &mut i32| *x *= 2);
    println!("使用 RcMutator: {:?}", values3);
    apply_to_all(&mut rc_mut, &mut values3);
    println!("结果: {:?}", values3);

    let mut values4 = vec![1, 2, 3, 4, 5];
    let mut closure = |x: &mut i32| *x *= 2;
    println!("使用闭包: {:?}", values4);
    apply_to_all(&mut closure, &mut values4);
    println!("结果: {:?}\n", values4);

    // ========================================================================
    // 示例 11: 复杂数据处理管道
    // ========================================================================
    println!("示例 11: 复杂数据处理管道");
    println!("{}", "-".repeat(50));

    let mut pipeline = BoxMutator::new(|x: &mut i32| {
        // 验证：限制到 0-100
        if *x < 0 {
            *x = 0;
        }
        if *x > 100 {
            *x = 100;
        }
    })
    .and_then(|x: &mut i32| {
        // 归一化：缩放到 0-10
        *x /= 10;
    })
    .and_then(|x: &mut i32| {
        // 转换：平方
        *x = *x * *x;
    });

    let mut value1 = -50;
    pipeline.accept(&mut value1);
    println!("-50 -> {}", value1);

    let mut value2 = 200;
    pipeline.accept(&mut value2);
    println!("200 -> {}", value2);

    let mut value3 = 30;
    pipeline.accept(&mut value3);
    println!("30 -> {}\n", value3);

    // ========================================================================
    // 示例 12: 字符串处理
    // ========================================================================
    println!("示例 12: 字符串处理");
    println!("{}", "-".repeat(50));

    let mut string_processor = BoxMutator::new(|s: &mut String| s.retain(|c| !c.is_whitespace()))
        .and_then(|s: &mut String| *s = s.to_lowercase())
        .and_then(|s: &mut String| s.push_str("!!!"));

    let mut text = String::from("Hello World");
    println!("原始: {}", text);
    string_processor.accept(&mut text);
    println!("处理后: {}\n", text);

    // ========================================================================
    // 示例 13: 类型转换
    // ========================================================================
    println!("示例 13: 类型转换");
    println!("{}", "-".repeat(50));

    // 闭包 -> BoxMutator
    let closure = |x: &mut i32| *x *= 2;
    let mut box_mut = closure.into_box();
    let mut value = 5;
    box_mut.accept(&mut value);
    println!("闭包 -> BoxMutator: 5 -> {}", value);

    // 闭包 -> RcMutator
    let closure = |x: &mut i32| *x *= 2;
    let mut rc_mut = closure.into_rc();
    let mut value = 5;
    rc_mut.accept(&mut value);
    println!("闭包 -> RcMutator: 5 -> {}", value);

    // 闭包 -> ArcMutator
    let closure = |x: &mut i32| *x *= 2;
    let mut arc_mut = closure.into_arc();
    let mut value = 5;
    arc_mut.accept(&mut value);
    println!("闭包 -> ArcMutator: 5 -> {}", value);

    // BoxMutator -> RcMutator
    let box_mut = BoxMutator::new(|x: &mut i32| *x *= 2);
    let mut rc_mut = box_mut.into_rc();
    let mut value = 5;
    rc_mut.accept(&mut value);
    println!("BoxMutator -> RcMutator: 5 -> {}", value);

    // RcMutator -> BoxMutator
    let rc_mut = RcMutator::new(|x: &mut i32| *x *= 2);
    let mut box_mut = rc_mut.into_box();
    let mut value = 5;
    box_mut.accept(&mut value);
    println!("RcMutator -> BoxMutator: 5 -> {}\n", value);

    // ========================================================================
    // 示例 14: 自定义类型
    // ========================================================================
    println!("示例 14: 自定义类型");
    println!("{}", "-".repeat(50));

    #[derive(Debug, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    let mut processor = BoxMutator::new(|p: &mut Point| p.x *= 2)
        .and_then(|p: &mut Point| p.y *= 2)
        .and_then(|p: &mut Point| p.x += p.y);

    let mut point = Point { x: 3, y: 4 };
    println!("原始点: {:?}", point);
    processor.accept(&mut point);
    println!("处理后: {:?}\n", point);

    println!("=== 所有示例完成 ===");
}

