//! COBALT CLI — COBOL Application Build & Layout Toolkit
//!
//! Usage:
//!   cobalt build <file.cbl>   Parse COBOL and emit IR as JSON
//!   cobalt run <file.cbl>     Parse and launch terminal renderer
//!   cobalt check <file.cbl>   Validate COBOL for COBALT compatibility

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cobalt", version, about = "COBALT — COBOL Application Build & Layout Toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse COBOL source and emit the IR as JSON
    Build {
        /// Path to the COBOL source file
        file: PathBuf,
    },
    /// Parse COBOL source and launch the terminal renderer
    Run {
        /// Path to the COBOL source file
        file: PathBuf,
    },
    /// Validate a COBOL source file for COBALT compatibility
    Check {
        /// Path to the COBOL source file
        file: PathBuf,
    },
    /// Scaffold a new COBALT project
    New {
        /// Project name
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file } => cmd_build(&file),
        Commands::Run { file } => cmd_run(&file),
        Commands::Check { file } => cmd_check(&file),
        Commands::New { name } => cmd_new(&name),
    }
}

fn read_source(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))
}

fn cmd_build(file: &PathBuf) -> Result<()> {
    let source = read_source(file)?;
    let app = cobalt_parser::parse(&source)
        .with_context(|| format!("Failed to parse {}", file.display()))?;

    let json = serde_json::to_string_pretty(&app)?;
    println!("{}", json);
    Ok(())
}

fn cmd_run(file: &PathBuf) -> Result<()> {
    let source = read_source(file)?;
    let app = cobalt_parser::parse(&source)
        .with_context(|| format!("Failed to parse {}", file.display()))?;

    let mut renderer = cobalt_term::TermRenderer::new();
    cobalt_render::run_app(&mut renderer, &app)
        .with_context(|| "Runtime error")?;

    Ok(())
}

fn cmd_check(file: &PathBuf) -> Result<()> {
    let source = read_source(file)?;
    match cobalt_parser::parse(&source) {
        Ok(app) => {
            println!("OK: {} is valid COBALT", file.display());
            println!(
                "  {} screen(s), {} state field(s), {} handler(s)",
                app.screens.len(),
                app.state.len(),
                app.handlers.len()
            );
            for screen in &app.screens {
                println!("  Screen: {}", screen.name);
                print_tree(&screen.root, 2);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_tree(node: &cobalt_ir::Node, indent: usize) {
    let pad = " ".repeat(indent);
    match node {
        cobalt_ir::Node::Container {
            name, children, ..
        } => {
            println!("{}[Container] {}", pad, name);
            for child in children {
                print_tree(child, indent + 2);
            }
        }
        cobalt_ir::Node::Text {
            name, pic, binding, ..
        } => {
            println!(
                "{}[Text] {} PIC X({}){}",
                pad,
                name,
                pic.width,
                binding.as_ref().map_or(String::new(), |b| format!(" USING {}", b))
            );
        }
        cobalt_ir::Node::Numeric {
            name, pic, binding, ..
        } => {
            println!(
                "{}[Numeric] {} PIC 9({}){}",
                pad,
                name,
                pic.width,
                binding.as_ref().map_or(String::new(), |b| format!(" USING {}", b))
            );
        }
        cobalt_ir::Node::Button {
            name, label, action, ..
        } => {
            println!(
                "{}[Button] {} label=\"{}\"{}",
                pad,
                name,
                label,
                action.as_ref().map_or(String::new(), |a| format!(" -> {}", a))
            );
        }
    }
}

fn cmd_new(name: &str) -> Result<()> {
    println!("Scaffolding new COBALT project: {}", name);
    println!("(Not yet implemented — coming in a future release)");
    Ok(())
}
