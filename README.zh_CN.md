# Prism3 Function

Rust 常用函数式编程类型别名，提供类似 Java 的函数式接口。

## 概述

此 crate 提供常见函数式编程模式的类型别名，类似于 Java 的函数式接口。这些类型别名简化了函数式编程中的类型声明，提供了更好的可读性和可维护性。

## 特性

- **Predicate<T>**：表示接受类型 `T` 的引用并返回 `bool` 的函数
- 线程安全：所有类型都实现了 `Send + Sync`
- 易于与闭包和函数指针配合使用
- 与标准库集合和迭代器兼容

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
prism3-function = "0.1.0"
```

## 使用方法

### Predicate（断言）

```rust
use prism3_function::Predicate;

// 创建一个判断数字是否为偶数的 Predicate
let is_even: Predicate<i32> = Box::new(|x| x % 2 == 0);

assert!(is_even(&4));
assert!(!is_even(&3));

// 用于集合过滤
let numbers = vec![1, 2, 3, 4, 5, 6];
let even_numbers: Vec<i32> = numbers
    .into_iter()
    .filter(|x| is_even(x))
    .collect();

assert_eq!(even_numbers, vec![2, 4, 6]);
```

## 计划实现的类型

- `Consumer<T>`：消费一个值
- `Function<T, R>`：将类型 `T` 的值转换为类型 `R`
- `Supplier<T>`：提供一个值
- `Operator<T>`：一元操作符
- `BiOperator<T>`：二元操作符
- `Transformer<T>`：转换相同类型的值
- `Filter<T>`：过滤值

## 许可证

基于 Apache License, Version 2.0 许可。

## 作者

胡海星 <starfish.hu@gmail.com>

