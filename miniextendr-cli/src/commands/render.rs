use anyhow::Result;

use crate::bridge::rscript_eval;
use crate::cli::RenderCmd;
use crate::project::ProjectContext;

pub fn dispatch(cmd: &RenderCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        RenderCmd::KnitrSetup => render_knitr_setup(ctx, quiet),
        RenderCmd::Rmarkdown => render_rmarkdown_setup(ctx, quiet),
        RenderCmd::Quarto => render_quarto_setup(ctx, quiet),
        RenderCmd::QuartoPre => render_quarto_pre(ctx, quiet),
        RenderCmd::Html => render_format(ctx, "html", quiet),
        RenderCmd::Pdf => render_format(ctx, "pdf", quiet),
        RenderCmd::Word => render_format(ctx, "word", quiet),
    }
}

/// Set up knitr integration — creates the knitr engine setup chunk.
fn render_knitr_setup(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let vignette_dir = ctx.root.join("vignettes");
    std::fs::create_dir_all(&vignette_dir)?;

    if !quiet {
        println!("Knitr setup: add this to your Rmd setup chunk:");
        println!();
        println!("  ```{{r setup}}");
        println!("  library({pkg})", pkg = pkg_name(ctx));
        println!("  ```");
    }
    Ok(())
}

/// Set up rmarkdown integration.
fn render_rmarkdown_setup(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let vignette_dir = ctx.root.join("vignettes");
    std::fs::create_dir_all(&vignette_dir)?;

    if !quiet {
        let pkg = pkg_name(ctx);
        println!("Rmarkdown setup: use this YAML header:");
        println!();
        println!("  output: {pkg}::miniextendr_html_document");
    }
    Ok(())
}

/// Set up Quarto integration.
fn render_quarto_setup(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    // Create _quarto.yml pre-render hook
    let quarto_yml = ctx.root.join("_quarto.yml");
    if !quarto_yml.exists() && !quiet {
        println!("Add to _quarto.yml:");
        println!();
        println!("  project:");
        println!("    pre-render: miniextendr workflow sync");
    }
    Ok(())
}

/// Run Quarto pre-render: sync project before render.
fn render_quarto_pre(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    // Equivalent to miniextendr_sync: autoconf + configure + document
    super::workflow::dispatch(&crate::cli::WorkflowCmd::Sync, ctx, quiet)
}

/// Render a document with auto-sync.
fn render_format(ctx: &ProjectContext, format: &str, quiet: bool) -> Result<()> {
    // First sync
    if !quiet {
        eprintln!("Syncing project before render...");
    }
    let _ = super::workflow::dispatch(&crate::cli::WorkflowCmd::Sync, ctx, true);

    // Then render with rmarkdown
    let root = ctx.root.to_string_lossy().replace('\\', "/");
    let expr = match format {
        "html" => format!("rmarkdown::render(\"{root}\", output_format = \"html_document\")"),
        "pdf" => format!("rmarkdown::render(\"{root}\", output_format = \"pdf_document\")"),
        "word" => format!("rmarkdown::render(\"{root}\", output_format = \"word_document\")"),
        _ => format!("rmarkdown::render(\"{root}\")"),
    };
    rscript_eval(&expr, &ctx.root, quiet)?;
    Ok(())
}

fn pkg_name(ctx: &ProjectContext) -> String {
    let desc = ctx.root.join("DESCRIPTION");
    if let Ok(content) = std::fs::read_to_string(desc) {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Package:") {
                return value.trim().to_string();
            }
        }
    }
    "mypackage".into()
}
