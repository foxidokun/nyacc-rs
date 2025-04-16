use crate::codegen::tests::macros::check_codegen;

#[test]
fn test_unreach_blocks() {
    // The problem here is `br if -> cond`
    // after return. We actually hacked it with
    // creating unreacheable block after return

    check_codegen!(
        "
        fn max(a: i32, b: i32) -> i32 {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
        ",
        [max as fn(i32, i32) -> i32],
        [assert max(1, 2) == 2],
        [assert max(2, 1) == 2],
        [assert max(1, 1) == 1],
        [assert max(-1, -2) == -1],
        [assert max(-1, -1) == -1],
        [assert max(i32::MAX, -2) == i32::MAX],
        [assert max(i32::MAX, i32::MIN) == i32::MAX],
        [assert max(i32::MIN, i32::MIN) == i32::MIN]
    )
}

#[test]
fn test_definitions() {
    // There was a problem of ordering type registration and function codegen

    check_codegen!("
        struct WrappedInt {
            value: i64
        }

        fn max_wrapped(a: WrappedInt, b: WrappedInt) -> i32 {
            if (a.value > b.value) {
                return a.value;
            } else {
                return b.value;
            }
        }

        fn max(a_in: i32, b_in: i32) -> i32 {
            let a: WrappedInt = WrappedInt {};
            let b = WrappedInt {};

            a.value = a_in;
            b.value = b_in;

            return max_wrapped(a, b);
        }
    ",
    [max as fn(i32, i32) -> i32],
    [assert max(1, 2) == 2],
    [assert max(-1, -2) == -1]
    )
}

/*
 * Struct layout aren't granted, because NyaC isn't repr(C) as u would expect
 *
 * C has it's own optimizations, so for example {i32, i32} is returned as i64 instead of {i32, i32}
 * U can see it in broken_struct_layout_test
 *
 * Probably u can expect repr(C) if struct is more than one register
*/

#[derive(Eq, PartialEq, Debug)]
#[repr(C)]
struct PairI64 {
    first: i64,
    second: i64,
}

#[test]
fn struct_layout() {
    check_codegen!(
        "
        struct PairI64 {first: i64, second: i64}

        fn test(x: i32) -> PairI64 {
            let a = PairI64 {};
            a.first = x;
            a.second = x;
            return a;
        }
        ",
        [test as fn(i32) -> PairI64],
        [assert test(1) == PairI64{first: 1, second: 1}],
        [assert test(2) == PairI64{first: 2, second: 2}],
        [assert test(3) == PairI64{first: 3, second: 3}],
        [assert test(4) == PairI64{first: 4, second: 4}]
    );

    check_codegen!(
        "
        struct PairI64 {first: i32, second: i32}

        fn test(x: i32) -> PairI64 {
            let a = PairI64 {};
            a.first = x;
            return a;
        }
        ",
        [test as fn(i32) -> PairI64],
        [assert test(1) == PairI64{first: 1, second: 0}],
        [assert test(2) == PairI64{first: 2, second: 0}],
        [assert test(3) == PairI64{first: 3, second: 0}],
        [assert test(4) == PairI64{first: 4, second: 0}]
    );
}

#[derive(Eq, PartialEq, Debug)]
#[repr(C)]
struct PairI32 {
    first: i32,
    second: i32,
}

#[test]
fn broken_struct_layout() {
    check_codegen!(
        "
        struct PairI32 {first: i32, second: i32}

        fn test(x: i32) -> PairI32 {
            let a = PairI32 {};
            a.first = x;
            a.second = x;
            return a;
        }
        ",
        [test as fn(i32) -> PairI32],
        [assert test(1) == PairI32{first: 1, second: 0}],
        [assert test(2) == PairI32{first: 2, second: 0}],
        [assert test(3) == PairI32{first: 3, second: 0}],
        [assert test(4) == PairI32{first: 4, second: 0}]
    );
}

#[test]
fn alloca_placement() {
    // There was a probleb with inserting allocas after branch
    // in entry block, leading to broken numeration and therefore segfault
    // in jit&optimizer

    check_codegen!(
        "
        fn test() -> i32 {
            let x = 1;
            if (x) {
                let x = 0;
            }
            return x;
        }
        ",
        [test as fn() -> i32],
        [assert test() == 1]
    );
}
