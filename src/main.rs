mod ast;
mod utils;
mod visitor;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar); // synthesized by LALRPOP

fn main() {
    println!("Hello, world!");
}
