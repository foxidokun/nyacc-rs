use super::macros::check_codegen;

#[test]
fn main_signature() {
    check_codegen!(
        "fn main(a: i8) -> i32 {}",
        CompilationError "Incorrect args for main function, should be none"
    );

    check_codegen!(
        "fn main() -> i64 {}",
        CompilationError "Incorrect return type for main function, should be i32"
    );
}

#[test]
fn different_definitions() {
    check_codegen!(
        "fn foo(a: i8) -> i32;
         fn foo(a: i32) -> i32;
        ",
        CompilationError "Mismatch arg types for fn foo"
    );
}

// TODO: Check for double implementations
#[ignore]
#[test]
fn double_impl() {
    check_codegen!(
        "fn foo(a: i8) -> i32 {}
         fn foo(a: i32) -> i32 {}
        ",
        CompilationError ""
    );
}
