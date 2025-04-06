use crate::ast::Expression;
use crate::codegen::{CodegenContext, TypedValue};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use llvm_sys::core::{LLVMConstInt, LLVMConstReal, LLVMDoubleTypeInContext, LLVMIntTypeInContext};
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Float {
    pub val: f64,
}

impl Expression for Float {
    fn codegen(&self, cxt: &mut CodegenContext) -> anyhow::Result<TypedValue> {
        let val;
        unsafe {
            let ty = LLVMDoubleTypeInContext(cxt.cxt);
            val = LLVMConstReal(ty, self.val);
        }

        Ok(TypedValue {
            value: val,
            ty: cxt.definitions.get_type("f64").unwrap(),
        })
    }
}

#[derive(new, Acceptor, Debug)]
pub struct Int {
    pub val: u64,
}

impl Expression for Int {
    fn codegen(&self, cxt: &mut CodegenContext) -> anyhow::Result<TypedValue> {
        let val;
        unsafe {
            let ty = LLVMIntTypeInContext(cxt.cxt, 64);
            val = LLVMConstInt(ty, self.val, 0);
        }

        Ok(TypedValue {
            value: val,
            ty: cxt.definitions.get_type("i64").unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar;

    macro_rules! check_parser {
        ($parser:tt, $str:expr, $expect:expr) => {
            let res = grammar::$parser::new().parse($str);
            if let Err(e) = &res {
                eprintln!("Failed with err {:?}", e);
                assert!(false);
            }
            let res = res.unwrap();
            assert_eq!(res, $expect);
        };
    }

    #[test]
    fn parse_id() {
        check_parser!(IDParser, "abc", "abc");
        check_parser!(IDParser, "s2", "s2");
        check_parser!(IDParser, "under_score", "under_score");
        assert!(grammar::IDParser::new().parse("2ba").is_err());
        assert!(grammar::IDParser::new().parse("@@a").is_err());
    }

    #[test]
    fn parse_numeric() {
        check_parser!(IntParser, "3", 3);
        check_parser!(IntParser, "1000", 1000);

        check_parser!(FloatParser, "3.0", 3.0);
        check_parser!(FloatParser, "1.99", 1.99);
        check_parser!(FloatParser, "12.", 12.);
    }
}
