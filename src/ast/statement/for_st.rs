use crate::ast::{Expression, Statement};
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct For {
    start: Box<dyn Statement>,
    check: Box<dyn Expression>,
    exec: Box<dyn Statement>,
    body: Vec<Box<dyn Statement>>,
}

impl Statement for For {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::macros::{ast_node, check_ast};
    use crate::ast::{Comparator, OpType};
    use crate::utils::nodes::*;

    #[test]
    fn existing_val_empty() {
        check_ast!(
            StatementParser,
            "for (a = 3; a < 100; a = a * 2) {}",
            ast_node!(
                For,
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(Int, 3)
                ),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "a".into(), vec![]),
                    Comparator::LT,
                    ast_node!(Int, 100)
                ),
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    )
                ),
                vec![]
            )
        );
    }

    #[test]
    fn existing_val_body() {
        check_ast!(
            StatementParser,
            "for (a = 3; a < 100; a = a * 2) {a = 3; a = 7;}",
            ast_node!(
                For,
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(Int, 3)
                ),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "a".into(), vec![]),
                    Comparator::LT,
                    ast_node!(Int, 100)
                ),
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    )
                ),
                vec![
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 3)
                    ),
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 7)
                    )
                ]
            )
        );
    }

    #[test]
    fn new_val() {
        check_ast!(
            StatementParser,
            "for (let a: u8 = 3; a < 100; a = a * 2) {a = 3; a = 7;}",
            ast_node!(
                For,
                ast_node!(Let, "a".into(), Some("u8".into()), ast_node!(Int, 3)),
                ast_node!(
                    Compare,
                    ast_node!(Variable, "a".into(), vec![]),
                    Comparator::LT,
                    ast_node!(Int, 100)
                ),
                ast_node!(
                    Assignment,
                    Variable::new("a".into(), vec![]),
                    ast_node!(
                        Arithmetic,
                        ast_node!(Variable, "a".into(), vec![]),
                        OpType::Mul,
                        ast_node!(Int, 2)
                    )
                ),
                vec![
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 3)
                    ),
                    ast_node!(
                        Assignment,
                        Variable::new("a".into(), vec![]),
                        ast_node!(Int, 7)
                    )
                ]
            )
        );
    }
}
