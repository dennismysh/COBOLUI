//! COBALT CLI — COBOL Application Build & Layout Toolkit
//!
//! Usage:
//!   cobalt build <file.cbl>   Parse COBOL and emit IR as JSON
//!   cobalt run <file.cbl>     Parse and launch terminal renderer
//!   cobalt check <file.cbl>   Validate COBOL for COBALT compatibility

use anyhow::{bail, Context, Result};
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
    cobalt_term::run_app(&mut renderer, &app)
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

fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Project name cannot be empty");
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        bail!(
            "Project name '{}' contains invalid characters (use alphanumeric and hyphens only)",
            name
        );
    }
    if name.starts_with('-') || name.ends_with('-') {
        bail!("Project name '{}' cannot start or end with a hyphen", name);
    }
    Ok(())
}

fn generate_cobalt_toml(name: &str) -> String {
    format!(
        r##"[project]
name = "{name}"
version = "0.1.0"
entry = "src/screens/MAIN.cbl"

[build]
target = "terminal"
out_dir = "dist"

[theme]
name = "default"

[theme.palette]
0 = "#1E293B"
1 = "#3B82F6"
2 = "#FFFFFF"
3 = "#F8FAFC"
4 = "#1A1A1A"
5 = "#2563EB"
6 = "#16A34A"
7 = "#DC2626"

[theme.typography]
heading = "Inter"
body = "Inter"
mono = "JetBrains Mono"
"##
    )
}

fn generate_main_cbl(name: &str) -> String {
    let program_id = name.to_uppercase();
    format!(
        "\
       IDENTIFICATION DIVISION.
       PROGRAM-ID. {program_id}.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  STATUS-MSG     PIC X(60) VALUE \"Welcome to {name}!\".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE \"{name}\".
           05  CONTENT.
               10  MSG-TEXT   PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.
"
    )
}

fn generate_gitignore() -> &'static str {
    "dist/\ntarget/\n"
}

fn cmd_new(name: &str) -> Result<()> {
    validate_project_name(name)?;

    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        bail!("Directory '{}' already exists", name);
    }

    let src_screens = project_dir.join("src").join("screens");
    std::fs::create_dir_all(&src_screens)
        .with_context(|| format!("Failed to create directory {}", src_screens.display()))?;

    let toml_path = project_dir.join("cobalt.toml");
    std::fs::write(&toml_path, generate_cobalt_toml(name))
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;

    let main_cbl_path = src_screens.join("MAIN.cbl");
    std::fs::write(&main_cbl_path, generate_main_cbl(name))
        .with_context(|| format!("Failed to write {}", main_cbl_path.display()))?;

    let gitignore_path = project_dir.join(".gitignore");
    std::fs::write(&gitignore_path, generate_gitignore())
        .with_context(|| format!("Failed to write {}", gitignore_path.display()))?;

    println!("Created new COBALT project '{}'", name);
    println!();
    println!("  {}/", name);
    println!("  ├── cobalt.toml");
    println!("  ├── .gitignore");
    println!("  └── src/");
    println!("      └── screens/");
    println!("          └── MAIN.cbl");
    println!();
    println!("Get started:");
    println!("  cd {}", name);
    println!("  cobalt run src/screens/MAIN.cbl");

    Ok(())
}
