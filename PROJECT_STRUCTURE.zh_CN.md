# 项目结构

本文档描述 prism3-rust-function crate 的目录结构和组织方式。

## 目录布局

```
prism3-rust-function/
├── src/                    # 源代码
│   ├── consumers/          # Consumer 相关抽象
│   ├── predicates/         # Predicate 相关抽象
│   ├── transformers/       # Transformer 相关抽象
│   ├── functions/          # Function 相关抽象
│   ├── suppliers/          # Supplier 相关抽象
│   ├── mutators/           # Mutator 相关抽象
│   ├── comparator.rs       # Comparator 抽象（独立）
│   ├── tester.rs           # Tester 抽象（独立）
│   └── lib.rs              # 库根文件和重导出
├── tests/                  # 集成测试
│   ├── consumers/          # Consumer 测试
│   ├── predicates/         # Predicate 测试
│   ├── transformers/       # Transformer 测试
│   ├── functions/          # Function 测试
│   ├── suppliers/          # Supplier 测试
│   ├── mutators/           # Mutator 测试
│   ├── comparator_tests.rs # Comparator 测试（独立）
│   ├── tester_tests.rs     # Tester 测试（独立）
│   └── mod.rs              # 测试模块根文件
├── examples/               # 示例程序
│   ├── consumers/          # Consumer 示例
│   ├── predicates/         # Predicate 示例
│   ├── transformers/       # Transformer 示例
│   ├── suppliers/          # Supplier 示例
│   └── mutators/           # Mutator 示例
└── doc/                    # 设计文档
```

## 模块组织

### Consumers 模块 (`src/consumers/`)

包含所有 consumer 相关的函数式抽象：

- `consumer.rs` - 不可变 consumer (`Fn(&T)`)
- `consumer_once.rs` - 一次性 consumer (`FnOnce(&T)`)
- `stateful_consumer.rs` - 有状态 consumer (`FnMut(&T)`)
- `bi_consumer.rs` - 双参数 consumer (`Fn(&T, &U)`)
- `bi_consumer_once.rs` - 一次性双参数 consumer (`FnOnce(&T, &U)`)
- `stateful_bi_consumer.rs` - 有状态双参数 consumer
  (`FnMut(&T, &U)`)
- `mod.rs` - 模块导出

### Predicates 模块 (`src/predicates/`)

包含 predicate 相关的函数式抽象：

- `predicate.rs` - 单参数 predicate (`Fn(&T) -> bool`)
- `bi_predicate.rs` - 双参数 predicate (`Fn(&T, &U) -> bool`)
- `mod.rs` - 模块导出

### Transformers 模块 (`src/transformers/`)

包含 transformer 相关的函数式抽象：

- `transformer.rs` - 值转换器 (`Fn(T) -> R`)
- `transformer_once.rs` - 一次性转换器 (`FnOnce(T) -> R`)
- `stateful_transformer.rs` - 有状态转换器 (`FnMut(T) -> R`)
- `bi_transformer.rs` - 双参数转换器 (`Fn(T, U) -> R`)
- `bi_transformer_once.rs` - 一次性双参数转换器
  (`FnOnce(T, U) -> R`)
- `stateful_bi_transformer.rs` - 有状态双参数转换器
  (`FnMut(T, U) -> R`)
- `mod.rs` - 模块导出

### Functions 模块 (`src/functions/`)

包含 function 相关的抽象（基于引用的转换）：

- `function.rs` - 引用函数 (`Fn(&T) -> R`)
- `function_once.rs` - 一次性引用函数 (`FnOnce(&T) -> R`)
- `stateful_function.rs` - 有状态引用函数 (`FnMut(&T) -> R`)
- `mod.rs` - 模块导出

### Suppliers 模块 (`src/suppliers/`)

包含 supplier 相关的抽象（值生成器）：

- `supplier.rs` - 不可变 supplier (`Fn() -> T`)
- `supplier_once.rs` - 一次性 supplier (`FnOnce() -> T`)
- `stateful_supplier.rs` - 有状态 supplier (`FnMut() -> T`)
- `mod.rs` - 模块导出

### Mutators 模块 (`src/mutators/`)

包含 mutator 相关的抽象（就地修改）：

- `mutator.rs` - 有状态 mutator (`FnMut(&mut T)`)
- `mutator_once.rs` - 一次性 mutator (`FnOnce(&mut T)`)
- `stateful_mutator.rs` - 带额外状态的有状态 mutator
- `mod.rs` - 模块导出

### 独立模块

- `comparator.rs` - Comparator 抽象 (`Fn(&T, &T) -> Ordering`)
- `tester.rs` - Tester 抽象 (`Fn() -> bool`)

## 测试组织

`tests/` 目录镜像 `src/` 目录结构：

- 每个模块都有对应的测试目录
- 测试文件命名为 `{模块名}_tests.rs`
- 独立模块的测试位于 `tests/` 根目录

## 示例组织

`examples/` 目录按功能组织：

- 每个模块都有对应的示例目录
- 示例文件展示典型的使用模式
- 示例文件使用描述性命名（如 `consumer_demo.rs`,
  `predicate_demo.rs`）

## 导入路径

### 内部导入（crate 内部）

模块使用新结构相互引用：

```rust
use crate::consumers::consumer::Consumer;
use crate::predicates::predicate::Predicate;
use crate::transformers::transformer::Transformer;
```

### 外部导入（用户使用）

用户可以通过两种方式导入类型：

1. **从根直接导入**（推荐，向后兼容）：
   ```rust
   use prism3_function::{Consumer, Predicate, Transformer};
   ```

2. **从特定模块导入**（显式）：
   ```rust
   use prism3_function::consumers::Consumer;
   use prism3_function::predicates::Predicate;
   use prism3_function::transformers::Transformer;
   ```

两种风格都通过 `lib.rs` 中的重导出得到支持。

## 设计理念

### 为什么使用复数目录名？

使用复数名称（如 `consumers`, `predicates`）避免路径歧义：

- ✅ `src/consumers/consumer.rs` → `crate::consumers::consumer`
- ❌ `src/consumer/consumer.rs` → `crate::consumer::consumer`（令人困惑）

### 为什么有独立文件？

`comparator.rs` 和 `tester.rs` 保留在根目录，因为：

1. 它们是单文件模块，没有变体
2. 它们作为工具抽象使用
3. 保持扁平化减少不必要的嵌套

### 模块分组逻辑

模块按以下方式分组：

1. **主要功能**（consumers, predicates, transformers 等）
2. **参数数量**（单参数和双参数变体在同一模块）
3. **状态管理**（无状态、once、有状态变体在同一模块）

这种组织方式使得查找相关抽象变得容易，并有助于理解不同变体之间的关系。

## 迁移说明

如果您正在更新引用旧路径的代码：

| 旧路径 | 新路径 |
|--------|--------|
| `crate::consumer` | `crate::consumers::consumer` |
| `crate::predicate` | `crate::predicates::predicate` |
| `crate::transformer` | `crate::transformers::transformer` |
| `crate::function` | `crate::functions::function` |
| `crate::supplier` | `crate::suppliers::supplier` |
| `crate::mutator` | `crate::mutators::mutator` |

由于 `lib.rs` 中的重导出，公共 API 导入保持不变。

## 作者

胡海星 <starfish.hu@gmail.com>

