use prism3_function::{ArcPredicate, BoxPredicate, FnPredicateOps, Predicate, RcPredicate};
use std::thread;

fn main() {
    println!("=== Predicate 新 API 示例 ===\n");

    // Example 1: BoxPredicate - 单一所有权
    println!("1. BoxPredicate 示例 (单一所有权):");
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

    // 组合谓词 - 消耗所有权
    let is_positive_and_even = is_positive.and(is_even);

    let numbers = vec![-4, -3, 0, 3, 4, 7, 10];
    println!("   测试数字: {:?}", numbers);
    println!("   正数且偶数:");
    for num in &numbers {
        if is_positive_and_even.test(num) {
            println!("     {}", num);
        }
    }
    println!();

    // Example 2: 使用闭包的 FnPredicateOps
    println!("2. 闭包组合示例 (FnPredicateOps):");
    let is_positive = |x: &i32| *x > 0;
    let is_even = |x: &i32| x % 2 == 0;

    // 闭包可以直接使用组合方法,返回 BoxPredicate
    let combined = is_positive.and(is_even);

    println!("   4 是正偶数: {}", combined.test(&4));
    println!("   3 是正偶数: {}", combined.test(&3));
    println!();

    // Example 3: ArcPredicate - 共享所有权,线程安全
    println!("3. ArcPredicate 示例 (共享所有权,线程安全):");
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

    // 使用 &self,不消耗所有权
    let combined = is_positive.and(&is_even);

    println!("   主线程测试: 4 => {}", combined.test(&4));

    // 可以克隆到其他线程
    let combined_clone = combined.clone();
    let handle = thread::spawn(move || {
        println!("   子线程测试: 10 => {}", combined_clone.test(&10));
    });
    handle.join().unwrap();

    // 原始谓词仍然可用
    println!("   is_positive 仍可用: 5 => {}", is_positive.test(&5));
    println!("   is_even 仍可用: 6 => {}", is_even.test(&6));
    println!();

    // Example 4: RcPredicate - 共享所有权,非线程安全
    println!("4. RcPredicate 示例 (共享所有权,非线程安全):");
    let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    let is_small = RcPredicate::new(|x: &i32| *x < 100);

    // 使用 &self,不消耗所有权
    let combined = is_positive.and(&is_small);

    println!("   50 是正数且小于100: {}", combined.test(&50));
    println!("   150 是正数且小于100: {}", combined.test(&150));

    // 原始谓词仍然可用
    println!("   is_positive 仍可用: 200 => {}", is_positive.test(&200));
    println!();

    // Example 5: 复杂逻辑组合
    println!("5. 复杂逻辑组合:");
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
    let is_large = ArcPredicate::new(|x: &i32| *x > 10);

    // (正数 AND 偶数) OR 大于10
    let complex = is_positive.and(&is_even).or(&is_large);

    let test_numbers = vec![-5, 2, 4, 7, 12, 15];
    println!("   测试数字: {:?}", test_numbers);
    println!("   满足 (正数且偶数) 或 大于10:");
    for num in &test_numbers {
        if complex.test(num) {
            println!("     {}", num);
        }
    }
    println!();

    // Example 6: NOT 操作
    println!("6. NOT 操作:");
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let is_not_positive = is_positive.not();

    println!("   负数和零:");
    for num in &[-5, -1, 0, 5, 10] {
        if is_not_positive.test(num) {
            println!("     {}", num);
        }
    }
    println!();

    // Example 7: XOR 操作
    println!("7. XOR 操作:");
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
    let xor_result = is_positive.xor(&is_even);

    println!("   正数或偶数,但不能同时满足:");
    for num in &[-4, -3, 0, 3, 4, 7, 10] {
        if xor_result.test(num) {
            println!("     {}", num);
        }
    }
    println!();

    // Example 8: 自定义类型
    println!("8. 自定义结构体:");

    #[derive(Debug)]
    struct Person {
        name: String,
        age: u32,
    }

    let is_adult = ArcPredicate::new(|p: &Person| p.age >= 18);
    let is_senior = ArcPredicate::new(|p: &Person| p.age >= 60);
    let is_working_age = is_adult.and(&is_senior.not());

    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 25,
        },
        Person {
            name: "Bob".to_string(),
            age: 15,
        },
        Person {
            name: "Charlie".to_string(),
            age: 65,
        },
    ];

    println!("   工作年龄 (18-60岁):");
    for person in &people {
        if is_working_age.test(person) {
            println!("     {} ({} 岁)", person.name, person.age);
        }
    }
    println!();

    // Example 9: 命名谓词
    println!("9. 命名谓词:");
    let pred = BoxPredicate::new(|x: &i32| *x > 0).with_name("is_positive");

    println!("   谓词名称: {}", pred.name().unwrap_or("unnamed"));
    println!("   测试 5: {}", pred.test(&5));
    println!();

    // Example 10: 使用 into_fn() 简化迭代器操作
    println!("10. 使用 into_fn() 简化迭代器操作:");

    // 传统方式需要 lambda 包装:
    // let predicate = BoxPredicate::new(|x: &i32| *x > 0);
    // let result: Vec<i32> = values.into_iter().filter(|v| predicate.test(v)).collect();

    // 新方式：使用 into_fn() 直接传递
    let predicate = BoxPredicate::new(|x: &i32| *x > 0);
    let values2 = vec![1, -2, 3, -4, 5];
    let result2: Vec<i32> = values2.into_iter().filter(predicate.into_fn()).collect();

    println!("   原始值: [1, -2, 3, -4, 5]");
    println!("   过滤后的正数: {:?}", result2);
    println!();

    // Example 11: into_fn() 与复杂谓词组合
    println!("11. into_fn() 与复杂谓词组合:");
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);
    let complex_predicate = is_positive.and(&is_even);

    let values = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let result: Vec<i32> = values
        .into_iter()
        .filter(complex_predicate.into_fn())
        .collect();

    println!("   原始值: [1, 2, 3, 4, 5, 6, 7, 8]");
    println!("   正数且偶数: {:?}", result);
    println!();

    // Example 12: into_fn() 与其他迭代器方法
    println!("12. into_fn() 与其他迭代器方法:");

    // take_while 示例
    let predicate = RcPredicate::new(|x: &i32| *x > 0);
    let values = vec![1, 2, 3, -1, 4, 5];
    let taken: Vec<i32> = values
        .iter()
        .copied()
        .take_while(predicate.into_fn())
        .collect();
    println!("   take_while(>0) from [1, 2, 3, -1, 4, 5]: {:?}", taken);

    // partition 示例
    let predicate = ArcPredicate::new(|x: &i32| *x > 0);
    let values = vec![1, -2, 3, -4, 5];
    let (positives, negatives): (Vec<i32>, Vec<i32>) =
        values.into_iter().partition(predicate.into_fn());
    println!(
        "   partition: positives={:?}, negatives={:?}",
        positives, negatives
    );
    println!();

    // Example 13: into_fn() 与字符串过滤
    println!("13. into_fn() 与字符串过滤:");
    let predicate = BoxPredicate::new(|s: &String| s.contains('a'));
    let words = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
        "date".to_string(),
    ];

    let result: Vec<String> = words.into_iter().filter(predicate.into_fn()).collect();
    println!("   包含 'a' 的单词: {:?}", result);
    println!();

    println!("=== 示例完成 ===");
}
