/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! 演示 Predicate 的 set_name 和 new_with_name 方法

use prism3_function::predicate::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};

fn main() {
    println!("=== Predicate 命名功能演示 ===\n");

    demo_box_predicate();
    demo_rc_predicate();
    demo_arc_predicate();
}

/// 演示 BoxPredicate 的命名功能
fn demo_box_predicate() {
    println!("1. BoxPredicate 命名功能");

    // 使用 new_with_name 创建带名称的谓词
    let pred1 = BoxPredicate::new_with_name("is_positive", |x: &i32| *x > 0);
    println!("   使用 new_with_name 创建:");
    println!("     名称: {:?}", pred1.name());
    println!("     测试 5: {}", pred1.test(&5));

    // 使用 set_name 为已存在的谓词设置名称
    let mut pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
    println!("\n   使用 new 创建后用 set_name:");
    println!("     初始名称: {:?}", pred2.name());
    pred2.set_name("is_even");
    println!("     设置后名称: {:?}", pred2.name());
    println!("     测试 4: {}", pred2.test(&4));

    // 组合谓词会自动生成新名称
    let pred3 = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
    let pred4 = BoxPredicate::new_with_name("even", |x: &i32| x % 2 == 0);
    let combined = pred3.and(pred4);
    println!("\n   组合谓词的名称:");
    println!("     自动生成的名称: {:?}", combined.name());
    println!("     测试 4: {}\n", combined.test(&4));
}

/// 演示 RcPredicate 的命名功能
fn demo_rc_predicate() {
    println!("2. RcPredicate 命名功能");

    // 使用 new_with_name
    let pred1 = RcPredicate::new_with_name("greater_than_10", |x: &i32| *x > 10);
    println!("   使用 new_with_name:");
    println!("     名称: {:?}", pred1.name());
    println!("     测试 15: {}", pred1.test(&15));

    // 使用 set_name
    let mut pred2 = RcPredicate::new(|x: &i32| *x < 100);
    println!("\n   使用 set_name:");
    println!("     初始名称: {:?}", pred2.name());
    pred2.set_name("less_than_100");
    println!("     设置后名称: {:?}", pred2.name());
    println!("     测试 50: {}", pred2.test(&50));

    // 克隆后名称也会被保留
    let pred3 = pred2.clone();
    println!("\n   克隆后名称保留:");
    println!("     克隆后的名称: {:?}", pred3.name());
    println!("     测试 80: {}\n", pred3.test(&80));
}

/// 演示 ArcPredicate 的命名功能
fn demo_arc_predicate() {
    println!("3. ArcPredicate 命名功能（线程安全）");

    // 使用 new_with_name
    let pred1 = ArcPredicate::new_with_name("is_uppercase", |s: &String| {
        s.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
    });
    println!("   使用 new_with_name:");
    println!("     名称: {:?}", pred1.name());
    println!("     测试 'HELLO': {}", pred1.test(&"HELLO".to_string()));

    // 使用 set_name
    let mut pred2 = ArcPredicate::new(|s: &String| s.len() > 5);
    println!("\n   使用 set_name:");
    println!("     初始名称: {:?}", pred2.name());
    pred2.set_name("longer_than_5");
    println!("     设置后名称: {:?}", pred2.name());
    println!(
        "     测试 'Hello World': {}",
        pred2.test(&"Hello World".to_string())
    );

    // 在线程间共享时名称也保留
    let pred3 = pred2.clone();
    let handle = std::thread::spawn(move || {
        let name = pred3.name().map(|s| s.to_string());
        let result = pred3.test(&"Threading".to_string());
        (name, result)
    });

    let (name, result) = handle.join().unwrap();
    println!("\n   线程中访问:");
    println!("     线程中的名称: {:?}", name);
    println!("     线程中测试 'Threading': {}", result);

    // 原始 predicate 仍然可用
    println!("\n   原始 predicate 仍可用:");
    println!("     原始名称: {:?}", pred2.name());
    println!("     测试 'Rust': {}\n", pred2.test(&"Rust".to_string()));
}
