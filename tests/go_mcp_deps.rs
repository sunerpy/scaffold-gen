//! Go MCP server dependency-contract regression tests.
//!
//! Bug being locked in: the generated `go.mod` used to lose
//! `github.com/sunerpy/protoc-gen-jsonschema` (and demote
//! `google.golang.org/protobuf` to `// indirect`) because the scaffold runs
//! `go mod tidy` while only import-free placeholder files exist under
//! `proto/gen/`. After the user ran `make generate`, the real
//! `proto/gen/echo.pb.go` imported both modules and `go build ./...` failed with
//! "no required module provides package
//! github.com/sunerpy/protoc-gen-jsonschema/mcp/jsonschema".
//!
//! The fix is a `//go:build tools` blank-import anchor (`tools.go`), the standard
//! Go tools.go pattern: `go mod tidy` evaluates source as if every build tag were
//! enabled, so the anchor keeps both modules in the `require` block, while the
//! build constraint keeps them out of every real build.
//!
//! Why these tests assert on the RENDERED TEMPLATE output (pre-tidy) rather
//! than on post-`go mod tidy` state: `orchestrator::generate_mcp_server_embedded`
//! runs `go mod tidy` whenever `go` is on PATH, and `go mod tidy` needs NETWORK
//! (module proxy + checksum database) and rewrites `go.mod` (adds an `// indirect`
//! block, and moves versions to whatever it can actually resolve — the template
//! pins `google.golang.org/protobuf v1.36.10`, tidy resolves `v1.36.11`). A test
//! that depended on tidy's post-state would be flaky in CI and would fail
//! offline entirely.
//!
//! So these tests drive `TemplateProcessor::process_embedded_template_directory`
//! directly — the same render entry point the orchestrator uses — and stop there.
//! No `go`, `buf`, `protoc`, or network access is involved, so `go.mod` at assert
//! time is byte-for-byte the rendered template. Whether tidy then honors the
//! build-tagged anchor is settled Go toolchain behavior (verified against the
//! real toolchain); what these tests lock in is that the scaffold still SHIPS the
//! evidence tidy needs.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use scaffold_gen::generators::core::{Parameters, TemplateProcessor};
use scaffold_gen::generators::language::go::GoParams;
use walkdir::WalkDir;

/// Modules the post-`make generate` code needs but no checked-in placeholder
/// imports. These are exactly what `go mod tidy` used to strip.
const CODEGEN_ONLY_MODULES: [&str; 2] = [
    "github.com/sunerpy/protoc-gen-jsonschema",
    "google.golang.org/protobuf",
];

/// Render `frameworks/go/mcp-server` into a temp dir with a realistic context.
fn render_mcp_server() -> tempfile::TempDir {
    let mut params =
        GoParams::from_project_name("mcp-deps-demo".to_string()).with_version("1.24".to_string());
    params.base.host = Some("0.0.0.0".to_string());
    params.base.port = Some(8080);

    let tmp = tempfile::tempdir().expect("create tempdir");
    let mut processor = TemplateProcessor::new().expect("create processor");
    processor
        .process_embedded_template_directory(
            "frameworks/go/mcp-server",
            tmp.path(),
            params.to_template_context(),
        )
        .expect("render embedded mcp-server templates");
    tmp
}

/// Module paths in `go.mod`'s direct (non-`// indirect`) requirements.
fn direct_requires(go_mod: &str) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    let mut in_block = false;

    for line in go_mod.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.contains("// indirect") {
            continue;
        }
        if trimmed == "require (" {
            in_block = true;
            continue;
        }
        if in_block && trimmed == ")" {
            in_block = false;
            continue;
        }

        // Accept both the block form and the single-line `require path v1.2.3` form.
        let spec = if in_block {
            trimmed
        } else if let Some(rest) = trimmed.strip_prefix("require ") {
            rest.trim()
        } else {
            continue;
        };

        if let Some(module) = spec.split_whitespace().next()
            && (module.contains('/') || module.contains('.'))
        {
            modules.insert(module.to_string());
        }
    }

    modules
}

/// Import paths from a Go source file: both `import "x"` and `import ( ... )`,
/// tolerating blank (`_ "x"`) and named (`alias "x"`) imports.
fn go_imports(source: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let mut in_block = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if in_block {
            if trimmed.starts_with(')') {
                in_block = false;
                continue;
            }
            if let Some(path) = quoted_path(trimmed) {
                imports.push(path);
            }
            continue;
        }
        if trimmed == "import (" {
            in_block = true;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("import ")
            && let Some(path) = quoted_path(rest.trim())
        {
            imports.push(path);
        }
    }

    imports
}

fn quoted_path(fragment: &str) -> Option<String> {
    let start = fragment.find('"')?;
    let rest = &fragment[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Third-party imports (first path segment looks like a domain) that do not
/// belong to the generated module itself.
fn third_party_imports(root: &Path, module_name: &str) -> BTreeSet<String> {
    let mut imports = BTreeSet::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "go"))
    {
        let source = fs::read_to_string(entry.path()).expect("read generated Go file");
        for import in go_imports(&source) {
            let is_domain = import
                .split('/')
                .next()
                .is_some_and(|host| host.contains('.'));
            if is_domain && !import.starts_with(module_name) {
                imports.insert(import);
            }
        }
    }

    imports
}

#[test]
fn go_mcp_server_ships_a_build_tagged_dependency_anchor() {
    let tmp = render_mcp_server();
    let anchor_path = tmp.path().join("tools.go");

    assert!(
        anchor_path.is_file(),
        "tools.go must be generated so `go mod tidy` keeps the codegen-only \
         modules in go.mod; files: {:?}",
        WalkDir::new(tmp.path())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>()
    );

    let anchor = fs::read_to_string(&anchor_path).expect("read tools.go");

    // The build constraint must be the first line: it keeps the anchor out of
    // every real build while `go mod tidy` still sees the imports.
    assert_eq!(
        anchor.lines().next(),
        Some("//go:build tools"),
        "tools.go must start with the `//go:build tools` constraint:\n{anchor}"
    );
    assert!(
        !anchor.contains("//go:build ignore"),
        "the `ignore` tag is the one constraint `go mod tidy` skips; it would \
         defeat the anchor:\n{anchor}"
    );

    // Every codegen-only module must be blank-imported so tidy retains it.
    let imports = go_imports(&anchor);
    for module in CODEGEN_ONLY_MODULES {
        assert!(
            imports.iter().any(|imp| imp.starts_with(module)),
            "tools.go must blank-import a package from {module}; imports: {imports:?}"
        );
    }
    for import in &imports {
        assert!(
            anchor.contains(&format!("_ \"{import}\"")),
            "anchor import {import} must be a blank import (`_ \"…\"`) so it \
             contributes no symbols:\n{anchor}"
        );
    }
}

#[test]
fn go_mcp_server_go_mod_requires_codegen_only_modules() {
    let tmp = render_mcp_server();
    let go_mod = fs::read_to_string(tmp.path().join("go.mod")).expect("read go.mod");
    let requires = direct_requires(&go_mod);

    for module in CODEGEN_ONLY_MODULES {
        assert!(
            requires.contains(module),
            "go.mod must declare {module} as a direct requirement — the code \
             emitted by `make generate` imports it:\n{go_mod}"
        );
    }
}

#[test]
fn go_mcp_server_every_third_party_import_has_a_direct_require() {
    let tmp = render_mcp_server();
    let module_name = "github.com/example/mcp-deps-demo";
    let go_mod = fs::read_to_string(tmp.path().join("go.mod")).expect("read go.mod");
    let requires = direct_requires(&go_mod);
    let imports = third_party_imports(tmp.path(), module_name);

    assert!(
        !imports.is_empty(),
        "expected the scaffold to import third-party packages; found none"
    );

    for import in &imports {
        let covered = requires
            .iter()
            .any(|module| import == module || import.starts_with(&format!("{module}/")));
        assert!(
            covered,
            "import {import} has no matching direct requirement in go.mod; \
             requires: {requires:?}"
        );
    }
}
