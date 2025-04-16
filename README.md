## Nyacc Reborn

NyaC compiler rewritten in Rust

### Intro
NyaC compiler is simple LLVM based compiler for some C-like language NyaC, which supports
- functions, if/for/while
- visibility scopes
- custom types (ctors are zero initialized)
- int, float, void types
- linking with own standart library

For simple syntax example you can check [this example](examples/simple.nya)

### NyaCC as MIPT course project
See [study project description](proj_description.md)

### Use NyaCC
Overall info:
```bash
Usage: nyacc --input <FILE> <COMMAND>

Commands:
  ast   Emit generated AST tree
  ir    Emit generated llvm IR
  jit   Compile & execute via LLVM JIT
  help  Print this message or the help of the given subcommand(s)

Options:
  -i, --input <FILE>  Path of input NyaC program
  -h, --help          Print help
  -V, --version       Print version
```
Examples:
```bash
nyacc --input examples/simple.nya jit
nyacc --input examples/simple.nya ast -o ./out.ast
nyacc --input examples/simple.nya ir -o ./out.ast #--no-optimize
```

jit & ir are target are optimized with `-O2 -march=native`, but it can be disabled for ir target

### Build NyaCC
**For Linux:** You can download artifacts of `build_release` job on master, it contains static linked nyacc executable

You will need LLVM19.1. With that, you can just build it as any rust project with 
```bash
cargo build --release
```

### stdlib
You can check `export_symbol!` in [sources](lib/nyastd/src/lib.rs), but currently we have only 2 functions:
```
print_int(i64) -> void
read_int() -> i64
```

### Roadmap
NyaC:
- [_] Support comments
- [_] ELF target
- [_] Pointers (AST + codegen)
- [_] va arg functions
- [_] Strings as i8* + std functions for them

NyaCC:
- [_] Embed stdlib definitions into prog
- [_] Simplify stdlib symbol & types export via proc_macro
- [_] Refactor
- And much more...