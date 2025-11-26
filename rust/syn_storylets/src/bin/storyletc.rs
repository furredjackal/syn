// Storylet Compiler CLI: compiles JSON storylets into an indexed binary library.

use clap::Parser;
use std::path::PathBuf;
use syn_storylets::compiler::StoryletCompiler;
use syn_storylets::validation::default_storylet_validator;

#[derive(Parser, Debug)]
#[command(
    name = "storyletc",
    about = "Compiles JSON storylets into a binary library for SYN",
    long_about = "Recursively loads all .json storylet definitions from INPUT directory, \
                 validates them, builds indexed structures, and writes a compiled binary to OUTPUT"
)]
struct Args {
    /// Input directory containing JSON storylet files
    #[arg(long, short)]
    input: PathBuf,

    /// Output path for compiled binary library
    #[arg(long, short)]
    output: PathBuf,

    /// Print detailed error information
    #[arg(long, default_value_t = false)]
    pretty_errors: bool,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("SYN Storylet Compiler");
        println!("Input directory:  {}", args.input.display());
        println!("Output library:   {}", args.output.display());
        println!();
    }

    // Create compiler with default validator
    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);

    // Compile storylets
    if args.verbose {
        println!("Scanning for JSON storylets...");
    }

    match compiler.compile_from_dir(&args.input) {
        Ok(library) => {
            if args.verbose {
                println!("✓ Compiled {} storylets", library.total_count);
                println!("  - Tags: {} unique tags", library.tag_index.len());
                println!("  - Life stages: {} stages present", library.life_stage_index.len());
                println!("  - Domains: {} domains present", library.domain_index.len());
                println!();
                println!("Writing binary library...");
            }

            // Write library to file
            match library.write_to_file(&args.output) {
                Ok(()) => {
                    if args.verbose {
                        let file_size = std::fs::metadata(&args.output)
                            .map(|m| m.len())
                            .unwrap_or(0);
                        println!("✓ Successfully wrote {} bytes to {}", file_size, args.output.display());
                    } else {
                        println!("✓ Compilation successful: {}", args.output.display());
                    }
                }
                Err(err) => {
                    eprintln!("✗ Failed to write library: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(errors) => {
            eprintln!("✗ Compilation failed with {} error(s):\n", errors.len());

            if args.pretty_errors {
                for (i, err) in errors.iter().enumerate() {
                    eprintln!("[Error {}]", i + 1);
                    eprintln!("{}\n", err);
                }
            } else {
                for err in &errors {
                    eprintln!("- {}", err);
                }
            }

            std::process::exit(1);
        }
    }
}
