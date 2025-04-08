use std::io::Write;

#[unsafe(no_mangle)]
pub extern "C" fn print_int(a: i64) {
    println!("Output: {a}");
}

#[unsafe(no_mangle)]
pub extern "C" fn read_int() -> i64 {
    print!("Input: ");
    std::io::stdout().flush().unwrap();
    let mut input_line = String::new();
    std::io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");
    let x: i64 = input_line.trim().parse().expect("Input not an integer");

    x
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
    export_symbol!(read_int);
}
