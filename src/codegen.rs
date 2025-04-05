use std::{collections::HashMap, rc::Rc};

use anyhow::Context;

use crate::{
    ast::TypedArg,
    utils::nodes::{Program, StructDef},
    visitor::{Acceptor, Visitor},
};

trait Type {}

struct CustomType {
    // Field name -> (position, Type)
    fields: HashMap<String, (usize, Rc<dyn Type>)>,
}

impl CustomType {
    fn from_def(structdef: &StructDef, types: &ProgramDefinitions) -> anyhow::Result<Self> {
        let mut type_fields = HashMap::new();

        for (pos, field) in structdef.fields.iter().enumerate() {
            let field_type = types.get_type(&field.tp);
            if let Some(field_type) = field_type {
                type_fields.insert(field.name.clone(), (pos, field_type));
            } else {
                anyhow::bail!(
                    "Unknown type {} in definition of {}",
                    field.tp,
                    structdef.name
                );
            }
        }

        Ok(Self {
            fields: type_fields,
        })
    }
}

impl Type for CustomType {}

struct IntType {
    byte_size: u8,
}

impl Type for IntType {}

struct FloatType {
    byte_size: i8,
}

impl Type for FloatType {}

struct VoidType {}
impl Type for VoidType {}

struct ProgramDefinitions {
    /// typename => typedata
    types: HashMap<String, Rc<dyn Type>>,
    /// func_name => func_info
    functions: HashMap<String, (Vec<TypedArg>, Rc<dyn Type>)>,
}

impl ProgramDefinitions {
    fn new() -> Self {
        let mut me = Self {
            types: HashMap::new(),
            functions: HashMap::new(),
        };

        // Insert basic types
        me.types.insert("void".into(), Rc::new(VoidType {}));

        me.types
            .insert("i8".into(), Rc::new(IntType { byte_size: 1 }));
        me.types
            .insert("i16".into(), Rc::new(IntType { byte_size: 2 }));
        me.types
            .insert("i32".into(), Rc::new(IntType { byte_size: 4 }));
        me.types
            .insert("i64".into(), Rc::new(IntType { byte_size: 8 }));

        me.types
            .insert("f32".into(), Rc::new(FloatType { byte_size: 4 }));
        me.types
            .insert("f64".into(), Rc::new(FloatType { byte_size: 8 }));

        // TODO: Insert std library functions [until we support includes]

        me
    }

    fn add_func(
        &mut self,
        name: &str,
        args: &Vec<TypedArg>,
        ret: Rc<dyn Type>,
    ) -> anyhow::Result<()> {
        let res = self.functions.get(name);
        if let Some((ex_args, ex_ret)) = res {
            if ex_args != args {
                anyhow::bail!("Mismatch arg types for fn {}", name);
            }

            if !std::ptr::addr_eq(Rc::as_ptr(ex_ret), Rc::as_ptr(&ret)) {
                anyhow::bail!("Mismatch ret types for fn {}", name);
            }
        } else {
            self.functions.insert(name.into(), (args.clone(), ret));
        }

        Ok(())
    }

    fn get_type(&self, name: &str) -> Option<Rc<dyn Type>> {
        self.types.get(name).cloned()
    }
}

impl Visitor for ProgramDefinitions {
    fn visit_program(&mut self, node: &crate::utils::nodes::Program) -> anyhow::Result<()> {
        for block in &node.blocks {
            block.accept(self)?;
        }

        Ok(())
    }

    fn visit_funcdef(&mut self, node: &crate::utils::nodes::FuncDef) -> anyhow::Result<()> {
        let rettype = self.get_type(&node.rettype).context(format!(
            "Unknown type {} in definition of {}",
            node.rettype, node.name
        ))?;

        self.add_func(&node.name, &node.args, rettype)
    }

    fn visit_funcimpl(&mut self, node: &crate::utils::nodes::FuncImpl) -> anyhow::Result<()> {
        let rettype = self.get_type(&node.rettype).context(format!(
            "Unknown type {} in definition of {}",
            node.rettype, node.name
        ))?;

        self.add_func(&node.name, &node.args, rettype)
    }

    fn visit_structdef(&mut self, node: &crate::utils::nodes::StructDef) -> anyhow::Result<()> {
        if self.types.contains_key(&node.name) {
            anyhow::bail!("Redefinition of {} type", node.name);
        }

        let ty = Rc::new(CustomType::from_def(node, self)?);
        self.types.insert(node.name.clone(), ty);
        Ok(())
    }
}

pub fn compile(prog: &Program) -> anyhow::Result<()> {
    let mut definitions = ProgramDefinitions::new();
    prog.accept(&mut definitions)?;

    for ty in &definitions.types {
        println!("We got type {}", ty.0);
    }

    for func in &definitions.functions {
        println!("We got function {}", func.0);
    }

    Ok(())
}
