/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFnFunction, BoxFnFunction, BoxFunction, FnFunctionOps, RcFnFunction};

fn main() {
    println!("=== Function Demo - 方案三：Trait + 多种实现 ===\n");

    // ========================================================================
    // BoxFunction - 一次性使用场景
    // ========================================================================
    println!("=== 1. BoxFunction - 一次性使用 ===");

    let double = BoxFunction::new(|x: i32| x * 2);
    println!(
        "BoxFunction::new(|x| x * 2).apply(21) = {}",
        double.apply(21)
    );
    // double 已被消耗，无法再次使用

    // 恒等函数
    let identity = BoxFunction::<i32, i32>::identity();
    println!("identity(42) = {}", identity.apply(42));

    // 常量函数
    let constant = BoxFunction::constant("hello");
    println!("constant(123) = {}", constant.apply(123));

    // 方法链组合
    let pipeline = BoxFunction::new(|x: i32| x + 1)
        .and_then(|x| x * 2)
        .and_then(|x| x.to_string());
    println!("pipeline(5) = {} (期望: \"12\")", pipeline.apply(5));

    println!();

    // ========================================================================
    // BoxFnFunction - 可重复调用，单一所有权
    // ========================================================================
    println!("=== 2. BoxFnFunction - 可重复调用，单一所有权 ===");

    let double_fn = BoxFnFunction::new(|x: i32| x * 2);
    println!("BoxFnFunction 可以多次调用:");
    println!("  double_fn.apply(21) = {}", double_fn.apply(21));
    println!("  double_fn.apply(42) = {}", double_fn.apply(42));
    println!("  double_fn.apply(10) = {}", double_fn.apply(10));

    // 组合（消耗所有权）
    let add_one = BoxFnFunction::new(|x: i32| x + 1);
    let double = BoxFnFunction::new(|x: i32| x * 2);
    let composed = add_one.and_then(double);
    println!("composed(5) = {} (期望: 12)", composed.apply(5));
    // add_one 和 double 已被消耗

    println!();

    // ========================================================================
    // ArcFnFunction - 多线程共享，可重复调用
    // ========================================================================
    println!("=== 3. ArcFnFunction - 多线程共享，可重复调用 ===");

    let arc_double = ArcFnFunction::new(|x: i32| x * 2);

    // 可以克隆
    let cloned = arc_double.clone();
    println!("ArcFnFunction 可以克隆:");
    println!("  arc_double.apply(21) = {}", arc_double.apply(21));
    println!("  cloned.apply(42) = {}", cloned.apply(42));

    // 组合不消耗所有权（使用 &self）
    let arc_add_one = ArcFnFunction::new(|x: i32| x + 1);
    let arc_double_2 = ArcFnFunction::new(|x: i32| x * 2);
    let arc_composed = arc_add_one.and_then(&arc_double_2);

    println!("组合后原始函数仍可用:");
    println!("  arc_add_one.apply(5) = {}", arc_add_one.apply(5));
    println!("  arc_double_2.apply(5) = {}", arc_double_2.apply(5));
    println!("  arc_composed.apply(5) = {}", arc_composed.apply(5));

    // 多线程使用
    println!("多线程使用:");
    use std::thread;
    let func_for_thread = arc_double.clone();
    let handle = thread::spawn(move || func_for_thread.apply(100));

    println!("  主线程: arc_double.apply(50) = {}", arc_double.apply(50));
    println!("  子线程: result = {}", handle.join().unwrap());

    println!();

    // ========================================================================
    // RcFnFunction - 单线程共享，可重复调用
    // ========================================================================
    println!("=== 4. RcFnFunction - 单线程共享，可重复调用 ===");

    let rc_double = RcFnFunction::new(|x: i32| x * 2);

    // 可以克隆
    let rc_cloned = rc_double.clone();
    println!("RcFnFunction 可以克隆:");
    println!("  rc_double.apply(21) = {}", rc_double.apply(21));
    println!("  rc_cloned.apply(42) = {}", rc_cloned.apply(42));

    // 组合不消耗所有权
    let rc_add_one = RcFnFunction::new(|x: i32| x + 1);
    let rc_double_2 = RcFnFunction::new(|x: i32| x * 2);
    let rc_composed = rc_add_one.and_then(&rc_double_2);

    println!("组合后原始函数仍可用:");
    println!("  rc_add_one.apply(5) = {}", rc_add_one.apply(5));
    println!("  rc_double_2.apply(5) = {}", rc_double_2.apply(5));
    println!("  rc_composed.apply(5) = {}", rc_composed.apply(5));

    println!();

    // ========================================================================
    // 闭包扩展方法
    // ========================================================================
    println!("=== 5. 闭包扩展方法 - FnFunctionOps ===");

    let closure_result = (|x: i32| x + 1)
        .and_then(|x| x * 2)
        .and_then(|x| x.to_string());

    println!(
        "闭包链式调用: (|x| x + 1).and_then(...).apply(5) = {}",
        closure_result.apply(5)
    );

    println!();

    // ========================================================================
    // Option 和 Result 辅助方法
    // ========================================================================
    println!("=== 6. Option 和 Result 辅助方法 ===");

    // map_option
    let double = |x: i32| x * 2;
    let option_double = BoxFunction::map_option(double);
    println!("map_option(Some(21)) = {:?}", option_double.apply(Some(21)));

    let double2 = |x: i32| x * 2;
    let option_double2 = BoxFunction::map_option(double2);
    println!("map_option(None) = {:?}", option_double2.apply(None));

    // map_result
    let double3 = |x: i32| x * 2;
    let result_double = BoxFunction::map_result(double3);
    println!(
        "map_result(Ok(21)) = {:?}",
        result_double.apply(Ok::<i32, &str>(21))
    );

    let double4 = |x: i32| x * 2;
    let result_double2 = BoxFunction::map_result(double4);
    println!(
        "map_result(Err(\"error\")) = {:?}",
        result_double2.apply(Err::<i32, &str>("error"))
    );

    // result_to_option
    let to_option = BoxFunction::<Result<i32, &str>, Option<i32>>::result_to_option();
    println!("result_to_option(Ok(42)) = {:?}", to_option.apply(Ok(42)));

    let to_option2 = BoxFunction::<Result<i32, &str>, Option<i32>>::result_to_option();
    println!(
        "result_to_option(Err(\"error\")) = {:?}",
        to_option2.apply(Err("error"))
    );

    // option_to_result
    let to_result = BoxFunction::option_to_result("missing");
    println!(
        "option_to_result(Some(42)) = {:?}",
        to_result.apply(Some(42))
    );

    let to_result2: BoxFunction<Option<i32>, Result<i32, &str>> =
        BoxFunction::option_to_result("missing");
    println!("option_to_result(None) = {:?}", to_result2.apply(None));

    // match_result
    let handle =
        BoxFunction::match_result(|x: i32| x.to_string(), |e: &str| format!("Error: {}", e));
    println!("match_result(Ok(42)) = {}", handle.apply(Ok(42)));

    let handle2 =
        BoxFunction::match_result(|x: i32| x.to_string(), |e: &str| format!("Error: {}", e));
    println!(
        "match_result(Err(\"failed\")) = {}",
        handle2.apply(Err("failed"))
    );

    // flatten_result
    let flatten =
        BoxFunction::<Result<Result<i32, &str>, &str>, Result<i32, &str>>::flatten_result();
    println!(
        "flatten_result(Ok(Ok(42))) = {:?}",
        flatten.apply(Ok(Ok(42)))
    );

    let flatten2 =
        BoxFunction::<Result<Result<i32, &str>, &str>, Result<i32, &str>>::flatten_result();
    println!(
        "flatten_result(Ok(Err(\"inner\"))) = {:?}",
        flatten2.apply(Ok(Err("inner")))
    );

    // flatten_option
    let flatten_opt = BoxFunction::<Option<Option<i32>>, Option<i32>>::flatten_option();
    println!(
        "flatten_option(Some(Some(42))) = {:?}",
        flatten_opt.apply(Some(Some(42)))
    );

    let flatten_opt2 = BoxFunction::<Option<Option<i32>>, Option<i32>>::flatten_option();
    println!(
        "flatten_option(Some(None)) = {:?}",
        flatten_opt2.apply(Some(None))
    );

    println!();

    // ========================================================================
    // 复杂场景示例
    // ========================================================================
    println!("=== 7. 复杂场景示例 ===");

    // 场景1：数据处理管道（一次性）
    println!("场景1：数据处理管道（BoxFunction）");
    let parse_and_process = BoxFunction::new(|s: String| s.parse::<i32>().unwrap_or(0))
        .and_then(|x| x * 2)
        .and_then(|x| format!("Result: {}", x));

    println!(
        "  parse_and_process(\"21\") = {}",
        parse_and_process.apply("21".to_string())
    );

    // 场景2：可重用的转换器（ArcFnFunction）
    println!("场景2：可重用的转换器（ArcFnFunction）");
    let parse = ArcFnFunction::new(|s: String| s.parse::<i32>().unwrap_or(0));
    let double = ArcFnFunction::new(|x: i32| x * 2);
    let to_string = ArcFnFunction::new(|x: i32| x.to_string());

    let pipeline = parse.and_then(&double).and_then(&to_string);

    println!("  pipeline(\"21\") = {}", pipeline.apply("21".to_string()));
    println!("  pipeline(\"10\") = {}", pipeline.apply("10".to_string()));
    println!(
        "  parse 仍可单独使用: parse(\"42\") = {}",
        parse.apply("42".to_string())
    );

    // 场景3：错误处理
    println!("场景3：错误处理");
    let divide = BoxFunction::new(|x: i32| {
        if x != 0 {
            Ok(100 / x)
        } else {
            Err("division by zero")
        }
    });

    let safe_divide = divide.and_then(|result| result.unwrap_or(0));
    println!("  safe_divide(10) = {}", safe_divide.apply(10));

    let divide2 = BoxFunction::new(|x: i32| {
        if x != 0 {
            Ok(100 / x)
        } else {
            Err("division by zero")
        }
    });
    let safe_divide2 = divide2.and_then(|result| result.unwrap_or(0));
    println!("  safe_divide(0) = {}", safe_divide2.apply(0));

    println!();

    // ========================================================================
    // 类型选择指南
    // ========================================================================
    println!("=== 8. 类型选择指南 ===");
    println!("BoxFunction:     一次性转换，构建后立即使用");
    println!("BoxFnFunction:   需要多次调用，但不需要克隆");
    println!("ArcFnFunction:   需要多次调用、克隆、跨线程使用");
    println!("RcFnFunction:    需要多次调用、克隆，仅在单线程内使用");

    println!("\n=== End of Function Demo ===");
}
