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

const PGJS_MODULE: &str = "github.com/sunerpy/protoc-gen-jsonschema";

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

/// The version `go.mod` pins `module` at, with the leading `v` stripped.
fn pinned_version(go_mod: &str, module: &str) -> String {
    for line in go_mod.lines() {
        let spec = line.trim().trim_start_matches("require ").trim();
        let mut fields = spec.split_whitespace();
        if fields.next() == Some(module)
            && let Some(version) = fields.next()
        {
            return version.trim_start_matches('v').to_string();
        }
    }
    panic!("go.mod has no require entry for {module}:\n{go_mod}");
}

/// The value of a `NAME = value` assignment in a Makefile.
fn makefile_variable(makefile: &str, name: &str) -> String {
    for line in makefile.lines() {
        if line.starts_with('\t') {
            continue;
        }
        if let Some(rest) = line.trim_end().strip_prefix(name)
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            return value.trim().to_string();
        }
    }
    panic!("Makefile has no `{name} = …` assignment:\n{makefile}");
}

/// A Makefile target's header line plus its whole recipe body.
fn makefile_target(makefile: &str, target: &str) -> String {
    let header = format!("{target}:");
    let mut lines = makefile.lines().skip_while(|l| !l.starts_with(&header));
    let first = lines
        .next()
        .unwrap_or_else(|| panic!("Makefile has no `{target}:` target:\n{makefile}"));

    let mut block = vec![first];
    for line in lines {
        if !line.starts_with('\t') && !line.trim().is_empty() {
            break;
        }
        block.push(line);
    }
    block.join("\n")
}

/// A Makefile target's declared prerequisites (the tokens after the `:` on its
/// header line). Parsing the header — rather than searching the file for a name
/// — is what makes a prerequisite assertion falsifiable: deleting the dependency
/// empties this list even though the `check-plugin:` target definition, its
/// `.PHONY` entry, and its `## check-plugin:` doc comment all still mention the
/// name elsewhere in the file.
fn makefile_prerequisites(makefile: &str, target: &str) -> Vec<String> {
    let block = makefile_target(makefile, target);
    let header = block.lines().next().unwrap_or_default();
    let (_, deps) = header
        .split_once(':')
        .unwrap_or_else(|| panic!("`{target}` header has no `:`: {header:?}"));
    deps.split_whitespace().map(str::to_string).collect()
}

/// The version the README's pinned `go install …@vX.Y.Z` command asks for.
fn readme_pinned_install_version(readme: &str) -> String {
    for line in readme.lines() {
        if !line.contains("protoc-gen-jsonschema/cmd") {
            continue;
        }
        if let Some((_, rest)) = line.split_once("@v") {
            return rest.trim().to_string();
        }
    }
    panic!("README has no pinned `protoc-gen-jsonschema@vX.Y.Z` install command:\n{readme}");
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

/// The single-source-of-truth invariant.
///
/// `constants::defaults::PROTOC_GEN_JSONSCHEMA_MIN_VERSION` is the only place the
/// required version may be written down. This test re-derives it from each of the
/// three rendered artifacts INDEPENDENTLY (go.mod's require pin, the Makefile's
/// `PGJS_MIN_VERSION`, the README's pinned `go install …@vX.Y.Z`) and demands all
/// four agree. Asserting each artifact against a literal `"0.2.0"` would not do
/// this: three literals can be edited apart one at a time and still each match
/// their own copy. Extracting and cross-comparing means changing ANY single site
/// — including hardcoding a template back to a literal — breaks the build.
#[test]
fn go_mcp_server_version_gate_has_exactly_one_source_of_truth() {
    let tmp = render_mcp_server();
    let go_mod = fs::read_to_string(tmp.path().join("go.mod")).expect("read go.mod");
    let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");
    let readme = fs::read_to_string(tmp.path().join("README.md")).expect("read README.md");

    let required = scaffold_gen::constants::defaults::PROTOC_GEN_JSONSCHEMA_MIN_VERSION;
    let from_go_mod = pinned_version(&go_mod, PGJS_MODULE);
    let from_makefile = makefile_variable(&makefile, "PGJS_MIN_VERSION");
    let from_readme = readme_pinned_install_version(&readme);

    assert_eq!(
        from_go_mod, from_makefile,
        "the go.mod pin (v{from_go_mod}) and the Makefile version assertion \
         ({from_makefile}) have drifted apart — both must render from \
         PROTOC_GEN_JSONSCHEMA_MIN_VERSION"
    );
    assert_eq!(
        from_makefile, from_readme,
        "the Makefile version assertion ({from_makefile}) and the README install \
         command (v{from_readme}) have drifted apart — both must render from \
         PROTOC_GEN_JSONSCHEMA_MIN_VERSION"
    );
    assert_eq!(
        from_makefile, required,
        "the rendered artifacts declare {from_makefile} but \
         PROTOC_GEN_JSONSCHEMA_MIN_VERSION is {required} — a template is carrying \
         a hardcoded literal instead of the constant"
    );

    // Lenient minijinja renders an unknown key as "", so a typo'd template
    // variable yields `v` / an empty assertion rather than an error.
    assert!(
        !required.is_empty() && required.starts_with(|c: char| c.is_ascii_digit()),
        "the required version must be a bare semver with no leading `v`: {required:?}"
    );
}

/// The assertion must actually gate `buf generate`, and must compare versions
/// NUMERICALLY and PORTABLY.
#[test]
fn go_mcp_server_makefile_blocks_generate_on_an_old_plugin() {
    let tmp = render_mcp_server();
    let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");

    let generate = makefile_target(&makefile, "generate");
    assert!(
        generate.lines().next().is_some_and(|header| header
            .split_once(':')
            .is_some_and(|(_, deps)| deps.split_whitespace().any(|d| d == "check-plugin"))),
        "`generate` must depend on `check-plugin` so the version gate runs BEFORE \
         buf generate:\n{generate}"
    );

    let check = makefile_target(&makefile, "check-plugin");
    assert!(
        check.contains("command -v protoc-gen-jsonschema"),
        "check-plugin must detect a missing plugin — buf resolves it from PATH:\n{check}"
    );
    assert!(
        check.contains("protoc-gen-jsonschema --version"),
        "check-plugin must read the installed plugin version:\n{check}"
    );
    // `exit 1; \` matches the shell-level exits only; the awk program's own
    // `exit 1` (its false branch) ends with `}` and is deliberately excluded.
    assert_eq!(
        check.matches("exit 1; \\").count(),
        3,
        "check-plugin must fail hard (never warn) in all three failure modes: \
         plugin absent, version unreadable, version too old:\n{check}"
    );
    assert!(
        !check.contains("sort -V"),
        "`sort -V` is a GNU coreutils extension and is not guaranteed on macOS \
         BSD sort; use the POSIX awk comparison:\n{check}"
    );
    assert!(
        check.contains(r#"split(have,h,".")"#) && check.contains(r#"split(min,m,".")"#),
        "the comparison must split on `.` and compare components numerically — a \
         string compare gets 0.10.0 vs 0.2.0 wrong:\n{check}"
    );

    // Every message must be actionable: found version, required version, fix.
    assert!(
        check.contains("$$have") && check.contains("$(PGJS_MIN_VERSION)"),
        "failure messages must name both the found and the required version:\n{check}"
    );
    assert!(
        check.contains("go install $(PGJS_PKG)@"),
        "failure messages must give the exact reinstall command:\n{check}"
    );
}

/// `make check` must also run the toolchain gate.
///
/// Why this matters: `make build` succeeds with a stale (or absent) plugin,
/// because `proto/gen/` ships checked-in placeholder files — nothing in a plain
/// compile reads the plugin. Only `make generate` used to notice, so a user could
/// run the project's own full self-check and still be silently one `make generate`
/// away from a wrong JSON Schema. `check` means "verify this project", so the
/// version gate belongs in it.
///
/// This asserts on `check`'s PREREQUISITE LIST, not on the presence of the string
/// `check-plugin` in the Makefile: that string necessarily also appears in the
/// `check-plugin:` target header, the `.PHONY` line, and the `## check-plugin:`
/// help comment, so a `makefile.contains("check-plugin")` assertion would keep
/// passing after the prerequisite were deleted.
#[test]
fn go_mcp_server_makefile_check_runs_the_plugin_version_gate() {
    let tmp = render_mcp_server();
    let makefile = fs::read_to_string(tmp.path().join("Makefile")).expect("read Makefile");

    let deps = makefile_prerequisites(&makefile, "check");
    assert!(
        deps.iter().any(|d| d == "check-plugin"),
        "`check` must depend on `check-plugin` so a stale protoc-gen-jsonschema \
         fails the project's own self-check — `build` alone cannot notice, the \
         checked-in proto/gen placeholders compile fine; prerequisites: {deps:?}"
    );

    // The gate must still be a hard failure here, exactly as it is for `generate`
    // — `check` inherits check-plugin's `exit 1`, so a soft warning would need
    // check-plugin itself to change.
    let gate = makefile_target(&makefile, "check-plugin");
    assert!(
        gate.contains("exit 1; \\"),
        "check-plugin must exit non-zero so `check` fails on an old plugin:\n{gate}"
    );

    // `make help` parses `## target: description` comments; the description must
    // describe what `check` now actually does.
    let doc = makefile
        .lines()
        .find(|l| l.starts_with("## check:"))
        .unwrap_or_else(|| panic!("Makefile has no `## check:` help comment:\n{makefile}"));
    assert!(
        doc.contains("plugin") || doc.contains("protoc-gen-jsonschema"),
        "the `## check:` help comment must mention the plugin/toolchain check so \
         `make help` is not misleading: {doc:?}"
    );
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
