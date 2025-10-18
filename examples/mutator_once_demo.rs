/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Demo
//!
//! 演示 MutatorOnce 的各种使用场景

use prism3_function::{BoxMutatorOnce, FnMutatorOnceOps, MutatorOnce};

fn main() {
    println!("=== MutatorOnce 示例 ===\n");

    // 1. 基本使用：移动捕获的变量
    println!("1. 基本使用：移动捕获的变量");
    let data = vec![1, 2, 3];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   正在添加数据: {:?}", data);
        x.extend(data);
    });

    let mut target = vec![0];
    mutator.mutate(&mut target);
    println!("   结果: {:?}\n", target);

    // 2. 方法链：组合多个操作
    println!("2. 方法链：组合多个操作");
    let prefix = vec![1, 2];
    let middle = vec![3, 4];
    let suffix = vec![5, 6];

    let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   添加前缀: {:?}", prefix);
        x.extend(prefix);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   添加中间: {:?}", middle);
        x.extend(middle);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   添加后缀: {:?}", suffix);
        x.extend(suffix);
    });

    let mut result = vec![0];
    chained.mutate(&mut result);
    println!("   结果: {:?}\n", result);

    // 3. 初始化器模式
    println!("3. 初始化器模式");

    struct Initializer {
        name: String,
        on_complete: Option<BoxMutatorOnce<Vec<String>>>,
    }

    impl Initializer {
        fn new<F>(name: impl Into<String>, callback: F) -> Self
        where
            F: FnOnce(&mut Vec<String>) + 'static,
        {
            Self {
                name: name.into(),
                on_complete: Some(BoxMutatorOnce::new(callback)),
            }
        }

        fn run(mut self, data: &mut Vec<String>) {
            println!("   初始化器 '{}' 正在运行", self.name);
            data.push(format!("Initialized by {}", self.name));

            if let Some(callback) = self.on_complete.take() {
                println!("   执行完成回调");
                callback.mutate(data);
            }
        }
    }

    let extra = vec!["extra1".to_string(), "extra2".to_string()];
    let init = Initializer::new("MainInit", move |values| {
        println!("   回调中添加额外数据: {:?}", extra);
        values.extend(extra);
    });

    let mut config = Vec::new();
    init.run(&mut config);
    println!("   最终配置: {:?}\n", config);

    // 4. 字符串构建器模式
    println!("4. 字符串构建器模式");
    let greeting = String::from("Hello, ");
    let name = String::from("Alice");
    let punctuation = String::from("!");

    let builder = BoxMutatorOnce::new(move |s: &mut String| {
        println!("   添加问候语: {}", greeting);
        s.insert_str(0, &greeting);
    })
    .and_then(move |s: &mut String| {
        println!("   添加名字: {}", name);
        s.push_str(&name);
    })
    .and_then(move |s: &mut String| {
        println!("   添加标点: {}", punctuation);
        s.push_str(&punctuation);
    })
    .and_then(|s: &mut String| {
        println!("   转换为大写");
        *s = s.to_uppercase();
    });

    let mut message = String::new();
    builder.mutate(&mut message);
    println!("   最终消息: {}\n", message);

    // 5. 闭包直接使用
    println!("5. 闭包直接使用");
    let data1 = vec![10, 20];
    let data2 = vec![30, 40];

    let chained_closure = (move |x: &mut Vec<i32>| {
        println!("   第一步: 添加 {:?}", data1);
        x.extend(data1);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   第二步: 添加 {:?}", data2);
        x.extend(data2);
    })
    .and_then(|x: &mut Vec<i32>| {
        println!("   第三步: 对每个元素乘以 2");
        x.iter_mut().for_each(|n| *n *= 2);
    });

    let mut values = vec![0];
    chained_closure.mutate(&mut values);
    println!("   结果: {:?}\n", values);

    // 6. 资源转移场景
    println!("6. 资源转移场景");
    let large_data = vec![1; 10];
    println!("   准备转移大型数据 (长度: {})", large_data.len());

    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   转移数据（移动而非克隆）");
        x.extend(large_data); // large_data 被移动而非克隆
    });

    let mut container = Vec::new();
    mutator.mutate(&mut container);
    println!("   容器中的数据长度: {}\n", container.len());

    // 7. 泛型函数使用
    println!("7. 泛型函数使用");

    fn apply_transformation<M: MutatorOnce<Vec<i32>>>(mutator: M, initial: Vec<i32>) -> Vec<i32> {
        let mut val = initial;
        mutator.mutate(&mut val);
        val
    }

    let data = vec![100, 200, 300];
    let result = apply_transformation(
        move |x: &mut Vec<i32>| {
            println!("   在泛型函数中添加: {:?}", data);
            x.extend(data);
        },
        vec![0],
    );
    println!("   结果: {:?}\n", result);

    // 8. 配置构建器
    println!("8. 配置构建器");

    struct Config {
        options: Vec<String>,
    }

    impl Config {
        fn new() -> Self {
            Self {
                options: Vec::new(),
            }
        }

        fn with_defaults(mut self) -> Self {
            println!("   添加默认选项");
            self.options.push("default1".to_string());
            self.options.push("default2".to_string());
            self
        }

        fn customize<F>(mut self, customizer: F) -> Self
        where
            F: FnOnce(&mut Vec<String>) + 'static,
        {
            println!("   应用自定义配置");
            customizer.mutate(&mut self.options);
            self
        }

        fn build(self) -> Self {
            println!("   配置构建完成");
            self
        }
    }

    let custom_opts = vec!["custom1".to_string(), "custom2".to_string()];
    let config = Config::new()
        .with_defaults()
        .customize(move |opts| {
            println!("   添加自定义选项: {:?}", custom_opts);
            opts.extend(custom_opts);
        })
        .build();

    println!("   最终选项: {:?}\n", config.options);

    println!("=== 示例完成 ===");
}
