use crate::ast::{Expression, Statement};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Let {
    var: String,
    tp: Option<String>,
    expr: Box<dyn Expression>,
}

impl Statement for Let {}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn no_type() {
        check_ast!(
            StatementParser,
            "let a = 1;",
            ast_node!(Let, "a".into(), None, ast_node!(Int, 1))
        );
    }

    #[test]
    fn with_type() {
        check_ast!(
            StatementParser,
            "let a: u8 = 1;",
            ast_node!(Let, "a".into(), Some("u8".into()), ast_node!(Int, 1))
        );
    }
}
