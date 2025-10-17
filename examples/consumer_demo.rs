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
//!
//! Consumer 用于消费（读取）值，不修改原始值。
//! 如果需要修改值，请参考 mutator_demo.rs

use prism3_function::{ArcConsumer, BoxConsumer, Consumer, FnConsumerOps, RcConsumer};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    println!("=== Consumer 示例 ===\n");
    println!("注意：Consumer 只读取值，不修改原始值");
    println!("如需修改值，请参考 mutator_demo.rs\n");

    // ========================================================================
    // 示例 1: BoxConsumer 基本使用
    // ========================================================================
    println!("示例 1: BoxConsumer 基本使用");
    println!("{}", "-".repeat(50));

    let mut consumer = BoxConsumer::new(|x: &i32| {
        println!("读取并计算: {} * 2 = {}", x, x * 2);
    });
    let value = 5;
    println!("值: {}", value);
    consumer.accept(&value);
    println!("值仍为: {} (未被修改)\n", value);

    // ========================================================================
    // 示例 2: BoxConsumer 方法链
    // ========================================================================
    println!("示例 2: BoxConsumer 方法链");
    println!("{}", "-".repeat(50));

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
    println!("收集的结果: {:?}", *results.lock().unwrap());
    println!("原始值: {} (未被修改)\n", value);

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
    })
    .and_then(move |_x: &i32| {
        *r2.lock().unwrap() += 10;
    });

    let value = 5;
    println!("初始值: {}", value);
    closure_chain.accept(&value);
    println!("计算结果: {}", *result.lock().unwrap());
    println!("原始值: {} (未被修改)\n", value);

    // ========================================================================
    // 示例 4: BoxConsumer 工厂方法
    // ========================================================================
    println!("示例 4: BoxConsumer 工厂方法");
    println!("{}", "-".repeat(50));

    // noop
    println!("noop - 不做任何操作:");
    let mut noop = BoxConsumer::<i32>::noop();
    let value = 42;
    noop.accept(&value);
    println!("值: {}\n", value);

    // print
    print!("print - 打印值: ");
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
    let mut check_positive = BoxConsumer::if_then(
        |x: &i32| *x > 0,
        |x: &i32| println!("正数: {}", x),
    );

    let positive = 5;
    let negative = -5;
    print!("检查 {}: ", positive);
    check_positive.accept(&positive);
    print!("检查 {}: ", negative);
    check_positive.accept(&negative);
    println!("(负数不打印)\n");

    // if_then_else
    let mut categorize = BoxConsumer::if_then_else(
        |x: &i32| *x > 0,
        |x: &i32| println!("正数: {}", x),
        |x: &i32| println!("非正数: {}", x),
    );

    let positive = 10;
    let negative = -10;
    categorize.accept(&positive);
    categorize.accept(&negative);
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
    print!("pipeline1 处理 5: ");
    p1.accept(&value1);

    let value2 = 5;
    let mut p2 = pipeline2;
    print!("pipeline2 处理 5: ");
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
    print!("pipeline1 处理 5: ");
    p1.accept(&value1);

    let value2 = 5;
    let mut p2 = pipeline2;
    print!("pipeline2 处理 5: ");
    p2.accept(&value2);
    println!();

    // ========================================================================
    // 示例 10: 统一的 Consumer trait
    // ========================================================================
    println!("示例 10: 统一的 Consumer trait");
    println!("{}", "-".repeat(50));

    fn log_all<C: Consumer<i32>>(consumer: &mut C, values: &[i32]) {
        for value in values.iter() {
            consumer.accept(value);
        }
    }

    let values = vec![1, 2, 3, 4, 5];

    let mut box_con = BoxConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("BoxConsumer 处理 {:?}: ", values);
    log_all(&mut box_con, &values);
    println!();

    let mut arc_con = ArcConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("ArcConsumer 处理 {:?}: ", values);
    log_all(&mut arc_con, &values);
    println!();

    let mut rc_con = RcConsumer::new(|x: &i32| print!("{} ", x * 2));
    print!("RcConsumer 处理 {:?}: ", values);
    log_all(&mut rc_con, &values);
    println!();

    let mut closure = |x: &i32| print!("{} ", x * 2);
    print!("闭包 处理 {:?}: ", values);
    log_all(&mut closure, &values);
    println!("\n");

    // ========================================================================
    // 示例 11: 数据验证和日志记录
    // ========================================================================
    println!("示例 11: 数据验证和日志记录");
    println!("{}", "-".repeat(50));

    let validator = BoxConsumer::new(|x: &i32| {
        let status = if *x >= 0 && *x <= 100 {
            "有效"
        } else {
            "超出范围"
        };
        println!("验证 {}: {}", x, status);
    });

    let logger = BoxConsumer::new(|x: &i32| {
        println!("记录到日志: 值={}, 平方={}", x, x * x);
    });

    let mut pipeline = validator.and_then(logger);

    let test_values = vec![-50, 30, 200];
    for value in test_values {
        pipeline.accept(&value);
    }
    println!();

    // ========================================================================
    // 示例 12: 字符串分析
    // ========================================================================
    println!("示例 12: 字符串分析");
    println!("{}", "-".repeat(50));

    let mut string_analyzer = BoxConsumer::new(|s: &String| {
        println!("长度: {}", s.len());
    })
    .and_then(|s: &String| {
        println!("小写形式: {}", s.to_lowercase());
    })
    .and_then(|s: &String| {
        println!("大写形式: {}", s.to_uppercase());
    })
    .and_then(|s: &String| {
        let word_count = s.split_whitespace().count();
        println!("单词数: {}", word_count);
    });

    let text = String::from("Hello World");
    println!("分析文本: \"{}\"", text);
    string_analyzer.accept(&text);
    println!("原始文本: \"{}\" (未被修改)\n", text);

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

    let mut analyzer = BoxConsumer::new(|p: &Point| {
        println!("点的坐标: ({}, {})", p.x, p.y);
    })
    .and_then(|p: &Point| {
        let distance = ((p.x * p.x + p.y * p.y) as f64).sqrt();
        println!("到原点的距离: {:.2}", distance);
    })
    .and_then(|p: &Point| {
        let quadrant = match (p.x >= 0, p.y >= 0) {
            (true, true) => "第一象限",
            (false, true) => "第二象限",
            (false, false) => "第三象限",
            (true, false) => "第四象限",
        };
        println!("所在象限: {}", quadrant);
    });

    let point = Point { x: 3, y: 4 };
    println!("分析点: {:?}", point);
    analyzer.accept(&point);
    println!("原始点: {:?} (未被修改)\n", point);

    // ========================================================================
    // 示例 15: 数据收集和统计
    // ========================================================================
    println!("示例 15: 数据收集和统计");
    println!("{}", "-".repeat(50));

    let sum = Arc::new(Mutex::new(0));
    let count = Arc::new(Mutex::new(0));
    let sum_clone = sum.clone();
    let count_clone = count.clone();

    let mut collector = BoxConsumer::new(move |x: &i32| {
        *sum_clone.lock().unwrap() += *x;
        *count_clone.lock().unwrap() += 1;
    });

    let numbers = vec![10, 20, 30, 40, 50];
    println!("数字: {:?}", numbers);
    for num in &numbers {
        collector.accept(num);
    }

    let total = *sum.lock().unwrap();
    let cnt = *count.lock().unwrap();
    println!("总和: {}", total);
    println!("数量: {}", cnt);
    println!("平均值: {:.2}\n", total as f64 / cnt as f64);

    println!("=== 所有示例完成 ===");
    println!("\n提示：如需修改值的功能，请参考 mutator_demo.rs");
}
