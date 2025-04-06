use std::{collections::HashMap, ffi::CString, rc::Rc};

use crate::{utils::nodes::Program, visitor::Acceptor};
use llvm_sys::{
    LLVMBuilder, LLVMContext, LLVMModule, LLVMValue,
    core::{
        LLVMAddFunction, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDisposeModule, LLVMFunctionType, LLVMModuleCreateWithNameInContext,
    },
    prelude::LLVMTypeRef,
};

use super::{Type, definitions::ProgramDefinitions};

#[derive(Debug, Clone)]
pub struct Value {
    pub llvm_val: *mut LLVMValue,
    pub ty: Rc<Type>,
}

pub struct VisibilityContext {
    layers: Vec<HashMap<String, Value>>,
    // Currently means function rettype, but possibly can have other meanings like in rust
    rettype: Option<Rc<Type>>,
}

impl VisibilityContext {
    pub fn new() -> Self {
        Self {
            layers: vec![],
            rettype: None,
        }
    }

    pub fn enter_layer(&mut self) {
        self.layers.push(HashMap::new());
    }

    pub fn exit_layer(&mut self) {
        let res = self.layers.pop();
        debug_assert!(res.is_some(), "Exited more than entered");
    }

    pub fn add_variable(&mut self, name: String, val: Value) {
        self.layers.last_mut().unwrap().insert(name, val);
    }

    pub fn get_variable(&mut self, name: &str) -> Option<Value> {
        for layer in self.layers.iter().rev() {
            if let Some(var) = layer.get(name) {
                return Some(var.clone());
            }
        }

        None
    }

    pub fn set_rettype(&mut self, ty: Rc<Type>) {
        debug_assert!(self.rettype.is_none(), "overwriting rettype");
        self.rettype = Some(ty);
    }

    pub fn clear_rettype(&mut self) {
        self.rettype = None;
    }
}

pub struct CodegenContext {
    pub cxt: *mut LLVMContext,
    pub builder: *mut LLVMBuilder,
    pub module: *mut LLVMModule,
    pub definitions: ProgramDefinitions,
    pub vislayers: VisibilityContext,
}

impl CodegenContext {
    pub fn prepare(prog: &Program) -> anyhow::Result<Self> {
        let mut definitions = ProgramDefinitions::new();
        prog.accept(&mut definitions)?;

        let context;
        let module;
        let builder;

        unsafe {
            context = LLVMContextCreate();
            module = LLVMModuleCreateWithNameInContext(c"nyac".as_ptr() as *const _, context);
            builder = LLVMCreateBuilderInContext(context);
        }
        assert!(!context.is_null() && !module.is_null() && !builder.is_null());

        // TODO: Populate types
        let cxt = Self {
            cxt: context,
            builder,
            module,
            definitions,
            vislayers: VisibilityContext::new(),
        };

        // -- Populate functions
        for funcname in cxt.definitions.function_names() {
            let func_type = cxt.definitions.get_func(funcname).unwrap();

            let mut llvm_arg_types: Vec<LLVMTypeRef> = Vec::with_capacity(func_type.0.len());

            for arg in &func_type.0 {
                let argtype = cxt.definitions.get_type(&arg.tp);
                if argtype.is_none() {
                    anyhow::bail!(
                        "Unknown type {} in {}-th arg of function {funcname}",
                        arg.tp,
                        llvm_arg_types.len()
                    );
                }
                let argtype = argtype.unwrap();

                llvm_arg_types.push(argtype.llvm_type(&cxt));
            }

            let llvm_func_type = unsafe {
                LLVMFunctionType(
                    func_type.1.llvm_type(&cxt),
                    llvm_arg_types.as_mut_ptr(),
                    llvm_arg_types.len() as u32,
                    false as i32,
                )
            };
            assert!(!llvm_func_type.is_null());

            let func_name_c = CString::new(funcname.clone()).unwrap();

            // Maybe save function into cxt here (?)
            let func = unsafe { LLVMAddFunction(cxt.module, func_name_c.as_ptr(), llvm_func_type) };
            assert!(!func.is_null());
        }

        Ok(cxt)
    }
}

impl Drop for CodegenContext {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.cxt);
        }
    }
}

#[cfg(test)]
mod test_vis_cxt {
    use super::*;
    use llvm_sys::LLVMValue;

    macro_rules! check_vis {
        (Ok $name:literal $exp:literal $vis:ident) => {{
            let res = $vis.get_variable($name);
            assert!(res.is_some());
            let res = res.unwrap();
            assert_eq!(res.llvm_val as usize, $exp);
        }};
        (Err $name:literal $vis:ident) => {{
            let res = $vis.get_variable($name);
            assert!(res.is_none());
        }};
        (EnterLayer $vis:ident) => {{
            $vis.enter_layer();
        }};
        (Put $name:literal $val:literal $vis:ident) => {{
            let pseudoval_ptr = $val as *const ();
            // SAFETY: Used only for comparasion and never derefenced
            let pseudoval_ptr: *mut LLVMValue  = unsafe { std::mem::transmute(pseudoval_ptr) };

            $vis.add_variable($name.into(), Value{ llvm_val: pseudoval_ptr, ty: Rc::new(Type::Void()) });
        }};
        (ExitLayer $vis:ident) => {{
            $vis.exit_layer();
        }};
        ($([$($args:tt)*]),*) => {{
            let mut vis = VisibilityContext::new();
            $(check_vis!($($args)* vis);)*
        }};
    }

    #[test]
    fn simple() {
        check_vis!(
            [Err "a"],
            [EnterLayer],
            [Err "a"],
            [Put "a" 3],
            [Ok "a" 3],
            [EnterLayer],
            [Ok "a" 3],
            [Put "a" 4],
            [Ok "a" 4],
            [ExitLayer],
            [Ok "a" 3],
            [ExitLayer],
            [Err "a"]
        );
    }
}
