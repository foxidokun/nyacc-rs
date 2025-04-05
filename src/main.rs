mod ast;
mod codegen;
mod utils;
mod visitor;

use codegen::compile;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar); // synthesized by LALRPOP

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    /// Path of input NyaC program
    input: PathBuf,

    #[arg(short, long, default_value = "./a.out")]
    /// Path for generated ELF file
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let input_content = std::fs::read_to_string(&cli.input);
    if let Err(e) = input_content {
        panic!(
            "Failed to read input file {} with error {}",
            cli.input.display(),
            e
        );
    }
    let input_content = input_content.unwrap();

    let ast = crate::grammar::ProgramParser::new().parse(&input_content);
    if let Err(e) = ast {
        panic!("Failed to parse into AST with error {}", e);
    }
    let ast = ast.unwrap();

    let res = compile(&ast);
    if let Err(e) = res {
        panic!("Failed to compile with error {}", e);
    }
}
