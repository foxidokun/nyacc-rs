use std::i32;

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
pub extern "C" fn test_id(a: i32) -> i32 {
    a
}

#[test]
fn ffi_cal() {
    check_codegen!(
        "
        fn test_id(a: i32) -> i32;

        fn mul(a: i32, b: i32) -> i32 { return test_id(a) * b; }
        ",
        [extern test_id],
        [mul as fn(i32, i32) -> i32],
        [assert mul(1, 2) == 2],
        [assert mul(1, 0) == 0],
        [assert mul(0, 2) == 0],
        [assert mul(-1, 2) == -2]
    )
}

#[test]
fn func_cal() {
    check_codegen!(
        "
        fn id(a: i32) -> i32 { return a; }

        fn mul(a: i32, b: i32) -> i32 { return id(a) * b; }
        ",
        [mul as fn(i32, i32) -> i32],
        [assert mul(1, 2) == 2],
        [assert mul(1, 0) == 0],
        [assert mul(0, 2) == 0],
        [assert mul(-1, 2) == -2]
    )
}

#[test]
fn test_for_loop() {
    check_codegen!(
        "
        fn test(end: i32) -> i32 {
            let accum: i64 = 0;
            for (let i = 0; i < end; i = i + 1) {
                accum = accum + i;
            }
            return accum;
        }
        ",
        [test as fn(i32) -> i32],
        [assert test(0) == 0],
        [assert test(1) == 0],
        [assert test(2) == 1],
        [assert test(3) == 3],
        [assert test(4) == 6],
        [assert test(5) == 10],
        [assert test(6) == 15]
    )
}

#[test]
fn test_for_loop_external_var() {
    check_codegen!(
        "
        fn test(end: i32) -> i32 {
            let accum: i64 = 0;
            let i = 100;
            for (i = 0; i < end; i = i + 1) {
                accum = accum + i;
            }
            return accum;
        }
        ",
        [test as fn(i32) -> i32],
        [assert test(0) == 0],
        [assert test(1) == 0],
        [assert test(2) == 1],
        [assert test(3) == 3],
        [assert test(4) == 6],
        [assert test(5) == 10],
        [assert test(6) == 15]
    )
}

#[test]
fn test_recursion() {
    check_codegen!(
        "
        fn fib(n: i32) -> i32 {
            if (n == 0) {
                return 1;
            }

            if (n == 1) {
                return 1;
            }

            return fib(n-1) + fib(n-2);
        }
        ",
        [fib as fn(i32) -> i32],
        [assert fib(0) == 1],
        [assert fib(1) == 1],
        [assert fib(2) == 2],
        [assert fib(3) == 3],
        [assert fib(4) == 5],
        [assert fib(5) == 8],
        [assert fib(6) == 13]
    )
}

#[test]
fn test_while() {
    check_codegen!(
        "
        fn test(end: i32) -> i32 {
            let accum: i64 = 0;
            let i: i8 = 0;
            while (i < end) {
                accum = accum + i;
                i = i + 1;
            }
            return accum;
        }
        ",
        [test as fn(i32) -> i32],
        [assert test(0) == 0],
        [assert test(1) == 0],
        [assert test(2) == 1],
        [assert test(3) == 3],
        [assert test(4) == 6],
        [assert test(5) == 10],
        [assert test(6) == 15]
    )
}

#[test]
fn test_overflow() {
    check_codegen!(
        "
        fn test() -> i32 {
            let overflowed: i8 = 0;
            for (let normal = 0; normal < 256; normal = normal + 1) {
                overflowed = overflowed + 1;
            }
            return overflowed;
        }
        ",
        [test as fn() -> i32],
        [assert test() == 0]
    );
}

#[test]
fn test_void_func() {
    check_codegen!(
        "
        fn nothing() -> void {
            return;
        }

        fn nothing_empty() -> void {}

        fn test() -> i32 {
            nothing();
            nothing_empty();
            return 0;
        }
        ",
        [test as fn() -> i32],
        [assert test() == 0]
    );
}

#[test]
fn test_unreachable() {
    check_codegen!(
        "
        fn test() -> i32 {
            return 0;
            return 1;
            return 2;
            return 3;
        }
        ",
        [test as fn() -> i32],
        [assert test() == 0]
    );
}
