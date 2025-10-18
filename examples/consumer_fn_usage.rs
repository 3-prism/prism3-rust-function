/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! 演示 into_fn 和 to_fn 如何用在接受闭包的函数参数上

use prism3_function::{ArcConsumer, BoxConsumer, Consumer, RcConsumer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== Consumer into_fn/to_fn 使用示例 ===\n");

    // 示例 1: 使用 BoxConsumer::into_fn 传递给标准库的 map
    println!("1. BoxConsumer::into_fn 用于 Iterator::for_each");
    let log = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();
    let consumer = BoxConsumer::new(move |x: &i32| {
        l.lock().unwrap().push(*x * 2);
    });

    // 将 consumer 转换为闭包，传递给 for_each
    [1, 2, 3, 4, 5].iter().for_each(consumer.into_fn());
    println!("   结果: {:?}\n", *log.lock().unwrap());

    // 示例 2: 使用 ArcConsumer::to_fn 可以多次使用
    println!("2. ArcConsumer::to_fn 可以多次使用");
    let log2 = Arc::new(Mutex::new(Vec::new()));
    let l2 = log2.clone();
    let consumer2 = ArcConsumer::new(move |x: &i32| {
        l2.lock().unwrap().push(*x + 10);
    });

    // to_fn 不消费 consumer，可以多次调用
    [1, 2, 3].iter().for_each(consumer2.to_fn());
    println!("   第一次: {:?}", *log2.lock().unwrap());

    [4, 5].iter().for_each(consumer2.to_fn());
    println!("   第二次: {:?}\n", *log2.lock().unwrap());

    // 示例 3: 使用 RcConsumer::to_fn
    println!("3. RcConsumer::to_fn 用于单线程场景");
    let log3 = Rc::new(RefCell::new(Vec::new()));
    let l3 = log3.clone();
    let consumer3 = RcConsumer::new(move |x: &i32| {
        l3.borrow_mut().push(*x * 3);
    });

    [1, 2, 3, 4].iter().for_each(consumer3.to_fn());
    println!("   结果: {:?}\n", *log3.borrow());

    // 示例 4: 在自定义函数中使用
    println!("4. 在自定义函数中使用");
    fn process_items<F>(items: Vec<i32>, consumer: F)
    where
        F: FnMut(&i32),
    {
        items.iter().for_each(consumer);
    }

    let log4 = Arc::new(Mutex::new(Vec::new()));
    let l4 = log4.clone();
    let consumer4 = BoxConsumer::new(move |x: &i32| {
        l4.lock().unwrap().push(*x * 5);
    });

    // 使用 into_fn 将 Consumer 转换为闭包传递给函数
    process_items(vec![1, 2, 3], consumer4.into_fn());
    println!("   结果: {:?}\n", *log4.lock().unwrap());

    // 示例 5: 链式操作后使用 into_fn
    println!("5. 链式操作后使用 into_fn");
    let log5 = Arc::new(Mutex::new(Vec::new()));
    let l5 = log5.clone();
    let l6 = log5.clone();

    let chained = BoxConsumer::new(move |x: &i32| {
        l5.lock().unwrap().push(format!("A: {}", x));
    })
    .and_then(move |x: &i32| {
        l6.lock().unwrap().push(format!("B: {}", x));
    });

    [1, 2].iter().for_each(chained.into_fn());
    println!("   结果: {:?}\n", *log5.lock().unwrap());

    println!("=== 演示完成 ===");
}
