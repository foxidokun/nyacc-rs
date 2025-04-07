use std::{collections::HashMap, ffi::CString, rc::Rc};

use crate::{utils::nodes::Program, visitor::Acceptor};
use llvm_sys::{
    LLVMBuilder, LLVMContext, LLVMModule, LLVMValue,
    core::{
        LLVMAddFunction, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDisposeModule, LLVMFunctionType, LLVMModuleCreateWithNameInContext,
    },
    prelude::{LLVMTypeRef, LLVMValueRef},
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
    cur_func: Option<(*mut LLVMValue, Rc<Type>)>,
}

impl VisibilityContext {
    pub fn new() -> Self {
        Self {
            layers: vec![],
            cur_func: None,
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

    pub fn enter_function(&mut self, func: LLVMValueRef, rettype: Rc<Type>) {
        debug_assert!(self.cur_func.is_none(), "overwriting cur func");

        self.cur_func = Some((func, rettype));
    }

    pub fn cur_fun(&self) -> Option<&(LLVMValueRef, Rc<Type>)> {
        self.cur_func.as_ref()
    }

    pub fn exit_function(&mut self) {
        self.cur_func = None;
    }
}

pub struct TypeCache {
    pub funcs: HashMap<String, LLVMTypeRef>,
}

impl TypeCache {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new(),
        }
    }

    pub fn store_func(&mut self, name: String, ty: LLVMTypeRef) {
        self.funcs.insert(name, ty);
    }

    pub fn get_func(&mut self, name: &str) -> Option<&LLVMTypeRef> {
        self.funcs.get(name)
    }
}

pub struct CodegenContext {
    pub cxt: *mut LLVMContext,
    pub builder: *mut LLVMBuilder,
    pub module: *mut LLVMModule,
    pub definitions: ProgramDefinitions,
    pub vislayers: VisibilityContext,
    pub type_cache: TypeCache,
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
        let mut cxt = Self {
            cxt: context,
            builder,
            module,
            definitions,
            vislayers: VisibilityContext::new(),
            type_cache: TypeCache::new(),
        };

        // -- Populate functions
        for funcname in cxt.definitions.function_names() {
            let func_type = cxt.definitions.get_func(funcname).unwrap();

            let mut llvm_arg_types: Vec<LLVMTypeRef> =
                func_type.0.iter().map(|t| t.llvm_type(&cxt)).collect();

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
            cxt.type_cache.store_func(funcname.clone(), llvm_func_type);
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
