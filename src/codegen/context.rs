use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_void},
    ptr::null_mut,
    rc::Rc,
};

use crate::{utils::nodes::Program, visitor::Acceptor};
use llvm_sys::{
    LLVMBuilder, LLVMContext, LLVMLinkage, LLVMModule, LLVMValue,
    core::{
        LLVMAddFunction, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDisposeModule, LLVMFunctionType, LLVMGetNamedFunction,
        LLVMModuleCreateWithNameInContext, LLVMSetLinkage,
    },
    execution_engine::{
        LLVMAddGlobalMapping, LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine,
        LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMOpaqueExecutionEngine,
    },
    prelude::{LLVMTypeRef, LLVMValueRef},
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
};

use super::{Type, TypedValue, definitions::ProgramDefinitions};

pub struct VisibilityContext {
    layers: Vec<HashMap<String, TypedValue>>,
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

    pub fn add_variable(&mut self, name: String, val: TypedValue) {
        self.layers.last_mut().unwrap().insert(name, val);
    }

    pub fn get_variable(&mut self, name: &str) -> Option<TypedValue> {
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
            // Set external linkage
            unsafe { LLVMSetLinkage(func, LLVMLinkage::LLVMExternalLinkage) };
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

pub struct JitEngine {
    cxt: *mut LLVMContext,
    ee: *mut LLVMOpaqueExecutionEngine,
    pub module: *mut LLVMModule,
}

impl JitEngine {
    pub fn from_codegen_cxt(mut cxt: CodegenContext) -> Self {
        let ee = unsafe {
            LLVMLinkInMCJIT();
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();

            // Build an execution engine.
            {
                let mut ee = std::mem::MaybeUninit::uninit();
                let mut err = std::mem::zeroed();

                // This moves ownership of the module into the execution engine.
                if LLVMCreateExecutionEngineForModule(ee.as_mut_ptr(), cxt.module, &mut err) != 0 {
                    // In case of error, we must avoid using the uninitialized ExecutionEngineRef.
                    assert!(!err.is_null());
                    panic!(
                        "Failed to create execution engine: {:?}",
                        CStr::from_ptr(err)
                    );
                }

                ee.assume_init()
            }
        };

        /*
         * CodegenContext::module is now owned by ExecutionEngine => no need to drop it but CodegenContext should't drop it
         * Also we need to stole CodegenContext::cxt
         */
        let llvm_context = cxt.cxt;
        let llvm_module = cxt.module;
        cxt.cxt = null_mut();
        cxt.module = null_mut();

        Self {
            cxt: llvm_context,
            ee,
            module: llvm_module,
        }
    }

    // Safety: you have to ensure that T is correct function type
    pub fn get_func_addr(&self, name: &str) -> anyhow::Result<*const ()> {
        let c_name = CString::new(name).unwrap();

        let addr = unsafe { LLVMGetFunctionAddress(self.ee, c_name.as_ptr()) };
        let ptr: *const () = addr as *const ();

        if ptr.is_null() {
            anyhow::bail!("No '{}' function found", name)
        }

        Ok(ptr)
    }

    pub fn add_func_mapping(&self, name: &str, obj: *mut ()) -> anyhow::Result<()> {
        let func_name = CString::new(name).unwrap();
        let func = unsafe { LLVMGetNamedFunction(self.module, func_name.as_ptr() as *const _) };
        if func.is_null() {
            anyhow::bail!("Function {} wasn;t imported", name);
        }

        unsafe {
            LLVMAddGlobalMapping(self.ee, func, obj as *mut c_void);
        }

        Ok(())
    }
}

impl Drop for JitEngine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeExecutionEngine(self.ee);
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
            assert_eq!(res.value as usize, $exp);
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

            $vis.add_variable($name.into(), TypedValue{ value: pseudoval_ptr, ty: Rc::new(Type::Void()) });
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
