use std::{
    ffi::{CStr, CString, c_char},
    path::Path,
    ptr::null_mut,
    rc::Rc,
};

use llvm_sys::{
    LLVMOpcode, LLVMValue,
    core::{LLVMBuildCast, LLVMBuildFPCast, LLVMBuildIntCast, LLVMPrintModuleToFile},
};
use macros::c_str;

use crate::{ast::Statement, utils::nodes::Program};

mod context;
mod definitions;

pub use context::{CodegenContext, Value};
pub use definitions::Type;

pub struct TypedValue {
    pub value: *mut LLVMValue,
    pub ty: Rc<Type>,
}

pub fn compile(prog: &Program, output: &Path) -> anyhow::Result<()> {
    let mut cxt = CodegenContext::prepare(prog)?;
    prog.codegen(&mut cxt)?;

    let filename_c = CString::new(output.to_str().unwrap()).unwrap();
    let mut errors: *mut c_char = null_mut();
    unsafe {
        LLVMPrintModuleToFile(cxt.module, filename_c.as_ptr(), &mut errors as _);
    }

    if !errors.is_null() {
        // SAFETY: We trust in llvm
        let error = unsafe { CStr::from_ptr(errors) };
        anyhow::bail!(
            "Failed to dump LLVM IR with err {}",
            error.to_str().unwrap()
        );
    }

    Ok(())
}

pub const ZERO_NAME: *const i8 = c_str!(c"");

pub mod macros {
    macro_rules! c_str {
        ($str:literal) => {
            $str.as_ptr() as *const _
        };
    }

    pub(crate) use c_str;
}

pub fn cast(
    cxt: &mut CodegenContext,
    from: &Type,
    to: &Type,
    val: *mut LLVMValue,
) -> *mut LLVMValue {
    // Quickpath
    if from == to {
        return val;
    }

    let res = match to {
        Type::Float(to_fp) => match from {
            Type::Float(_) => unsafe {
                LLVMBuildFPCast(cxt.builder, val, to_fp.llvm_type(cxt), ZERO_NAME)
            },
            Type::Int(_) => unsafe {
                LLVMBuildCast(
                    cxt.builder,
                    LLVMOpcode::LLVMSIToFP,
                    val,
                    to_fp.llvm_type(cxt),
                    ZERO_NAME,
                )
            },
            _ => panic!("Cast from incompatable type"),
        },
        Type::Int(to_int) => match from {
            Type::Float(_) => unsafe {
                LLVMBuildCast(
                    cxt.builder,
                    LLVMOpcode::LLVMFPToSI,
                    val,
                    to_int.llvm_type(cxt),
                    ZERO_NAME,
                )
            },
            Type::Int(_) => unsafe {
                LLVMBuildIntCast(cxt.builder, val, to_int.llvm_type(cxt), ZERO_NAME)
            },
            _ => panic!("Cast from incompatable type"),
        },
        _ => panic!("Cast to incompatable type"),
    };

    assert!(!res.is_null());
    res
}

pub fn bool_from_llvm(cxt: &mut CodegenContext, val: *mut LLVMValue) -> TypedValue {
    TypedValue {
        value: val,
        ty: cxt.definitions.get_type("bool").unwrap(),
    }
}

pub fn bool_from_value(cxt: &mut CodegenContext, val: &TypedValue) -> TypedValue {
    let target_type = cxt.definitions.get_type("bool").unwrap();
    let value = cast(cxt, &val.ty, target_type.as_ref(), val.value);

    TypedValue {
        value,
        ty: target_type,
    }
}
