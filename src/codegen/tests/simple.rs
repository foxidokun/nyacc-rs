use crate::codegen::tests::macros::check_codegen;

#[test]
fn sum() {
    check_codegen!(
        "fn sum(a: i32, b: i32) -> i32 { return a + b; }",
        [sum as fn(i32, i32) -> i32],
        [assert sum(1, 2) == 3]
    )
}

#[test]
fn mul() {
    check_codegen!(
        "fn mul(a: i32, b: i32) -> i32 { return a * b; }",
        [mul as fn(i32, i32) -> i32],
        [assert mul(1, 2) == 2]
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn aboba(a: i32) -> i32 {
    a
}

#[test]
fn ffi_cal() {
    check_codegen!(
        "
        fn aboba(a: i32) -> i32;

        fn mul(a: i32, b: i32) -> i32 { return aboba(a) * b; }
        ",
        [extern aboba],
        [mul as fn(i32, i32) -> i32],
        [assert mul(1, 2) == 2]
    )
}
