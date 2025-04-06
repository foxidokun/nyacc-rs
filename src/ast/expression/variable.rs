use std::fmt::Display;

use crate::ast::Expression;
use crate::visitor::{Acceptor, Visitor};
use derive_new::new;
use nyacc_proc::Acceptor;

#[derive(new, Acceptor, Debug)]
pub struct Variable {
    name: String,
    fields: Vec<String>,
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        for field in &self.fields {
            write!(f, ".{}", field)?;
        }

        Ok(())
    }
}

impl Expression for Variable {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::macros::{ast_node, check_ast};

    #[test]
    fn simple() {
        check_ast!(ExprParser, "a", ast_node!(Variable, "a".into(), vec![]));

        check_ast!(ExprParser, "a12", ast_node!(Variable, "a12".into(), vec![]));

        check_ast!(
            ExprParser,
            "a12_lol",
            ast_node!(Variable, "a12_lol".into(), vec![])
        );
    }

    #[test]
    fn fields() {
        check_ast!(
            ExprParser,
            "a.b",
            ast_node!(Variable, "a".into(), vec!["b".into()])
        );

        check_ast!(
            ExprParser,
            "a.b.c.d",
            ast_node!(
                Variable,
                "a".into(),
                vec!["b".into(), "c".into(), "d".into()]
            )
        );

        check_ast!(
            ExprParser,
            "a12.s5",
            ast_node!(Variable, "a12".into(), vec!["s5".into()])
        );
    }
}
