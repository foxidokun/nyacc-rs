use std::io::Write;

use crate::visitor::{Acceptor, Visitor};

use super::statement::Program;

struct ASTPrinter<'a, T: Write> {
    writer: &'a mut T,
    ident: usize,
}

impl<'a, T: Write> ASTPrinter<'a, T> {
    fn new(writer: &'a mut T) -> Self {
        Self { writer, ident: 0 }
    }

    fn shift(&mut self) -> anyhow::Result<()> {
        let padding = String::from(" ");
        let padding = padding.repeat(self.ident);
        Ok(write!(self.writer, "{}", padding)?)
    }
}

macro_rules! print_subtree {
    ($self:ident, $name:expr, $subnode:expr) => {{
        $self.shift()?;
        $self.ident += 1;
        writeln!($self.writer, "- {}:", $name)?;
        $self.ident += 2;
        $subnode.accept($self)?;
        $self.ident -= 3;
    }};
}

macro_rules! print_body {
    ($self:ident, $name:expr, $body:expr) => {{
        $self.shift()?;
        $self.ident += 1;
        writeln!($self.writer, "- {}:", $name)?;
        $self.ident += 2;
        for node in &$body {
            node.accept($self)?;
        }
        $self.ident -= 3;
    }};
}

impl<T: Write> Visitor for ASTPrinter<'_, T> {
    fn visit_arithmetic(&mut self, node: &super::expression::Arithmetic) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Arithmetic node (sign: {})", node.op)?;
        print_subtree!(self, "LHS", node.lhs);
        print_subtree!(self, "RHS", node.rhs);
        Ok(())
    }

    fn visit_assignment(&mut self, node: &super::statement::Assignment) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Assignment to var {}", node.var)?;
        print_subtree!(self, "Value", node.expr);
        Ok(())
    }

    fn visit_compare(&mut self, node: &super::expression::Compare) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Compare node (comparator: {})", node.cmp)?;
        print_subtree!(self, "LHS", node.lhs);
        print_subtree!(self, "RHS", node.rhs);
        Ok(())
    }

    fn visit_exprstatement(
        &mut self,
        node: &super::statement::ExprStatement,
    ) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "ExprStatement node")?;
        print_subtree!(self, "Expr", node.expr);
        Ok(())
    }

    fn visit_float(&mut self, node: &super::expression::Float) -> anyhow::Result<()> {
        self.shift()?;
        Ok(writeln!(self.writer, "Float {}", node.val)?)
    }

    fn visit_int(&mut self, node: &super::expression::Int) -> anyhow::Result<()> {
        self.shift()?;
        Ok(writeln!(self.writer, "Int {}", node.val)?)
    }

    fn visit_for(&mut self, node: &super::statement::For) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "For Loop")?;
        print_subtree!(self, "Start", node.start);
        print_subtree!(self, "Check", node.check);
        print_subtree!(self, "Step", node.step);
        print_body!(self, "Body", node.body);
        Ok(())
    }

    fn visit_funcdef(&mut self, node: &super::statement::FuncDef) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(
            self.writer,
            "FuncDef of fn {} -> {}",
            node.name, node.rettype
        )?;
        self.shift()?;
        self.ident += 1;
        writeln!(self.writer, "- Args:")?;
        self.ident += 2;
        for arg in &node.args {
            self.shift()?;
            writeln!(self.writer, "{}: {}", arg.name, arg.tp)?;
        }
        self.ident -= 3;

        Ok(())
    }

    fn visit_funcimpl(&mut self, node: &super::statement::FuncImpl) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(
            self.writer,
            "FuncImpl of fn {} -> {}",
            node.name, node.rettype
        )?;
        self.shift()?;
        self.ident += 1;
        writeln!(self.writer, "- Args:")?;
        self.ident += 2;
        for arg in &node.args {
            self.shift()?;
            writeln!(self.writer, "{}: {}", arg.name, arg.tp)?;
        }
        self.ident -= 3;

        print_body!(self, "Body", node.body);

        Ok(())
    }

    fn visit_functioncall(&mut self, node: &super::expression::FunctionCall) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Calling function {}", node.name)?;
        print_body!(self, "Args", node.args);
        Ok(())
    }

    fn visit_if(&mut self, node: &super::statement::If) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "If")?;
        print_subtree!(self, "Condition", node.check);
        print_body!(self, "True Body", node.true_body);
        if let Some(body) = &node.else_body {
            print_body!(self, "Else Body", *body);
        }
        Ok(())
    }

    fn visit_let(&mut self, node: &super::statement::Let) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Let to var {} of type {:?}", node.var, node.tp)?;
        print_subtree!(self, "Value", node.expr);
        Ok(())
    }

    fn visit_not(&mut self, node: &super::expression::Not) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Not")?;
        print_subtree!(self, "Value", node.expr);
        Ok(())
    }

    fn visit_program(&mut self, node: &super::statement::Program) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Program")?;
        print_body!(self, "Blocks", node.blocks);
        Ok(())
    }

    fn visit_structdef(&mut self, node: &super::statement::StructDef) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "StructDef of type {}", node.name)?;
        self.shift()?;
        self.ident += 1;
        writeln!(self.writer, "- Fields:")?;
        self.ident += 2;
        for arg in &node.fields {
            self.shift()?;
            writeln!(self.writer, "{}: {}", arg.name, arg.tp)?;
        }
        self.ident -= 3;

        Ok(())
    }

    fn visit_unaryminus(&mut self, node: &super::expression::UnaryMinus) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "UnaryMinus")?;
        print_subtree!(self, "Value", node.expr);
        Ok(())
    }

    fn visit_variable(&mut self, node: &super::expression::Variable) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Variable {}", node)?;
        Ok(())
    }

    fn visit_while(&mut self, node: &super::statement::While) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "While Loop")?;
        print_subtree!(self, "Condition", node.cond);
        print_body!(self, "Body", node.body);
        Ok(())
    }

    fn visit_structctor(&mut self, node: &super::expression::StructCtor) -> anyhow::Result<()> {
        self.shift()?;
        writeln!(self.writer, "Struct Ctor of type {}", node.name)?;
        print_body!(self, "Args", node.args);
        Ok(())
    }

    fn visit_return(&mut self, node: &super::statement::Return) -> anyhow::Result<()> {
        self.shift()?;
        if let Some(retval) = &node.expr {
            writeln!(self.writer, "Return val")?;
            print_subtree!(self, "Val", retval);
        } else {
            writeln!(self.writer, "Return void")?;
        }

        Ok(())
    }
}

pub fn print_ast<T: Write>(writer: &mut T, program: &Program) -> anyhow::Result<()> {
    let mut printer = ASTPrinter::new(writer);
    program.accept(&mut printer)
}
