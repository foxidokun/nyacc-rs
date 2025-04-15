use super::macros::check_codegen;

#[test]
fn main_signature() {
    check_codegen!(
        "fn main(a: i8) -> void {}",
        CompilationError "Incorrect args for main function, should be none"
    );

    check_codegen!(
        "fn main() -> i64 {}",
        CompilationError "Incorrect return type for main function, should be none"
    );
}

#[test]
fn double_func_def() {
    check_codegen!(
        "fn foo(a: i8) -> i32 {}
         fn foo(a: i8) -> i32 {}
        ",
        CompilationError "Redefenition of func foo"
    );
}

#[test]
fn double_func_def_diff_arg() {
    check_codegen!(
        "fn foo(a: i32) -> i32 {}
         fn foo(a: i8) -> i32 {}
        ",
        CompilationError "Redefenition of func foo"
    );
}

#[test]
fn double_type_def() {
    check_codegen!(
        "struct A {}
         struct A {}
        ",
        CompilationError "Redefinition of A type"
    );
}
