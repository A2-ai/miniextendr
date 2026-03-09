use std::path::Path;

use anyhow::{Result, bail};

use crate::bridge::{has_program, run_command};
use crate::cli::InitCmd;
use crate::project::ProjectContext;

pub fn dispatch(cmd: &InitCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        InitCmd::Package { path } => init_package(path, quiet),
        InitCmd::Monorepo {
            path,
            package,
            crate_name,
            rpkg_name,
            local_path: _,
            miniextendr_version: _,
        } => init_monorepo(
            path,
            package.as_deref(),
            crate_name.as_deref(),
            rpkg_name.as_deref(),
            quiet,
        ),
        InitCmd::Use {
            template_type: _,
            rpkg_name: _,
            miniextendr_version: _,
            local_path: _,
        } => init_use(&ctx.root, quiet),
    }
}

/// Create a new R package with miniextendr scaffolding.
fn init_package(path: &str, quiet: bool) -> Result<()> {
    let root = Path::new(path);
    if root.exists() {
        bail!("Directory already exists: {path}");
    }

    let pkg_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mypackage");
    let crate_name = pkg_name.replace(['.', '-'], "_");

    if !quiet {
        eprintln!("Creating R package: {pkg_name}");
    }

    // Create directory structure
    std::fs::create_dir_all(root.join("R"))?;
    std::fs::create_dir_all(root.join("man"))?;
    std::fs::create_dir_all(root.join("src/rust"))?;
    std::fs::create_dir_all(root.join("tools"))?;
    std::fs::create_dir_all(root.join("inst/include"))?;

    // DESCRIPTION
    write_description(root, pkg_name)?;

    // NAMESPACE
    std::fs::write(root.join("NAMESPACE"), "")?;

    // .Rbuildignore
    write_rbuildignore(root)?;

    // .gitignore
    write_gitignore(root)?;

    // configure.ac
    write_configure_ac(root, pkg_name)?;

    // src/Makevars.in
    write_makevars_in(root)?;

    // src/stub.c
    write_stub_c(root)?;

    // src/rust/Cargo.toml
    write_cargo_toml(root, &crate_name)?;

    // src/rust/lib.rs
    write_lib_rs(root, &crate_name)?;

    // src/rust/build.rs
    write_build_rs(root)?;

    // src/rust/cargo-config.toml.in
    write_cargo_config_in(root)?;

    // bootstrap.R
    write_bootstrap_r(root)?;

    // cleanup
    write_cleanup(root)?;

    // miniextendr.yml
    write_config_yml(root)?;

    // R/{pkg}-package.R
    write_package_r(root, pkg_name)?;

    // Run autoconf if available
    if has_program("autoconf") {
        let _ = run_command("autoconf", &["-vif"], root, quiet);
    }

    if !quiet {
        eprintln!("\nPackage created at: {path}");
        eprintln!("Next steps:");
        eprintln!("  cd {path}");
        eprintln!("  miniextendr workflow build");
    }

    Ok(())
}

/// Create a Rust workspace with an embedded R package.
fn init_monorepo(
    path: &str,
    package: Option<&str>,
    _crate_name: Option<&str>,
    rpkg_name: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let root = Path::new(path);
    if root.exists() {
        bail!("Directory already exists: {path}");
    }

    let dir_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("myproject");
    let pkg_name = package.unwrap_or(dir_name);
    let rpkg_dir = rpkg_name.unwrap_or("rpkg");

    if !quiet {
        eprintln!("Creating monorepo: {dir_name}");
    }

    // Create workspace root
    std::fs::create_dir_all(root)?;

    // Workspace Cargo.toml
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            "[workspace]\nresolver = \"3\"\nmembers = []\nexclude = [\"{rpkg_dir}\", \"{rpkg_dir}/src/rust\"]\n"
        ),
    )?;

    // Create the R package inside
    let rpkg_path = root.join(rpkg_dir);
    init_package(&rpkg_path.to_string_lossy(), quiet)?;

    // Override DESCRIPTION package name if specified
    if package.is_some() {
        write_description(&rpkg_path, pkg_name)?;
    }

    if !quiet {
        eprintln!("\nMonorepo created at: {path}");
        eprintln!("  R package: {rpkg_dir}/");
        eprintln!("Next steps:");
        eprintln!("  cd {path}/{rpkg_dir}");
        eprintln!("  miniextendr workflow build");
    }

    Ok(())
}

/// Add miniextendr scaffolding to an existing project.
fn init_use(root: &Path, quiet: bool) -> Result<()> {
    let desc_path = root.join("DESCRIPTION");
    if !desc_path.exists() {
        bail!("No DESCRIPTION found. Is this an R package directory?");
    }

    // Read package name
    let pkg_name = read_pkg_name(root)?;
    let crate_name = pkg_name.replace(['.', '-'], "_");

    if !quiet {
        eprintln!("Adding miniextendr to: {pkg_name}");
    }

    // Create directories
    std::fs::create_dir_all(root.join("src/rust"))?;
    std::fs::create_dir_all(root.join("tools"))?;
    std::fs::create_dir_all(root.join("inst/include"))?;

    write_configure_ac(root, &pkg_name)?;
    write_makevars_in(root)?;
    write_stub_c(root)?;
    write_cargo_toml(root, &crate_name)?;
    write_lib_rs(root, &crate_name)?;
    write_build_rs(root)?;
    write_cargo_config_in(root)?;
    write_bootstrap_r(root)?;
    write_cleanup(root)?;
    write_config_yml(root)?;

    // Update DESCRIPTION
    update_description_for_miniextendr(root)?;

    // Run autoconf if available
    if has_program("autoconf") {
        let _ = run_command("autoconf", &["-vif"], root, quiet);
    }

    if !quiet {
        eprintln!("\nminiextendr scaffolding added.");
        eprintln!("Next: miniextendr workflow build");
    }

    Ok(())
}

// --- File generation helpers ---

fn write_description(root: &Path, pkg_name: &str) -> Result<()> {
    let content = format!(
        "Package: {pkg_name}\n\
         Title: What the Package Does (One Line, Title Case)\n\
         Version: 0.0.0.9000\n\
         Authors@R: person(\"First\", \"Last\", email = \"first.last@example.com\", role = c(\"aut\", \"cre\"))\n\
         Description: What the package does (one paragraph).\n\
         License: MIT + file LICENSE\n\
         Encoding: UTF-8\n\
         Roxygen: list(markdown = TRUE)\n\
         RoxygenNote: 7.3.2\n\
         SystemRequirements: Rust (>= 1.83), Cargo\n\
         Config/build/bootstrap: TRUE\n\
         NeedsCompilation: yes\n"
    );
    std::fs::write(root.join("DESCRIPTION"), content)?;
    Ok(())
}

fn write_rbuildignore(root: &Path) -> Result<()> {
    let content = "^.*\\.Rproj$\n\
                   ^\\.Rproj\\.user$\n\
                   ^src/rust/target$\n\
                   ^vendor$\n\
                   ^src/rust/\\.cargo$\n";
    std::fs::write(root.join(".Rbuildignore"), content)?;
    Ok(())
}

fn write_gitignore(root: &Path) -> Result<()> {
    let content = "src/rust/target/\n\
                   src/rust/.cargo/\n\
                   src/Makevars\n\
                   src/*.o\n\
                   src/*.so\n\
                   src/*.dll\n\
                   src/*.a\n\
                   src/*.lib\n\
                   src/*.def\n";
    std::fs::write(root.join(".gitignore"), content)?;
    Ok(())
}

fn write_configure_ac(root: &Path, pkg_name: &str) -> Result<()> {
    let content = format!(
        r#"AC_INIT([{pkg_name}], [0.0.0.9000])

# Detect build context: dev-monorepo, dev-detached, vendored-install, or prepare-cran
AC_MSG_CHECKING([build context])
if test "${{PREPARE_CRAN}}" = "true"; then
  BUILD_CONTEXT=prepare-cran
elif test "${{NOT_CRAN}}" = "true" -o -d "${{srcdir}}/../../miniextendr-api"; then
  BUILD_CONTEXT=dev-monorepo
elif test -f "${{srcdir}}/inst/vendor.tar.xz" -o -d "${{srcdir}}/vendor"; then
  BUILD_CONTEXT=vendored-install
else
  BUILD_CONTEXT=dev-detached
fi
AC_MSG_RESULT([$BUILD_CONTEXT])

AC_SUBST(BUILD_CONTEXT)
AC_SUBST(PACKAGE_NAME)

AC_CONFIG_FILES([src/Makevars])
AC_CONFIG_FILES([src/rust/.cargo/config.toml:src/rust/cargo-config.toml.in])
AC_OUTPUT
"#
    );
    let p = root.join("configure.ac");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_makevars_in(root: &Path) -> Result<()> {
    let content = r#"RUST_SRC = rust
CARGO_MANIFEST = $(RUST_SRC)/Cargo.toml
STATLIB = $(RUST_SRC)/target/release/lib@PACKAGE_NAME@.a

PKG_LIBS = -L$(RUST_SRC)/target/release -l@PACKAGE_NAME@ -lpthread

all: $(SHLIB)

$(SHLIB): $(STATLIB)

$(STATLIB):
	@cd $(RUST_SRC) && cargo build --lib --release --manifest-path=Cargo.toml

clean:
	rm -Rf $(RUST_SRC)/target $(SHLIB) $(OBJECTS)
"#;
    let p = root.join("src/Makevars.in");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_stub_c(root: &Path) -> Result<()> {
    let content = "// Minimal stub so R's build system produces a shared library.\n\
                   // All entry points (R_init_*) are defined in Rust via miniextendr_init!().\n";
    let p = root.join("src/stub.c");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_cargo_toml(root: &Path, crate_name: &str) -> Result<()> {
    let content = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"
publish = false

[workspace]

[lib]
path = "lib.rs"
crate-type = ["staticlib"]

[features]
default = []

[dependencies]
miniextendr-api = {{ path = "../../vendor/miniextendr-api" }}

[build-dependencies]
miniextendr-lint = {{ path = "../../vendor/miniextendr-lint" }}
"#
    );
    let p = root.join("src/rust/Cargo.toml");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_lib_rs(root: &Path, crate_name: &str) -> Result<()> {
    let content = format!(
        r#"use miniextendr_api::miniextendr;

/// Add two numbers
///
/// @param a First number
/// @param b Second number
/// @return Sum of a and b
#[miniextendr]
pub fn add(a: f64, b: f64) -> f64 {{
    a + b
}}

/// Say hello
///
/// @param name Name to greet
/// @return Greeting string
#[miniextendr]
pub fn hello(name: &str) -> String {{
    format!("Hello, {{}}!", name)
}}

miniextendr_api::miniextendr_init!({crate_name});
"#
    );
    let p = root.join("src/rust/lib.rs");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_build_rs(root: &Path) -> Result<()> {
    let content = r#"fn main() {
    println!("cargo::rerun-if-changed=lib.rs");
}
"#;
    let p = root.join("src/rust/build.rs");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_cargo_config_in(root: &Path) -> Result<()> {
    let content = r#"# Generated by configure from cargo-config.toml.in
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "../../vendor"
"#;
    let dir = root.join("src/rust");
    std::fs::create_dir_all(&dir)?;
    let p = dir.join("cargo-config.toml.in");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_bootstrap_r(root: &Path) -> Result<()> {
    let content = r#"# Bootstrap script — called by devtools/pkgbuild before compilation.
# Runs autoconf + configure to generate Makevars, etc.

local({
  # Only bootstrap if configure.ac exists and configure doesn't
  if (file.exists("configure.ac") && !file.exists("configure")) {
    if (Sys.which("autoconf") != "") {
      system("autoconf -vif")
    }
  }

  if (file.exists("configure") && !file.exists("src/Makevars")) {
    system("NOT_CRAN=true bash ./configure")
  }
})
"#;
    let p = root.join("bootstrap.R");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_cleanup(root: &Path) -> Result<()> {
    let content =
        "#!/bin/sh\nrm -rf src/rust/target src/Makevars src/rust/.cargo\n";
    let p = root.join("cleanup");
    if !p.exists() {
        std::fs::write(&p, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}

fn write_config_yml(root: &Path) -> Result<()> {
    let content = "class_system: env\nstrict: false\ncoerce: false\nfeatures: []\nrust_version: stable\nvendor: true\n";
    let p = root.join("miniextendr.yml");
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn write_package_r(root: &Path, pkg_name: &str) -> Result<()> {
    let content = format!(
        "#' @keywords internal\n\"_PACKAGE\"\n\n#' @useDynLib {pkg_name}, .registration = TRUE\nNULL\n"
    );
    let r_dir = root.join("R");
    std::fs::create_dir_all(&r_dir)?;
    let p = r_dir.join(format!("{pkg_name}-package.R"));
    if !p.exists() {
        std::fs::write(p, content)?;
    }
    Ok(())
}

fn update_description_for_miniextendr(root: &Path) -> Result<()> {
    let desc_path = root.join("DESCRIPTION");
    let content = std::fs::read_to_string(&desc_path)?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut modified = false;

    // Add SystemRequirements if missing
    if !lines.iter().any(|l| l.starts_with("SystemRequirements:")) {
        lines.push("SystemRequirements: Rust (>= 1.83), Cargo".into());
        modified = true;
    }

    // Add Config/build/bootstrap if missing
    if !lines
        .iter()
        .any(|l| l.starts_with("Config/build/bootstrap:"))
    {
        lines.push("Config/build/bootstrap: TRUE".into());
        modified = true;
    }

    // Add NeedsCompilation if missing
    if !lines.iter().any(|l| l.starts_with("NeedsCompilation:")) {
        lines.push("NeedsCompilation: yes".into());
        modified = true;
    }

    if modified {
        let mut out = lines.join("\n");
        if !out.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(&desc_path, out)?;
    }

    Ok(())
}

fn read_pkg_name(root: &Path) -> Result<String> {
    let desc = std::fs::read_to_string(root.join("DESCRIPTION"))?;
    for line in desc.lines() {
        if let Some(value) = line.strip_prefix("Package:") {
            return Ok(value.trim().to_string());
        }
    }
    bail!("Could not read Package field from DESCRIPTION");
}
