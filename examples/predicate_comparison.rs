use prism3_function::{ArcPredicate, BoxPredicate, Predicate, RcPredicate};
use std::time::Instant;

fn main() {
    println!("=== Predicate 类型对比 ===\n");

    // BoxPredicate vs ArcPredicate vs RcPredicate

    println!("1. BoxPredicate - 单一所有权:");
    println!("   特点:");
    println!("   - 不可克隆");
    println!("   - 组合方法消耗 self");
    println!("   - 适合一次性使用的场景");
    println!();

    let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
    println!("   测试 5: {}", box_pred.test(&5));
    // box_pred 在这里仍然可用,因为 test 只是借用

    let box_pred2 = BoxPredicate::new(|x: &i32| x % 2 == 0);
    let combined = box_pred.and(box_pred2); // 消耗两个谓词
    println!("   4 是正偶数: {}", combined.test(&4));
    // box_pred 和 box_pred2 在这里已经不可用
    println!();

    println!("2. ArcPredicate - 共享所有权 + 线程安全:");
    println!("   特点:");
    println!("   - 可克隆");
    println!("   - 线程安全 (Send + Sync)");
    println!("   - 组合方法使用 &self,不消耗所有权");
    println!("   - 适合多线程共享场景");
    println!();

    let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    let arc_pred2 = ArcPredicate::new(|x: &i32| x % 2 == 0);

    let combined = arc_pred.and(&arc_pred2); // 不消耗所有权
    println!("   4 是正偶数: {}", combined.test(&4));

    // arc_pred 和 arc_pred2 仍然可用
    println!("   arc_pred 仍可用: {}", arc_pred.test(&5));
    println!("   arc_pred2 仍可用: {}", arc_pred2.test(&6));
    println!();

    println!("3. RcPredicate - 共享所有权 + 非线程安全:");
    println!("   特点:");
    println!("   - 可克隆");
    println!("   - 非线程安全 (不实现 Send/Sync)");
    println!("   - 组合方法使用 &self,不消耗所有权");
    println!("   - 比 Arc 性能更好(无原子操作)");
    println!("   - 适合单线程共享场景");
    println!();

    let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
    let rc_pred2 = RcPredicate::new(|x: &i32| x % 2 == 0);

    let combined = rc_pred.and(&rc_pred2); // 不消耗所有权
    println!("   4 是正偶数: {}", combined.test(&4));

    // rc_pred 和 rc_pred2 仍然可用
    println!("   rc_pred 仍可用: {}", rc_pred.test(&5));
    println!("   rc_pred2 仍可用: {}", rc_pred2.test(&6));
    println!();

    // 性能对比
    println!("4. 性能对比 (克隆 1,000,000 次):");

    let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _clone = arc_pred.clone();
    }
    let arc_time = start.elapsed();
    println!("   ArcPredicate 克隆: {:?}", arc_time);

    let rc_pred = RcPredicate::new(|x: &i32| *x > 0);
    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _clone = rc_pred.clone();
    }
    let rc_time = start.elapsed();
    println!("   RcPredicate 克隆: {:?}", rc_time);

    println!(
        "   RcPredicate 比 ArcPredicate 快 {:.2}x",
        arc_time.as_nanos() as f64 / rc_time.as_nanos() as f64
    );
    println!();

    println!("5. 使用场景建议:");
    println!("   - BoxPredicate: 简单的一次性谓词,或者作为函数参数/返回值");
    println!("   - ArcPredicate: 需要跨线程共享的谓词");
    println!("   - RcPredicate: 单线程内需要共享的谓词,追求更好性能");
    println!();

    println!("=== 对比完成 ===");
}
