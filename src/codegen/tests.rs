#![cfg(test)]
mod macros {
    macro_rules! check_codegen {
        ($code: expr, $([$($args:tt)*]),*) => {{
            use crate::codegen::{CodegenContext, JitEngine};
            use crate::ast::Statement;

            let prog = crate::grammar::ProgramParser::new().parse($code.into()).unwrap();
            let mut cxt = CodegenContext::prepare(&prog).unwrap();

            prog.codegen(&mut cxt).unwrap();
            let ee = JitEngine::from_codegen_cxt(cxt);
            // Codegen each rule
            $( check_codegen!(ee $($args)*); )*
        }};
        ($ee:ident $name:tt as fn($($args:ty),*) -> $ret:ty) => {
            let func_ptr = $ee.get_func_addr(stringify!($name));
            if let Err(e) = func_ptr {
                panic!("Can't find function {} in test code with err {}", stringify!($name), e);
            }
            let func_ptr = func_ptr.unwrap();

            let $name: extern "C" fn($($args),*) -> $ret = unsafe { std::mem::transmute(func_ptr) };
        };
        ($ee:ident assert $name:ident($($arg:tt),*) == $exp:expr) => {{
            assert_eq!($name($($arg),*), $exp);
        }};
        ($ee:ident extern $name:ident) => {{
            $ee.add_func_mapping(stringify!($name), $name as *mut ()).unwrap();
        }};
    }

    pub(super) use check_codegen;
}

mod simple;
