use std::{collections::HashMap, ffi::CString, fmt::Display, rc::Rc};

use anyhow::Context;
use llvm_sys::{
    core::{
        LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMGetTypeByName2, LLVMIntTypeInContext,
        LLVMVoidTypeInContext,
    },
    prelude::LLVMTypeRef,
};

use crate::{ast::TypedArg, utils::nodes::StructDef, visitor::Visitor};

use super::CodegenContext;

#[derive(PartialEq, Eq, Debug)]
pub struct CustomType {
    pub name: String,
    // Field name -> (position, Type)
    pub fields: HashMap<String, (usize, Rc<Type>)>,
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
            name: structdef.name.clone(),
            fields: type_fields,
        })
    }

    pub fn llvm_type(&self, cxt: &CodegenContext) -> LLVMTypeRef {
        let name = CString::new(self.name.clone()).unwrap();
        let res = unsafe { LLVMGetTypeByName2(cxt.cxt, name.as_ptr()) };
        assert!(!res.is_null());
        res
    }

    #[cfg(test)]
    fn test_sample() -> Self {
        Self {
            name: "test".into(),
            fields: HashMap::new(),
        }
    }
}

impl Display for CustomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct IntType {
    bitwidth: u8,
}

impl IntType {
    pub fn llvm_type(&self, cxt: &CodegenContext) -> LLVMTypeRef {
        unsafe { LLVMIntTypeInContext(cxt.cxt, self.bitwidth as u32) }
    }
}

impl Display for IntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i{}", self.bitwidth)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FloatType {
    pub bitwidth: u8,
}

impl FloatType {
    pub fn llvm_type(&self, cxt: &CodegenContext) -> LLVMTypeRef {
        if self.bitwidth == 64 {
            unsafe { LLVMDoubleTypeInContext(cxt.cxt) }
        } else {
            assert!(self.bitwidth == 32, "Unexpected float size");
            unsafe { LLVMFloatTypeInContext(cxt.cxt) }
        }
    }
}

impl Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.bitwidth)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Type {
    Void(),
    Float(FloatType),
    Int(IntType),
    Custom(CustomType),
}

impl Type {
    pub fn llvm_type(&self, cxt: &CodegenContext) -> LLVMTypeRef {
        let res = match self {
            Type::Void() => unsafe { LLVMVoidTypeInContext(cxt.cxt) },
            Type::Float(float_type) => float_type.llvm_type(cxt),
            Type::Int(int_type) => int_type.llvm_type(cxt),
            Type::Custom(custom_type) => custom_type.llvm_type(cxt),
        };

        assert!(!res.is_null());

        res
    }

    /// This type can perform arithmetic
    pub fn arithmetic(&self) -> bool {
        match self {
            Type::Void() | Type::Custom(_) => false,
            Type::Float(_) | Type::Int(_) => true,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void() => write!(f, "void"),
            Type::Float(float_type) => float_type.fmt(f),
            Type::Int(int_type) => int_type.fmt(f),
            Type::Custom(custom_type) => custom_type.fmt(f),
        }
    }
}

macro_rules! bad_type {
    (1, $ty:tt, $name:ident) => {{
        if let Type::$ty(_) = $name.as_ref() {
            anyhow::bail!("Can't find common type when type is {}", $name);
        }
    }};

    (0, $ty:tt, $name:ident) => {{
        if let Type::$ty() = $name.as_ref() {
            anyhow::bail!("Can't find common type when type is {}", $name);
        }
    }};
}

impl Type {
    pub fn common_type(lhs: &Rc<Type>, rhs: &Rc<Type>) -> anyhow::Result<Rc<Type>> {
        bad_type!(0, Void, lhs);
        bad_type!(0, Void, rhs);
        bad_type!(1, Custom, lhs);
        bad_type!(1, Custom, rhs);

        if let Type::Float(l_v) = lhs.as_ref() {
            if let Type::Float(r_v) = rhs.as_ref() {
                if l_v > r_v {
                    return Ok(lhs.clone());
                } else {
                    return Ok(rhs.clone());
                }
            } else {
                return Ok(lhs.clone());
            }
        } else if let Type::Float(_) = rhs.as_ref() {
            return Ok(rhs.clone());
        }

        // They are both ints

        if let Type::Int(l_v) = lhs.as_ref() {
            if let Type::Int(r_v) = rhs.as_ref() {
                if l_v > r_v {
                    return Ok(lhs.clone());
                } else {
                    return Ok(rhs.clone());
                }
            }
        }

        unreachable!(" Ifs above should never miss");
    }
}

type FuncType = (Vec<Rc<Type>>, Rc<Type>);

pub struct ProgramDefinitions {
    /// typename => typedata
    types: HashMap<String, Rc<Type>>,
    /// func_name => func_info
    functions: HashMap<String, Rc<FuncType>>,
}

impl ProgramDefinitions {
    pub fn new() -> Self {
        let mut me = Self {
            types: HashMap::new(),
            functions: HashMap::new(),
        };

        // Insert basic types
        me.types.insert("void".into(), Rc::new(Type::Void()));

        me.types
            .insert("bool".into(), Rc::new(Type::Int(IntType { bitwidth: 1 })));

        me.types
            .insert("i8".into(), Rc::new(Type::Int(IntType { bitwidth: 8 })));
        me.types
            .insert("i16".into(), Rc::new(Type::Int(IntType { bitwidth: 16 })));
        me.types
            .insert("i32".into(), Rc::new(Type::Int(IntType { bitwidth: 32 })));
        me.types
            .insert("i64".into(), Rc::new(Type::Int(IntType { bitwidth: 64 })));

        me.types.insert(
            "f32".into(),
            Rc::new(Type::Float(FloatType { bitwidth: 32 })),
        );
        me.types.insert(
            "f64".into(),
            Rc::new(Type::Float(FloatType { bitwidth: 64 })),
        );

        // TODO: Insert std library functions [until we support includes]

        me
    }

    fn add_func(&mut self, name: &str, args: &Vec<TypedArg>, ret: Rc<Type>) -> anyhow::Result<()> {
        let mut processed_args = Vec::with_capacity(args.len());
        for arg in args {
            let argtype = self.get_type(&arg.tp);
            if argtype.is_none() {
                anyhow::bail!(
                    "Unknown type {} in {}-th arg of function {name}",
                    arg.tp,
                    processed_args.len()
                );
            }
            processed_args.push(argtype.unwrap());
        }

        /* Check main function signature */
        if name == "main" {
            if *ret != Type::Void() {
                anyhow::bail!("Incorrect return type for main function, should be none");
            }
            if !args.is_empty() {
                anyhow::bail!("Incorrect args for main function, should be none");
            }
        }

        let res = self.functions.get(name);
        if let Some(ex_type) = res {
            let ex_args = &ex_type.0;
            let ex_ret = &ex_type.1;

            if ex_args != &processed_args {
                anyhow::bail!("Mismatch arg types for fn {}", name);
            }

            if &ret != ex_ret {
                anyhow::bail!("Mismatch ret types for fn {}", name);
            }
        } else {
            self.functions
                .insert(name.into(), Rc::new((processed_args, ret)));
        }

        Ok(())
    }

    pub fn get_func(&self, name: &str) -> Option<&Rc<FuncType>> {
        self.functions.get(name)
    }

    pub fn get_type(&self, name: &str) -> Option<Rc<Type>> {
        self.types.get(name).cloned()
    }

    pub fn function_names(&self) -> impl IntoIterator<Item = &String> {
        self.functions.keys()
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

        let ty = Rc::new(Type::Custom(CustomType::from_def(node, self)?));
        self.types.insert(node.name.clone(), ty);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! common_type_test {
        (Ok, $lhs:expr, $rhs:expr, $common:expr) => {{
            let lhs_wr = Rc::new($lhs);
            let rhs_wr = Rc::new($rhs);

            let res = Type::common_type(&lhs_wr, &rhs_wr);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(*res, $common);
        }};
        (Err, $lhs:expr, $rhs:expr) => {{
            let lhs_wr = Rc::new($lhs);
            let rhs_wr = Rc::new($rhs);

            let res = Type::common_type(&lhs_wr, &rhs_wr);
            assert!(res.is_err());
        }};
    }

    fn int(size: u8) -> Type {
        Type::Int(IntType { bitwidth: size })
    }

    fn float(size: u8) -> Type {
        Type::Float(FloatType { bitwidth: size })
    }

    #[test]
    fn common_type() {
        // All ints
        common_type_test!(Ok, int(2), int(2), int(2));
        common_type_test!(Ok, int(1), int(8), int(8));
        common_type_test!(Ok, int(8), int(1), int(8));

        // With float
        common_type_test!(Ok, int(1), float(1), float(1));
        common_type_test!(Ok, int(8), float(1), float(1));
        common_type_test!(Ok, float(8), int(7), float(8));

        // Can't negotiate with void
        common_type_test!(Err, int(1), Type::Void());
        common_type_test!(Err, float(1), Type::Void());
        common_type_test!(Err, Type::Void(), Type::Void());
        common_type_test!(Err, Type::Void(), int(1));
        common_type_test!(Err, Type::Void(), float(1));

        // Can't negotiate with Custom Type
        common_type_test!(Err, Type::Custom(CustomType::test_sample()), float(1));
        common_type_test!(Err, Type::Custom(CustomType::test_sample()), int(1));
        common_type_test!(Err, Type::Custom(CustomType::test_sample()), Type::Void());
        common_type_test!(Err, float(1), Type::Custom(CustomType::test_sample()));
        common_type_test!(Err, int(1), Type::Custom(CustomType::test_sample()));
        common_type_test!(Err, Type::Void(), Type::Custom(CustomType::test_sample()));
    }
}
