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
