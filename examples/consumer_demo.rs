/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Consumer 类型演示
//!
//! 本示例演示了 Consumer 的三种实现（BoxConsumer、ArcConsumer、RcConsumer）
//! 以及它们的各种使用方式。

use prism3_function::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
use std::thread;

fn main() {
    println!("=== Consumer 示例 ===\n");

    // ========================================================================
    // 示例 1: BoxConsumer 基本使用
    // ========================================================================
    println!("示例 1: BoxConsumer 基本使用");
    println!("{}", "-".repeat(50));

    let mut consumer = BoxConsumer::new(|x: &i32| {
        println!("处理值: {}", x * 2);
    });
    let value = 5;
    println!("初始值: {}", value);
    consumer.accept(&value);
    println!("执行 BoxConsumer 后\n");

    // ========================================================================
    // 示例 2: BoxConsumer 方法链
    // ========================================================================
    println!("示例 2: BoxConsumer 方法链");
    println!("{}", "-".repeat(50));

    use std::sync::{Arc, Mutex};
    let results = Arc::new(Mutex::new(Vec::new()));
    let r1 = results.clone();
    let r2 = results.clone();
    let r3 = results.clone();

    let mut chained = BoxConsumer::new(move |x: &i32| {
        r1.lock().unwrap().push(*x * 2);
    })
    .and_then(move |x: &i32| {
        r2.lock().unwrap().push(*x + 10);
    })
    .and_then(move |x: &i32| {
        r3.lock().unwrap().push(*x);
        println!("处理值: {}", x);
    });

    let value = 5;
    println!("初始值: {}", value);
    chained.accept(&value);
    println!("收集的结果: {:?}\n", *results.lock().unwrap());

    // ========================================================================
    // 示例 3: 闭包扩展方法
    // ========================================================================
    println!("示例 3: 闭包直接使用扩展方法");
    println!("{}", "-".repeat(50));

    let result = Arc::new(Mutex::new(0));
    let r1 = result.clone();
    let r2 = result.clone();

    let mut closure_chain = (move |x: &i32| {
        *r1.lock().unwrap() = *x * 2;
    }).and_then(move |_x: &i32| {
        *r2.lock().unwrap() += 10;
    });

    let value = 5;
    println!("初始值: {}", value);
    closure_chain.accept(&value);
    println!("结果: {}\n", *result.lock().unwrap());

    // ========================================================================
    // 示例 4: BoxConsumer 工厂方法
    // ========================================================================
    println!("示例 4: BoxConsumer 工厂方法");
    println!("{}", "-".repeat(50));

    // noop
    let mut noop = BoxConsumer::<i32>::noop();
    let value = 42;
    println!("noop 前: {}", value);
    noop.accept(&value);
    println!("noop 后: {} (未改变)\n", value);

    // print
    let mut print = BoxConsumer::<i32>::print();
    let value = 42;
    print.accept(&value);
    println!();

    // print_with
    let mut print_with = BoxConsumer::<i32>::print_with("值为: ");
    let value = 42;
    print_with.accept(&value);
    println!();

    // ========================================================================
    // 示例 5: 条件 Consumer
    // ========================================================================
    println!("示例 5: 条件 Consumer");
    println!("{}", "-".repeat(50));

    // if_then
    let mut increment_if_positive = BoxConsumer::if_then(
        |x: &i32| *x > 0,
        |x: &i32| println!("正数: {}", x)
    );

    let positive = 5;
    let negative = -5;
    print!("if_then - positive: {}, ", positive);
    increment_if_positive.accept(&positive);
    print!("negative: {}, ", negative);
    increment_if_positive.accept(&negative);
    println!();

    // if_then_else
    let mut adjust = BoxConsumer::if_then_else(
        |x: &i32| *x > 0,
        |x: &i32| println!("正数: {}", x),
        |x: &i32| println!("负数: {}", x),
    );

    let positive = 10;
    let negative = -10;
    print!("if_then_else - positive: {}, ", positive);
    adjust.accept(&positive);
    print!("negative: {}, ", negative);
    adjust.accept(&negative);
    println!();

    // ========================================================================
    // 示例 6: ArcConsumer - 多线程共享
    // ========================================================================
    println!("示例 6: ArcConsumer - 多线程共享");
    println!("{}", "-".repeat(50));

    let shared = ArcConsumer::new(|x: &i32| println!("处理值: {}", x * 2));

    // 克隆用于另一个线程
    let shared_clone = shared.clone();
    let handle = thread::spawn(move || {
        let value = 5;
        let mut consumer = shared_clone;
        consumer.accept(&value);
        value
    });

    // 主线程使用
    let value = 3;
    let mut consumer = shared;
    consumer.accept(&value);

    let thread_result = handle.join().unwrap();
    println!("线程结果: {}\n", thread_result);

    // ========================================================================
    // 示例 7: ArcConsumer 组合（不消耗原始 consumer）
    // ========================================================================
    println!("示例 7: ArcConsumer 组合（借用 &self）");
    println!("{}", "-".repeat(50));

    let double = ArcConsumer::new(|x: &i32| println!("double: {}", x * 2));
    let add_ten = ArcConsumer::new(|x: &i32| println!("add_ten: {}", x + 10));

    // 组合不消耗原始 consumer
    let pipeline1 = double.and_then(&add_ten);
    let pipeline2 = add_ten.and_then(&double);

    let value1 = 5;
    let mut p1 = pipeline1;
    print!("pipeline1 (double then add) 处理 5: ");
    p1.accept(&value1);

    let value2 = 5;
    let mut p2 = pipeline2;
    print!("pipeline2 (add then double) 处理 5: ");
    p2.accept(&value2);

    // double 和 add_ten 仍然可用
    let value3 = 10;
    let mut d = double;
    print!("原始 double 仍可用，处理 10: ");
    d.accept(&value3);
    println!();

    // ========================================================================
    // 示例 8: RcConsumer - 单线程共享
    // ========================================================================
    println!("示例 8: RcConsumer - 单线程共享");
    println!("{}", "-".repeat(50));

    let rc_consumer = RcConsumer::new(|x: &i32| println!("处理: {}", x * 2));

    // 克隆多个副本
    let clone1 = rc_consumer.clone();
    let clone2 = rc_consumer.clone();

    let value1 = 5;
    let mut c1 = clone1;
    print!("clone1 处理 5: ");
    c1.accept(&value1);

    let value2 = 3;
    let mut c2 = clone2;
    print!("clone2 处理 3: ");
    c2.accept(&value2);

    let value3 = 7;
    let mut c3 = rc_consumer;
    print!("原始 处理 7: ");
    c3.accept(&value3);
    println!();

    // ========================================================================
    // 示例 9: RcConsumer 组合（借用 &self）
    // ========================================================================
    println!("示例 9: RcConsumer 组合（借用 &self）");
    println!("{}", "-".repeat(50));

    let double = RcConsumer::new(|x: &i32| println!("double: {}", x * 2));
    let add_ten = RcConsumer::new(|x: &i32| println!("add_ten: {}", x + 10));

    let pipeline1 = double.and_then(&add_ten);
    let pipeline2 = add_ten.and_then(&double);

    let value1 = 5;
    let mut p1 = pipeline1;
    print!("pipeline1 (double then add) 处理 5: ");
    p1.accept(&value1);

    let value2 = 5;
    let mut p2 = pipeline2;
    print!("pipeline2 (add then double) 处理 5: ");
    p2.accept(&value2);
    println!();

    // ========================================================================
    // 示例 10: 统一的 Consumer trait
    // ========================================================================
    println!("示例 10: 统一的 Consumer trait");
    println!("{}", "-".repeat(50));

    fn apply_to_all<C: Consumer<i32>>(consumer: &mut C, values: &[i32]) {
        for value in values.iter() {
            consumer.accept(value);
        }
    }

    let values1 = vec![1, 2, 3, 4, 5];
    let mut box_con = BoxConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("使用 BoxConsumer 处理 {:?}: ", values1);
    apply_to_all(&mut box_con, &values1);
    println!();

    let values2 = vec![1, 2, 3, 4, 5];
    let mut arc_con = ArcConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("使用 ArcConsumer 处理 {:?}: ", values2);
    apply_to_all(&mut arc_con, &values2);
    println!();

    let values3 = vec![1, 2, 3, 4, 5];
    let mut rc_con = RcConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("使用 RcConsumer 处理 {:?}: ", values3);
    apply_to_all(&mut rc_con, &values3);
    println!();

    let values4 = vec![1, 2, 3, 4, 5];
    let mut closure = |x: &i32| print!("{} ", x * 2);
    print!("使用闭包 处理 {:?}: ", values4);
    apply_to_all(&mut closure, &values4);
    println!("\n");

    // ========================================================================
    // 示例 11: 复杂数据处理管道
    // ========================================================================
    println!("示例 11: 复杂数据处理管道");
    println!("{}", "-".repeat(50));

    let mut pipeline = BoxConsumer::new(|x: &i32| {
        // 验证：限制到 0-100
        let clamped = if *x < 0 {
            0
        } else if *x > 100 {
            100
        } else {
            *x
        };
        print!("验证后: {} -> ", clamped);
    })
    .and_then(|x: &i32| {
        // 归一化：缩放到 0-10
        let clamped = if *x < 0 { 0 } else if *x > 100 { 100 } else { *x };
        let normalized = clamped / 10;
        print!("归一化: {} -> ", normalized);
    })
    .and_then(|x: &i32| {
        // 转换：平方
        let clamped = if *x < 0 { 0 } else if *x > 100 { 100 } else { *x };
        let normalized = clamped / 10;
        let squared = normalized * normalized;
        println!("平方: {}", squared);
    });

    print!("-50: ");
    pipeline.accept(&-50);

    print!("200: ");
    pipeline.accept(&200);

    print!("30: ");
    pipeline.accept(&30);
    println!();

    // ========================================================================
    // 示例 12: 字符串处理
    // ========================================================================
    println!("示例 12: 字符串处理");
    println!("{}", "-".repeat(50));

    let mut string_processor = BoxConsumer::new(|s: &String| {
        let no_space: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        print!("去除空格: {} -> ", no_space);
    })
    .and_then(|s: &String| {
        let no_space: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let lower = no_space.to_lowercase();
        print!("小写: {} -> ", lower);
    })
    .and_then(|s: &String| {
        let no_space: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let lower = no_space.to_lowercase();
        println!("添加后缀: {}!!!", lower);
    });

    let text = String::from("Hello World");
    print!("原始: {} -> ", text);
    string_processor.accept(&text);
    println!();

    // ========================================================================
    // 示例 13: 类型转换
    // ========================================================================
    println!("示例 13: 类型转换");
    println!("{}", "-".repeat(50));

    // 闭包 -> BoxConsumer
    let closure = |x: &i32| print!("处理: {} ", x * 2);
    let mut box_con = closure.into_box();
    let value = 5;
    print!("闭包 -> BoxConsumer: ");
    box_con.accept(&value);
    println!();

    // 闭包 -> RcConsumer
    let closure = |x: &i32| print!("处理: {} ", x * 2);
    let mut rc_con = closure.into_rc();
    let value = 5;
    print!("闭包 -> RcConsumer: ");
    rc_con.accept(&value);
    println!();

    // 闭包 -> ArcConsumer
    let closure = |x: &i32| print!("处理: {} ", x * 2);
    let mut arc_con = closure.into_arc();
    let value = 5;
    print!("闭包 -> ArcConsumer: ");
    arc_con.accept(&value);
    println!();

    // BoxConsumer -> RcConsumer
    let box_con = BoxConsumer::new(|x: &i32| print!("处理: {} ", x * 2));
    let mut rc_con = box_con.into_rc();
    let value = 5;
    print!("BoxConsumer -> RcConsumer: ");
    rc_con.accept(&value);
    println!();

    // RcConsumer -> BoxConsumer
    let rc_con = RcConsumer::new(|x: &i32| print!("处理: {} ", x * 2));
    let mut box_con = rc_con.into_box();
    let value = 5;
    print!("RcConsumer -> BoxConsumer: ");
    box_con.accept(&value);
    println!("\n");

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

    let mut processor = BoxConsumer::new(|p: &Point| println!("x * 2 = {}", p.x * 2))
        .and_then(|p: &Point| println!("y * 2 = {}", p.y * 2))
        .and_then(|p: &Point| println!("x + y = {}", p.x + p.y));

    let point = Point { x: 3, y: 4 };
    println!("处理点: {:?}", point);
    processor.accept(&point);
    println!();

    println!("=== 所有示例完成 ===");
}
