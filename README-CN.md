# Scaffold-Gen - 现代化项目脚手架生成器

一个强大的CLI工具，用于生成内置现代开发实践的项目脚手架。

## 🚀 特性

- **现代化架构**: 基于统一的生成器接口设计
- **多框架支持**: 支持生成Go-Zero和Gin框架项目
- **统一生成器接口**: 通过ProjectGenerator trait实现一致的API
- **后处理管道**: 自动化项目设置和依赖安装
- **交互式CLI**: 友好的用户交互体验
- **环境验证**: 自动检查开发环境依赖

## 📦 安装

### 前置要求

- Rust (1.70 或更高版本)
- Cargo 包管理器
- Go (1.21 或更高版本)

### 从源码构建

```bash
git clone <repository-url>
cd devcli
cargo build --release
```

可执行文件将位于 `target/release/scafgen`。

## 🛠️ 使用方法

### 基本命令

```bash
scafgen new <项目名称>
```

### 交互式项目生成

Scaffold-Gen 提供友好的交互式界面：

```bash
# 启动交互式项目生成
scafgen new my-project

# 工具将引导您完成以下选择：
# 1. 选择框架 (gin/go-zero)
# 2. 配置项目参数
# 3. 选择许可证类型
# 4. 确认生成设置
```

### 架构设计

#### 核心组件

- **ParameterScope**: 类型安全的参数管理系统
- **Scaffold**: 核心脚手架引擎，处理模板和目录结构
- **ProjectGenerator Trait**: 统一的生成器接口
- **后处理管道**: 自动化项目设置任务

#### 后处理管道

生成项目后，系统会自动执行：

1. **Go模块初始化**: `go mod init`
2. **依赖整理**: `go mod tidy`  
3. **Git仓库初始化**: `git init`
4. **环境验证**: 检查必要的开发工具

### 快速开始

1. **生成新项目**:
   ```bash
   scafgen new my-awesome-api
   ```

2. **进入项目目录**:
   ```bash
   cd my-awesome-api
   ```

3. **运行项目**:
   ```bash
   # Gin项目
   go run main.go
   
   # go-zero项目
   go run api/main.go
   ```

### 示例

#### 创建Gin项目

```bash
scafgen new my-gin-app
# 在交互界面中选择 "gin" 框架
```

#### 创建go-zero项目

```bash
scafgen new my-microservice  
# 在交互界面中选择 "go-zero" 框架
```

## 🏗️ 支持的框架

### Go-Zero

一个云原生的Go微服务框架，具有以下特性：

- **API网关**: 带自动路由的RESTful API
- **gRPC支持**: 内置gRPC服务器配置
- **服务上下文**: 依赖注入和配置管理
- **健康检查**: 开箱即用的健康检查端点
- **错误处理**: 带自定义错误类型的结构化错误处理
- **Docker支持**: 多阶段Dockerfile和docker-compose配置

**生成的结构:**

```
my-project/
├── api/
│   ├── internal/
│   │   ├── config/
│   │   ├── handler/
│   │   ├── logic/
│   │   ├── svc/
│   │   └── types/
│   └── main.go
├── rpc/
├── etc/
│   ├── api.yaml
│   └── rpc.yaml
├── Dockerfile
├── docker-compose.yml
└── README.md
```

### Gin

一个高性能的Go HTTP Web框架：

- **快速路由**: 基于基数树的路由，内存占用小
- **中间件支持**: 内置和自定义中间件集成
- **JSON验证**: 请求/响应验证和绑定
- **错误管理**: 集中式错误处理
- **日志记录**: 带可配置级别的结构化日志
- **健康端点**: 内置健康检查路由

**生成的结构:**

```
my-gin-app/
├── cmd/
│   └── main.go
├── internal/
│   ├── config/
│   ├── handler/
│   ├── middleware/
│   ├── model/
│   ├── response/
│   └── service/
├── config.yaml
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## 🔧 Pre-commit集成

使用 `--precommit` 标志时，DevCLI会自动设置全面的pre-commit配置：

### 包含的钩子

#### 通用钩子 (pre-commit-hooks)

- **end-of-file-fixer**: 确保文件以换行符结尾
- **check-yaml**: 验证YAML语法
- **check-added-large-files**: 防止提交大文件
- **check-merge-conflict**: 检测合并冲突标记
- **check-case-conflict**: 检查大小写冲突

#### Go特定钩子 (TekWizely/pre-commit-golang)

- **go-fmt**: 使用 `gofmt` 格式化Go代码
- **go-vet-mod**: 运行带模块支持的 `go vet`
- **go-mod-tidy**: 清理 `go.mod` 和 `go.sum`
- **go-test-mod**: 运行带模块支持的Go测试
- **golangci-lint-mod**: 运行带模块支持的golangci-lint

#### 提交信息验证 (commitizen)

- **commitizen**: 强制使用约定式提交信息格式

### 设置Pre-commit

使用 `--precommit` 生成项目后，安装并设置pre-commit：

```bash
cd your-project
pip install pre-commit
pre-commit install
pre-commit install --hook-type commit-msg
```

## 🏃‍♂️ 快速开始

1. **生成新项目:**

   ```bash
   devcli new my-awesome-api --precommit
   ```
2. **进入项目目录:**

   ```bash
   cd my-awesome-api
   ```
3. **安装依赖 (对于Go-Zero):**

   ```bash
   go mod tidy
   ```
4. **运行应用:**

   ```bash
   go run main.go
   ```
5. **测试健康检查端点:**

   ```bash
   curl http://localhost:8888/health
   ```

## 🔧 配置

### 模板系统

模板位于 `templates/` 目录中，使用Handlebars语法：

```
templates/
├── gin/
│   ├── main.go.tmpl
│   ├── config.yaml.tmpl
│   ├── internal/
│   │   ├── handler/
│   │   ├── middleware/
│   │   └── service/
│   └── ...
├── go-zero/
│   ├── api/
│   │   ├── main.go.tmpl
│   │   ├── internal/
│   │   └── ...
│   ├── etc/
│   │   ├── api.yaml.tmpl
│   │   └── rpc.yaml.tmpl
│   └── ...
└── licenses/
    ├── MIT.tmpl
    ├── Apache-2.0.tmpl
    └── ...
```

### 可用的模板变量

`ParameterScope` 系统为模板提供以下变量：

#### 通用变量
- `{{project_name}}`: 项目名称
- `{{module_name}}`: Go模块名称（通常与project_name相同）
- `{{framework}}`: 选择的框架（"gin" 或 "go-zero"）
- `{{go_version}}`: Go版本（默认: "1.21"）

#### 框架特定变量

**Gin项目:**
- `{{use_gin}}`: Gin框架的布尔标志

**go-zero项目:**
- `{{use_go_zero}}`: go-zero框架的布尔标志
- `{{api_port}}`: API服务器端口（默认: 8888）
- `{{rpc_port}}`: RPC服务器端口（默认: 9999）

#### 许可证变量
- `{{year}}`: 当前年份
- `{{author}}`: 来自git配置的作者姓名

### 扩展系统

#### 添加新的生成器

1. **创建生成器结构体:**

```rust
pub struct MyFrameworkGenerator;

impl ProjectGenerator for MyFrameworkGenerator {
    fn name(&self) -> &'static str {
        "MyFramework"
    }
    
    fn template_path(&self) -> &'static str {
        "templates/my-framework"
    }
    
    fn prepare_parameters(&self, project_name: &str) -> ParameterScope {
        let mut params = ParameterScope::new();
        params
            .add("project_name", project_name)
            .add("framework", "my-framework")
            .add("custom_param", "value");
        params
    }
    
    fn print_next_steps(&self, project_name: &str) {
        println!("🔧 下一步:");
        println!("   cd {}", project_name);
        println!("   my-framework run");
    }
}
```

2. **在 `templates/my-framework/` 中创建模板**

3. **在命令处理器中注册生成器**

#### 自定义后处理器

添加自定义后处理步骤：

```rust
PostProcessor::Command {
    command: "npm".to_string(),
    args: vec!["install".to_string()],
    description: "安装Node.js依赖".to_string(),
}
```

## 🐳 Docker支持

所有生成的项目都包含Docker支持：

### Dockerfile特性

- 多阶段构建以优化镜像大小
- 非root用户以提高安全性
- 健康检查集成
- 正确的信号处理

### Docker Compose

- 服务编排
- 环境变量管理
- 开发用卷挂载
- 网络配置

### 使用方法

```bash
# 使用Docker Compose构建并运行
docker-compose up --build

# 或手动构建
docker build -t my-app .
docker run -p 8888:8888 my-app
```

## 🧪 开发

### 项目结构

```
devcli/
├── src/
│   ├── commands/          # CLI命令实现
│   ├── generators/        # 项目生成器
│   ├── utils/            # 工具函数
│   ├── template_engine.rs # 模板处理
│   ├── lib.rs            # 库导出
│   └── main.rs           # CLI入口点
├── templates/            # 项目模板
└── Cargo.toml           # Rust依赖
```

### 核心组件

#### 1. ProjectGenerator Trait

所有项目生成器的统一接口：

```rust
pub trait ProjectGenerator {
    fn name(&self) -> &'static str;
    fn template_path(&self) -> &'static str;
    fn prepare_parameters(&self, project_name: &str) -> ParameterScope;
    fn post_processors(&self) -> Vec<PostProcessor>;
    fn generate(&self, project_name: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn print_success_message(&self, project_name: &str);
    fn print_next_steps(&self, project_name: &str);
}
```

#### 2. Scaffold系统

处理模板和应用转换的核心脚手架引擎：

- **模板处理**: 基于Handlebars的模板渲染
- **目录结构**: 递归目录创建
- **文件生成**: 模板到文件的转换
- **后处理**: 自动化设置任务

#### 3. ParameterScope

类型安全的参数管理系统：

```rust
let mut params = ParameterScope::new();
params
    .add("project_name", "my-app")
    .add("framework", "gin")
    .add("go_version", "1.21");
```

### 构建和测试

#### 构建项目

```bash
# 调试构建
cargo build

# 发布构建
cargo build --release

# 检查编译错误
cargo check
```

#### 运行测试

```bash
# 运行所有测试
cargo test

# 带输出运行测试
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

#### 开发工作流

1. **修改源代码**
2. **运行 cargo check** 验证编译
3. **本地测试** 使用 `cargo run -- new my-test-project`
4. **运行测试** 确保功能正常
5. **构建发布版本** 用于分发

### 贡献

1. Fork 仓库
2. 创建功能分支
3. 进行修改
4. 为新功能添加测试
5. 确保所有测试通过
6. 提交 pull request

### 许可证

本项目采用 MIT 许可证 - 详见 LICENSE 文件。

### 代码质量

项目使用pre-commit钩子确保代码质量：

```bash
pre-commit install
pre-commit run --all-files
```

## 📝 贡献

1. Fork仓库
2. 创建功能分支
3. 进行更改
4. 如适用，添加测试
5. 运行pre-commit钩子
6. 提交pull request

## 📄 许可证

本项目采用MIT许可证 - 详见LICENSE文件。

## 🤝 支持

- 为错误报告或功能请求创建issue
- 在创建新issue之前检查现有issue
- 提供详细信息以便更快解决问题

## 🔄 更新日志

### v0.1.0

- 初始版本
- Go-Zero和Gin框架支持
- Pre-commit集成
- Docker支持
- 基于模板的生成
