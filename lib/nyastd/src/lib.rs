#[unsafe(no_mangle)]
pub extern "C" fn print_int(a: i64) {
    println!("{a}");
}

// TODO: rewrite with proc macro magic
pub fn register_functions<T>(mut callback: T)
where T: FnMut(&'static str, *mut ())
{
    macro_rules! export_symbol {
        ($symbol:ident) => {
            callback(stringify!($symbol), $symbol as *mut ());
        };
    }

    export_symbol!(print_int);
}
