# MutatingFunction 系列设计文档

## 概述

`MutatingFunction` 系列为 Rust 提供了一套完整的"修改并返回"函数式抽象，填补了 `Function` 和 `Mutator` 之间的空白。这些抽象允许函数既修改输入值，又返回结果信息。

## 设计动机

### 问题背景

在现有的函数式抽象中：

- **Function**: `Fn(&T) -> R` - 只读输入，返回结果
- **Mutator**: `Fn(&mut T)` - 修改输入，无返回值

但在实际应用中，我们经常需要：
1. 修改输入值
2. 同时返回关于修改的信息（如旧值、修改计数、验证结果等）

### 解决方案

引入 `MutatingFunction` 系列，对应三种闭包类型：

| 闭包类型 | Trait | 特性 | 使用场景 |
|---------|------|------|---------|
| `Fn(&mut T) -> R` | `MutatingFunction` | 无状态，可重复调用 | 原子操作、缓存更新 |
| `FnMut(&mut T) -> R` | `StatefulMutatingFunction` | 有状态，可修改自身 | 计数器、累加器 |
| `FnOnce(&mut T) -> R` | `MutatingFunctionOnce` | 一次性，消耗自身 | 资源转移、验证修复 |

## 核心抽象

### 1. MutatingFunction - 无状态修改并返回

**定义**: 对应 `Fn(&mut T) -> R` 闭包

**特性**:
- 无状态操作（使用 `Fn` 而非 `FnMut`）
- 可以多次调用
- 不能修改捕获的环境

**实现类型**:
- `BoxMutatingFunction<T, R>` - 单一所有权，不可克隆
- `ArcMutatingFunction<T, R>` - 线程安全，可克隆
- `RcMutatingFunction<T, R>` - 单线程，可克隆

**使用场景**:
- 原子操作：递增计数器并返回新值
- 缓存更新：更新缓存并返回旧值
- 数据验证：验证并修正数据，返回验证结果
- 事件处理：处理事件并返回是否继续

**示例**:

```rust
use prism3_function::{MutatingFunction, BoxMutatingFunction};

// 递增并返回新值
let incrementer = BoxMutatingFunction::new(|x: &mut i32| {
    *x += 1;
    *x
});

let mut value = 5;
let result = incrementer.apply(&mut value);
assert_eq!(value, 6);
assert_eq!(result, 6);

// 缓存更新模式
use std::collections::HashMap;

let updater = BoxMutatingFunction::new(
    |cache: &mut HashMap<String, i32>| {
        cache.insert("key".to_string(), 42)
    }
);

let mut cache = HashMap::new();
cache.insert("key".to_string(), 10);
let old_value = updater.apply(&mut cache);
assert_eq!(old_value, Some(10));
assert_eq!(cache.get("key"), Some(&42));
```

**方法链式调用**:

```rust
use prism3_function::{MutatingFunction, BoxMutatingFunction};

let chained = BoxMutatingFunction::new(|x: &mut i32| {
    *x *= 2;
    *x
})
.and_then(|x: &mut i32| {
    *x += 10;
    *x
});

let mut value = 5;
let result = chained.apply(&mut value);
assert_eq!(value, 20); // (5 * 2) + 10
assert_eq!(result, 20);
```

### 2. StatefulMutatingFunction - 有状态修改并返回

**定义**: 对应 `FnMut(&mut T) -> R` 闭包

**特性**:
- 有状态操作（使用 `FnMut`）
- 可以修改捕获的环境
- 可以多次调用，每次调用可能改变内部状态

**实现类型**:
- `BoxStatefulMutatingFunction<T, R>` - 单一所有权
- `ArcStatefulMutatingFunction<T, R>` - 线程安全（使用 `Mutex`）
- `RcStatefulMutatingFunction<T, R>` - 单线程（使用 `RefCell`）

**使用场景**:
- 有状态计数器：跟踪修改次数
- 累加器：收集统计信息
- 速率限制器：跟踪调用次数并条件性修改
- 验证器：累积错误信息
- 有状态转换器：基于历史应用转换

**示例**:

```rust
use prism3_function::{StatefulMutatingFunction,
                      BoxStatefulMutatingFunction};

// 计数器：递增值并跟踪调用次数
let mut counter = {
    let mut call_count = 0;
    BoxStatefulMutatingFunction::new(move |x: &mut i32| {
        call_count += 1;
        *x += 1;
        call_count
    })
};

let mut value = 5;
assert_eq!(counter.apply(&mut value), 1);
assert_eq!(value, 6);
assert_eq!(counter.apply(&mut value), 2);
assert_eq!(value, 7);

// 累加器模式
let mut accumulator = {
    let mut sum = 0;
    BoxStatefulMutatingFunction::new(move |x: &mut i32| {
        *x *= 2;
        sum += *x;
        sum
    })
};

let mut value = 5;
assert_eq!(accumulator.apply(&mut value), 10);
assert_eq!(value, 10);

let mut value2 = 3;
assert_eq!(accumulator.apply(&mut value2), 16); // 10 + 6
assert_eq!(value2, 6);
```

**线程安全示例**:

```rust
use prism3_function::{StatefulMutatingFunction,
                      ArcStatefulMutatingFunction};
use std::thread;

let counter = {
    let mut count = 0;
    ArcStatefulMutatingFunction::new(move |x: &mut i32| {
        count += 1;
        *x *= 2;
        count
    })
};

let mut counter_clone = counter.clone();

let handle = thread::spawn(move || {
    let mut value = 5;
    counter_clone.apply(&mut value)
});

let result = handle.join().unwrap();
assert_eq!(result, 1);
```

### 3. MutatingFunctionOnce - 一次性修改并返回

**定义**: 对应 `FnOnce(&mut T) -> R` 闭包

**特性**:
- 一次性操作（使用 `FnOnce`）
- 消耗自身，只能调用一次
- 可以移动捕获的变量

**实现类型**:
- `BoxMutatingFunctionOnce<T, R>` - 单一所有权，一次性使用

**为什么只有 Box 变体？**

`FnOnce` 只能调用一次，这与 `Arc`/`Rc` 的共享所有权语义冲突：
- `Arc`/`Rc` 意味着多个所有者可能需要调用
- `FnOnce` 在调用后被消耗，无法再次调用
- 这种语义不兼容使得 `Arc`/`Rc` 变体没有意义

**使用场景**:
- 初始化后回调：移动数据，返回状态
- 资源转移并返回结果：移动 Vec，返回旧值
- 一次性复杂操作：需要移动捕获变量
- 验证并修复：修复数据一次，返回验证结果

**示例**:

```rust
use prism3_function::{MutatingFunctionOnce, BoxMutatingFunctionOnce};

// 资源转移模式
let data = vec![1, 2, 3];
let func = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
    let old_len = x.len();
    x.extend(data); // 移动 data
    old_len
});

let mut target = vec![0];
let old_len = func.apply_once(&mut target);
assert_eq!(old_len, 1);
assert_eq!(target, vec![0, 1, 2, 3]);

// 验证模式
struct Data {
    value: i32,
}

let validator = BoxMutatingFunctionOnce::new(|data: &mut Data| {
    if data.value < 0 {
        data.value = 0;
        Err("Fixed negative value")
    } else {
        Ok("Valid")
    }
});

let mut data = Data { value: -5 };
let result = validator.apply_once(&mut data);
assert_eq!(data.value, 0);
assert!(result.is_err());
```

**方法链式调用**:

```rust
use prism3_function::{MutatingFunctionOnce, BoxMutatingFunctionOnce};

let data1 = vec![1, 2];
let data2 = vec![3, 4];

let chained = BoxMutatingFunctionOnce::new(move |x: &mut Vec<i32>| {
    x.extend(data1);
    x.len()
})
.and_then(move |x: &mut Vec<i32>| {
    x.extend(data2);
    x.len()
});

let mut target = vec![0];
let final_len = chained.apply_once(&mut target);
assert_eq!(final_len, 5);
assert_eq!(target, vec![0, 1, 2, 3, 4]);
```

## 与相关类型的比较

### MutatingFunction vs Function vs Mutator

| 特性 | Function | Mutator | MutatingFunction |
|------|----------|---------|------------------|
| **输入** | `&T` | `&mut T` | `&mut T` |
| **修改输入？** | ❌ | ✅ | ✅ |
| **返回结果？** | ✅ | ❌ | ✅ |
| **使用场景** | 只读转换 | 就地修改 | 修改+返回信息 |

### 完整的函数式抽象矩阵

| 输入类型 | 无状态 (Fn) | 有状态 (FnMut) | 一次性 (FnOnce) |
|---------|------------|---------------|----------------|
| **&T** | Function | StatefulFunction | FunctionOnce |
| **&mut T** | MutatingFunction | StatefulMutatingFunction | MutatingFunctionOnce |
| **T** | Transformer | StatefulTransformer | TransformerOnce |

## 实现细节

### 所有权模型

#### Box - 单一所有权
- 不可克隆
- 零开销（无引用计数）
- `and_then` 消耗 `self`
- 适用于构建器模式和所有权自然流动的场景

#### Rc - 单线程共享
- 可克隆（通过 `RefCell` 实现内部可变性）
- 引用计数开销
- `and_then` 借用 `&self`
- 适用于单线程场景

#### Arc - 线程安全共享
- 可克隆（通过 `Mutex` 实现内部可变性）
- 引用计数 + 锁开销
- `and_then` 借用 `&self`
- 适用于多线程场景

### 方法 API 设计

所有类型都提供：

1. **构造方法**:
   - `new(f)` - 创建新实例
   - `identity()` - 创建恒等函数

2. **组合方法**:
   - `and_then(next)` - 链式调用
   - `map(mapper)` - 映射结果

3. **转换方法**:
   - `into_box()` / `to_box()` - 转换为 Box
   - `into_rc()` / `to_rc()` - 转换为 Rc
   - `into_arc()` / `to_arc()` - 转换为 Arc
   - `into_fn()` / `to_fn()` - 转换为闭包

### 自动 Trait 实现

所有闭包自动实现相应的 trait：

```rust
// Fn(&mut T) -> R 自动实现 MutatingFunction<T, R>
let closure = |x: &mut i32| {
    *x *= 2;
    *x
};

let mut value = 5;
assert_eq!(closure.apply(&mut value), 10);

// 可以直接使用扩展方法
let chained = (|x: &mut i32| {
    *x *= 2;
    *x
})
.and_then(|x: &mut i32| {
    *x += 10;
    *x
});
```

## 使用指南

### 选择合适的类型

#### 何时使用 MutatingFunction？
- 需要修改输入并返回信息
- 操作是无状态的
- 不需要跟踪调用历史

#### 何时使用 StatefulMutatingFunction？
- 需要修改输入并返回信息
- 需要维护内部状态（计数器、累加器等）
- 需要跟踪调用历史

#### 何时使用 MutatingFunctionOnce？
- 需要移动捕获的变量
- 操作只执行一次
- 需要转移资源所有权

### 选择所有权模型

| 场景 | 推荐类型 |
|------|---------|
| 单次使用，无需共享 | Box |
| 单线程，需要共享 | Rc |
| 多线程，需要共享 | Arc |
| 构建器模式 | Box |
| 事件处理（单线程） | Rc |
| 并发任务处理 | Arc |

## 性能考虑

### 性能排序（从快到慢）

1. **Box** - 零开销，直接调用
2. **Rc** - 引用计数 + `RefCell` 运行时借用检查
3. **Arc** - 原子引用计数 + `Mutex` 锁获取

### 优化建议

1. **优先使用 Box**：如果不需要共享，使用 Box 获得最佳性能
2. **避免不必要的克隆**：对于 Rc/Arc，只在需要时克隆
3. **批量操作**：对于 StatefulMutatingFunction，考虑批量处理以减少锁争用
4. **选择合适的所有权模型**：不要为了方便而牺牲性能

## 测试覆盖

所有类型都有全面的单元测试：

- 基本功能测试
- 方法链式调用测试
- 类型转换测试
- 线程安全测试（Arc 类型）
- 克隆行为测试（Rc/Arc 类型）

测试文件位置：
- `tests/functions/mutating_function_tests.rs`
- `tests/functions/stateful_mutating_function_tests.rs`
- `tests/functions/mutating_function_once_tests.rs`

## 未来扩展

可能的未来增强：

1. **条件执行**：类似 `Mutator` 的 `when()` 方法
2. **错误处理**：返回 `Result<R, E>` 的变体
3. **异步支持**：异步版本的 trait
4. **更多组合器**：`filter_map`、`flat_map` 等

## 作者

Haixing Hu

