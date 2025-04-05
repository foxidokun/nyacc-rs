use crate::ast::{Expression, Statement, expression::Variable};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Assignment {
    pub var: Variable,
    pub expr: Box<dyn Expression>,
}

impl Statement for Assignment {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Comparator;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::utils::nodes::*;

    #[test]
    fn simple() {
        check_ast!(
            StatementParser,
            "a = 12;",
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(Int, 12)
            )
        );

        check_ast!(
            StatementParser,
            "a = b;",
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(Variable, "b".into(), vec![])
            )
        );
    }

    #[test]
    fn complex() {
        check_ast!(
            StatementParser,
            "a = b == c;",
            ast_node!(
                Assignment,
                Variable::new("a".into(), vec![]),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "b".into(), vec![]),
                    Comparator::EQ,
                    ast_node!(Variable, "c".into(), vec![])
                )
            )
        );
    }
}
