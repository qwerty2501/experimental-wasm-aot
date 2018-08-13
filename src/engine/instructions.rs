
use super::*;
use failure::Error;
use error::RuntimeError::*;
use parity_wasm::elements::{Instruction};

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

pub fn progress_instruction<'a>(build_context:&'a BuildContext, instruction:Instruction,local_stack:&mut Vec<&'a Value>){
    match instruction{
        Instruction::I32Const(v)=> local_stack.push( i32_const(build_context,v)),
        Instruction::I64Const(v)=> local_stack.push(i64_const(build_context,v)),
        Instruction::F32Const(v)=> local_stack.push(f32_const(build_context,f32_reinterpret_i32(v))),
        Instruction::F64Const(v)=> local_stack.push(  f64_const(build_context, f64_reinterpret_i64(v))),

    }
}


#[inline]
pub fn i32_reinterpret_f32(v: f32) -> u32 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[inline]
pub fn i64_reinterpret_f64(v: f64) -> u64 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[inline]
pub fn f32_reinterpret_i32(v: u32) -> f32 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[inline]
pub fn f64_reinterpret_i64(v: u64) -> f64 {
    unsafe {
        ::std::mem::transmute(v)
    }
}