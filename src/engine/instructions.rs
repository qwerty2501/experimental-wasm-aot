
use super::*;
use failure::Error;
use error::RuntimeError::*;

const WASM_GLOBAL_PREFIX:&str = "__WASM_GLOBAL_";

pub fn i64_const<'c>(build_context:&'c BuildContext,v:i64)->&'c Value{
    Value::const_int(Type::int64(build_context.context()),v as u64,true)
}

pub fn i32_const<'c>(build_context:&'c BuildContext,v:i32)->&'c Value{
    Value::const_int(Type::int64(build_context.context()),v as u64,true)
}

pub fn f64_const<'c>(build_context:&'c BuildContext,v:f64)->&'c Value{
    Value::const_real(Type::float64(build_context.context()),v)
}

pub fn f32_const<'c>(build_context:&'c BuildContext,v:f32)->&'c Value{
    Value::const_real(Type::float32(build_context.context()),v as ::libc::c_double)
}

pub fn get_global_internal<'c>(build_context:&'c BuildContext, index:u32) ->Result< &'c Value,Error>{
    let name = get_global_name(index);
    Ok(build_context.module().get_named_global(name.as_ref()).ok_or_else(|| NoSuchLLVMGlobalValue {name})?)
}



pub fn get_global_name(index:u32) -> String {
    [WASM_GLOBAL_PREFIX,index.to_string().as_ref()].concat()
}