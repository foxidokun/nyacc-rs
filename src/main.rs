mod ast;
mod codegen;
mod utils;
mod visitor;

use ast::debug::print_ast;
use codegen::{ir_target, jit_target};
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar); // synthesized by LALRPOP

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    /// Compilation target
    target: CompileTarget,

    #[arg(short, long, value_name = "FILE")]
    /// Path of input NyaC program
    input: PathBuf,
}

#[derive(Subcommand)]
enum CompileTarget {
    /// Emit generated AST tree
    Ast {
        #[arg(short, long, default_value = "./out.ast")]
        output: PathBuf,
    },
    /// Emit generated llvm IR
    Ir {
        #[arg(short, long, default_value = "./out.ll")]
        output: PathBuf,
    },
    /// Compile & execute via LLVM JIT
    Jit {},
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

    match cli.target {
        CompileTarget::Ast { output } => {
            let file = std::fs::File::create(output);
            if let Err(e) = file {
                panic!("Failed to open AST dump file with err {}", e);
            }
            let mut file = file.unwrap();

            if let Err(e) = print_ast(&mut file, &ast) {
                panic!("Failed to write AST with error {}", e);
            }
        }
        CompileTarget::Ir { output } => {
            let res = ir_target(&ast, &output);
            if let Err(e) = res {
                panic!("Failed to compile with error {}", e);
            }
        }
        CompileTarget::Jit {} => {
            let res = jit_target(&ast);
            if let Err(e) = res {
                panic!("Failed to compile with error {}", e);
            }
        }
    }
}
