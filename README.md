# Scaffold-Gen

A modern, extensible scaffolding generator for creating project templates with support for multiple frameworks and languages.

## Features

- **Modern Architecture**: Clean, modular design with trait-based generators
- **Unified Generator Interface**: Consistent API across all framework generators  
- **Post-Processing Pipeline**: Extensible hooks for custom project setup
- **Interactive CLI**: User-friendly prompts for project configuration
- **Environment Validation**: Automatic checks for required tools and dependencies

## Prerequisites

- **Rust** (1.70 or later)
- **Go** (1.19 or later) - for Go-based projects

## Installation

### From Source

```bash
git clone <repository-url>
cd scaffold-gen
cargo build --release
```

### Quick Build with Binary Copy

For development and testing, you can build and automatically copy the binary to the project root:

```bash
# Windows
$env:SCAFFOLD_COPY_BINARY="1"; cargo build

# Linux/macOS  
SCAFFOLD_COPY_BINARY=1 cargo build

# Or use the provided scripts
# Windows
powershell -ExecutionPolicy Bypass -File build_with_copy.ps1

# Linux/macOS
./build_with_copy.sh
```

This will create a `scafgen` (or `scafgen.exe` on Windows) binary in the project root for easy testing.

## Usage

### Interactive Mode (Recommended)

```bash
scafgen new my-project
```

The CLI will guide you through:
- Language selection (Go, etc.)
- Framework selection (Gin, Go-Zero, etc.)
- Project configuration (host, port, features)
- License selection

### Direct Framework Selection

```bash
# Create a Gin project
scafgen new my-gin-app --framework gin

# Create a Go-Zero project  
scafgen new my-gozero-app --framework go-zero
```

## Architecture

### Core Components

- **`ParameterScope`**: Manages template variables and project configuration
- **`Scaffold`**: Orchestrates the project generation process
- **`ProjectGenerator` Trait**: Unified interface for all framework generators
- **Post-Processing Pipeline**: Handles setup tasks after template generation

### Post-Processing Pipeline

The system supports extensible post-processing hooks:

1. **Pre-commit setup**: Configures Git hooks and linting
2. **Dependency installation**: Runs `go mod tidy`, `npm install`, etc.
3. **Initial Git setup**: Creates repository and initial commit
4. **Custom hooks**: Framework-specific setup tasks

## Quick Start

1. **Build the project**:
   ```bash
   # With automatic binary copy for testing
   $env:SCAFFOLD_COPY_BINARY="1"; cargo build
   ```

2. **Create a new project**:
   ```bash
   ./scafgen new my-awesome-project
   ```

3. **Follow the interactive prompts** to configure your project

4. **Navigate to your project** and start coding!

## Configuration

### Template System

The generator uses a hierarchical template system:

```
templates/
├── frameworks/          # Framework-specific templates
│   ├── gin/            # Gin framework templates
│   └── go-zero/        # Go-Zero framework templates
├── languages/          # Language-specific templates  
│   └── go/             # Go language templates
└── licenses/           # License templates
```

### Template Variables

#### Common Variables
- `{{project_name}}` - Project name
- `{{author}}` - Project author
- `{{license}}` - License type
- `{{year}}` - Current year

#### Framework-Specific Variables
- `{{host}}` - Server host (default: localhost)
- `{{port}}` - HTTP port (default: 8080)
- `{{grpc_port}}` - gRPC port (Go-Zero only, default: 9090)

#### License Variables
- `{{license_text}}` - Full license text
- `{{license_header}}` - License header for source files

### Extending the System

#### Adding New Generators

1. **Create a new generator** implementing the `ProjectGenerator` trait:
   ```rust
   pub struct MyFrameworkGenerator;
   
   impl ProjectGenerator for MyFrameworkGenerator {
       fn generate(&self, scope: &ParameterScope, project_path: &Path) -> Result<()> {
           // Implementation
       }
   }
   ```

2. **Add framework-specific templates** in `templates/frameworks/my-framework/`

3. **Register the generator** in the CLI command handler

#### Adding Custom Post-Processors

Create custom post-processing hooks by implementing processing logic in the generator's `generate` method or by extending the `Scaffold` system.

## Development

### Project Structure

```
src/
├── commands/           # CLI command implementations
├── generators/         # Framework generators
│   ├── go/            # Go language generators
│   │   ├── gin.rs     # Gin framework generator
│   │   └── go_zero.rs # Go-Zero framework generator
│   └── traits.rs      # Generator trait definitions
├── scaffold.rs        # Core scaffolding system
├── template_engine.rs # Template processing engine
└── utils/             # Utility modules
```

### Core Components

- **`ProjectGenerator` Trait**: Defines the interface for all framework generators
- **Scaffold System**: Manages the overall project generation workflow
- **`ParameterScope`**: Handles template variable management and substitution

### Building

```bash
# Debug build
cargo build

# Release build  
cargo build --release

# With automatic binary copy for testing
$env:SCAFFOLD_COPY_BINARY="1"; cargo build
```

### Testing

```bash
cargo test
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
