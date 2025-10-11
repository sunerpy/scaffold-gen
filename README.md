# Scaffold-Gen

一个现代化、可扩展的脚手架生成器，支持多种框架和编程语言的项目模板创建。

## 特性

- **现代化架构**: 基于trait的清洁、模块化设计
- **三层生成器架构**: 项目级、语言级、框架级分层生成
- **统一生成器接口**: 所有框架生成器的一致API
- **后处理管道**: 可扩展的自定义项目设置钩子
- **交互式CLI**: 用户友好的项目配置提示
- **环境验证**: 自动检查所需工具和依赖

## 系统要求

- **Rust** (1.70 或更高版本)
- **Go** (1.19 或更高版本) - 用于Go项目生成

## 安装

### 从源码构建

```bash
git clone <repository-url>
cd scaffold-gen
cargo build --release
```

### 快速构建并复制二进制文件

用于开发和测试，可以构建并自动将二进制文件复制到项目根目录：

```bash
# Windows
$env:SCAFFOLD_COPY_BINARY="1"; cargo build

# Linux/macOS  
SCAFFOLD_COPY_BINARY=1 cargo build
```

这将在项目根目录创建一个 `scafgen` (或Windows上的 `scafgen.exe`) 二进制文件，便于测试。

## 使用方法

### 交互模式 (推荐)

```bash
scafgen new my-project
```

CLI将引导您完成：
- 语言选择 (Go等)
- 框架选择 (Gin、Go-Zero等)
- 项目配置 (主机、端口、功能)
- 许可证选择

### 直接指定框架

```bash
# 创建Gin项目
scafgen new my-gin-app --framework gin

# 创建Go-Zero项目  
scafgen new my-gozero-app --framework go-zero
```

## 架构设计

### 三层生成器架构

Scaffold-Gen采用分层的生成器架构，提供清晰的职责分离：

#### 1. 项目级生成器 (ProjectGenerator)
负责通用项目文件的生成：
- LICENSE文件生成
- Git仓库初始化
- Pre-commit hooks安装
- README文件生成

#### 2. 语言级生成器 (LanguageGenerator)
处理特定编程语言的设置：
- **GoGenerator**: Go模块初始化、依赖管理
- 环境配置和验证
- 语言特定的配置文件

#### 3. 框架级生成器 (FrameworkGenerator)
生成框架特定的代码结构：
- **GinGenerator**: Gin web框架项目结构
- **GoZeroGenerator**: Go-Zero微服务框架结构
- 框架特定的中间件、路由、配置

### 核心组件

- **`Generator` Trait**: 所有生成器的基础接口
- **`Parameters` Trait**: 类型安全的参数管理
- **`TemplateProcessor`**: 模板处理和渲染引擎
- **`GeneratorOrchestrator`**: 协调三层生成器的执行

### 生成流程

1. **参数验证**: 验证用户输入和环境要求
2. **项目级生成**: 创建基础项目文件和结构
3. **语言级生成**: 设置语言特定的环境和配置
4. **框架级生成**: 生成框架特定的代码和结构
5. **后处理**: 执行依赖安装、Git初始化等

## 快速开始

1. **构建项目**:
   ```bash
   # 带自动二进制复制的测试构建
   $env:SCAFFOLD_COPY_BINARY="1"; cargo build
   ```

2. **创建新项目**:
   ```bash
   ./scafgen new my-awesome-project
   ```

3. **按照交互提示** 配置您的项目

4. **进入项目目录** 开始编码！

## 配置

### 模板系统

生成器使用分层模板系统：

```
templates/
├── frameworks/          # 框架特定模板
│   ├── go/
│   │   ├── gin/        # Gin框架模板
│   │   └── go-zero/    # Go-Zero框架模板
├── languages/          # 语言特定模板  
│   └── go/             # Go语言模板
└── licenses/           # 许可证模板
    ├── MIT.tmpl
    ├── Apache-2.0.tmpl
    └── GPL-3.0.tmpl
```

### 模板变量

#### 通用变量
- `{{project_name}}` - 项目名称
- `{{author}}` - 项目作者
- `{{license}}` - 许可证类型
- `{{year}}` - 当前年份

#### Go语言变量
- `{{module_name}}` - Go模块名称
- `{{go_version}}` - Go版本

#### 框架特定变量
- `{{host}}` - 服务器主机 (默认: localhost)
- `{{port}}` - HTTP端口 (默认: 8080)
- `{{grpc_port}}` - gRPC端口 (Go-Zero专用, 默认: 9090)
- `{{enable_swagger}}` - 是否启用Swagger文档
- `{{enable_database}}` - 是否启用数据库支持
- `{{database_type}}` - 数据库类型 (mysql, postgres, sqlite)

## 扩展系统

### 添加新的框架生成器

1. **实现Generator trait**:
   ```rust
   pub struct MyFrameworkGenerator {
       template_processor: TemplateProcessor,
   }
   
   impl Generator for MyFrameworkGenerator {
       type Params = MyFrameworkParams;
       
       fn name(&self) -> &'static str {
           "MyFramework"
       }
       
       fn get_template_path(&self) -> &'static str {
           "frameworks/my-language/my-framework"
       }
   }
   ```

2. **实现FrameworkGenerator trait**:
   ```rust
   impl FrameworkGenerator for MyFrameworkGenerator {
       fn framework(&self) -> &'static str {
           "my-framework"
       }
       
       fn language(&self) -> &'static str {
           "my-language"
       }
       
       fn generate_basic_structure(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
           // 实现框架特定的结构生成
       }
   }
   ```

3. **添加框架模板** 到 `templates/frameworks/my-language/my-framework/`

4. **在编排器中注册** 新的生成器

### 添加新的语言支持

1. **创建语言参数结构**:
   ```rust
   #[derive(Debug, Clone)]
   pub struct MyLanguageParams {
       pub project_name: String,
       pub version: String,
       // 其他语言特定参数
   }
   ```

2. **实现LanguageGenerator trait**:
   ```rust
   impl LanguageGenerator for MyLanguageGenerator {
       fn language(&self) -> &'static str {
           "my-language"
       }
       
       fn setup_environment(&mut self, params: &Self::Params, output_path: &Path) -> Result<()> {
           // 实现语言环境设置
       }
   }
   ```

## 开发

### 项目结构

```
src/
├── commands/           # CLI命令实现
├── generators/         # 生成器模块
│   ├── core/          # 核心生成器traits和工具
│   ├── project/       # 项目级生成器
│   ├── language/      # 语言级生成器
│   │   └── go/        # Go语言生成器
│   ├── framework/     # 框架级生成器
│   │   ├── gin/       # Gin框架生成器
│   │   └── go_zero/   # Go-Zero框架生成器
│   └── orchestrator.rs # 生成器编排器
├── scaffold.rs        # 核心脚手架系统
├── template_engine.rs # 模板处理引擎
└── utils/             # 工具模块
```

### 核心组件

- **三层生成器架构**: 项目、语言、框架分层处理
- **`GeneratorOrchestrator`**: 协调各层生成器的执行
- **模板处理系统**: 基于嵌入式模板的文件生成
- **参数管理**: 类型安全的参数传递和验证

### 构建

```bash
# 调试构建
cargo build

# 发布构建  
cargo build --release

# 带自动二进制复制的测试构建
$env:SCAFFOLD_COPY_BINARY="1"; cargo build
```

### 测试

```bash
cargo test
```

### 代码质量

```bash
# 运行clippy检查
cargo clippy --all-targets --all-features

# 格式化代码
cargo fmt
```

## 贡献

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 为新功能添加测试
5. 提交Pull Request

## 许可证

本项目基于MIT许可证 - 详见LICENSE文件。
