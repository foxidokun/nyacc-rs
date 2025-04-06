use crate::ast::{Expression, Statement};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Return {
    pub expr: Option<Box<dyn Expression>>,
}

impl Statement for Return {}

#[cfg(test)]
mod tests {
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn retval() {
        check_ast!(
            StatementParser,
            "return 1;",
            ast_node!(Return, Some(ast_node!(Int, 1)))
        );
    }

    #[test]
    fn retvoid() {
        check_ast!(StatementParser, "return ;", ast_node!(Return, None));
    }
}
