
use super::*;
use failure::Error;
use error::RuntimeError::*;
use parity_wasm::elements::{Instruction,ValueType};

const WASM_GLOBAL_PREFIX:&str = "__WASM_GLOBAL_";

pub fn i64_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:i64,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(i64_const_internal(build_context,v));
    Ok(stack)
}

pub fn i64_const_internal<'c>(build_context:&'c BuildContext, v:i64) ->&'c Value{
    Value::const_int(Type::int64(build_context.context()),v as u64,true)
}

pub fn i32_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:i32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(i32_const_internal(build_context,v));
    Ok(stack)
}

pub fn i32_const_internal<'c>(build_context:&'c BuildContext, v:i32) ->&'c Value{
    Value::const_int(Type::int64(build_context.context()),v as u64,true)
}

pub fn f64_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:f64,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(f64_const_internal(build_context,v));
    Ok(stack)
}

pub fn f64_const_internal<'c>(build_context:&'c BuildContext, v:f64) ->&'c Value{
    Value::const_real(Type::float64(build_context.context()),v)
}

pub fn f32_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:f32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(f32_const_internal(build_context,v));
    Ok(stack)
}

pub fn f32_const_internal<'c>(build_context:&'c BuildContext, v:f32) ->&'c Value{
    Value::const_real(Type::float32(build_context.context()),v as ::libc::c_double)
}

pub fn get_global_internal<'c>(build_context:&'c BuildContext, index:u32) ->Result< &'c Value,Error>{
    let name = get_global_name(index);
    Ok(build_context.module().get_named_global(name.as_ref()).ok_or_else(|| NoSuchLLVMGlobalValue {name})?)
}

pub fn get_global<'c,T:WasmIntType>(build_context:&'c BuildContext,index:u32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    let global_value = get_global_internal(build_context,index)?;
    stack.values.push(build_context.builder().build_load(global_value,""));
    Ok(stack)
}

pub fn set_global<'c,T:WasmIntType>(build_context:&'c BuildContext,index:u32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    let global_value = get_global_internal(build_context,index)?;
    build_context.builder().build_store( stack.values.pop().ok_or(NotExistValue)?,global_value);
    Ok(stack)
}

pub fn get_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current()?;
        stack.values.push( current_frame.locals.get(index as usize).ok_or(NoSuchLocalValue{index})?);
    }
    Ok(stack)
}

pub fn set_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  v = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        *v = stack.values.pop().ok_or(NotExistValue)?;
    }
    Ok(stack)
}

pub fn tee_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  v = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        *v = stack.values.last().ok_or(NotExistValue)?;

    }
    Ok(stack)
}

pub fn store<'a,T:WasmIntType>(build_context:&'a BuildContext,offset:u32,align:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let v = stack.values.pop().ok_or(NotExistValue)?;
        let memory = current_frame.module_instance.linear_memory_compiler.build_get_real_address(build_context,0,Value::const_int(Type::int32(build_context.context()),offset as u64,false),"");
        let value_type = match align{
            1 => Type::int8(build_context.context()),
            2 => Type::int16(build_context.context()),
            4 => Type::int32(build_context.context()),
            8 => Type::int64(build_context.context()),
            _=>Err(InCorrectAlign{align})?,
        };
        let v = build_context.builder().build_cast(Opcode::LLVMTrunc,v,value_type,"");
        let memory = build_context.builder().build_bit_cast(memory,Type::ptr(value_type,0),"");
        build_context.builder().build_store(v,memory);
        stack.values.push(v);
    }
    Ok(stack)

}

pub fn get_global_name(index:u32) -> String {
    [WASM_GLOBAL_PREFIX,index.to_string().as_ref()].concat()
}

pub fn progress_instruction<'a,T:WasmIntType>(build_context:&'a BuildContext, instruction:Instruction,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    match instruction{
        Instruction::I32Const(v)=> i32_const(build_context, v,stack),
        Instruction::I64Const(v)=> i64_const(build_context, v,stack),
        Instruction::F32Const(v)=> f32_const(build_context, f32_reinterpret_i32(v),stack),
        Instruction::F64Const(v)=> f64_const(build_context, f64_reinterpret_i64(v),stack),
        Instruction::GetGlobal(index)=> get_global(build_context,index,stack),
        Instruction::SetGlobal(index)=> set_global(build_context,index,stack),
        Instruction::GetLocal(index)=>get_local(build_context, index,stack),
        Instruction::SetLocal(index)=>set_local(build_context,index,stack),
        Instruction::TeeLocal(index)=>tee_local(build_context,index,stack),
        Instruction::F32Store(offset,align)=>store(build_context,offset,align,stack),
        Instruction::F64Store(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I32Store(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I64Store(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I32Store8(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I32Store16(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I64Store8(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I64Store16(offset,align)=>store(build_context,offset,align,stack),
        Instruction::I64Store32(offset,align)=>store(build_context,offset,align,stack),
        instruction=>Err(InvalidInstruction {instruction})?,
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

pub fn value_type_to_type<'a>(build_context:&'a BuildContext, value_type:&ValueType)->&'a Type{
    match value_type{
        ValueType::I32 => Type::int32(build_context.context()),
        ValueType::I64 => Type::int64(build_context.context()),
        ValueType::F32 => Type::float32(build_context.context()),
        ValueType::F64 => Type::float64(build_context.context()),
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    pub fn i32_const_internal_works(){
        let context = Context::new();
        let build_context = BuildContext::new("i32_const_works",&context);
        let expected:i32 = ::std::i32::MAX;
        let value= i32_const_internal(&build_context, expected);
        assert_eq!(expected,value.const_int_get_sign_extended_value() as i32);
    }

    #[test]
    pub fn i64_const_internal_works(){
        let context = Context::new();
        let build_context = BuildContext::new("i64_const_works",&context);
        let expected = ::std::i64::MAX;
        let value = i64_const_internal(&build_context, expected);
        assert_eq!(expected,value.const_int_get_sign_extended_value());
    }

    #[test]
    pub fn f32_const_internal_works(){
        let context = Context::new();
        let build_context = BuildContext::new("f32_const_works",&context);
        let expected = ::std::f32::MAX;
        let value = f32_const_internal(&build_context, expected);
        assert_eq!(expected,value.const_real_get_double().result as f32);
    }

    #[test]
    pub fn f64_const_internal_works(){
        let context = Context::new();
        let build_context = BuildContext::new("f64_const_works",&context);
        let expected = ::std::f64::MAX;
        let value = f64_const_internal(&build_context, expected);
        assert_eq!(expected,value.const_real_get_double().result);
    }

    #[test]
    pub fn globals_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("globals_works",&context);
        let global_name = get_global_name(0);
        let global_value = build_context.module().set_declare_global(&global_name,Type::int32(build_context.context()));
        global_value.set_initializer(Value::const_int(Type::int32(build_context.context()),0,false));
        let expected = 22;
        let test_function_name = "test_function";
        let test_function = build_context.module().set_declare_function(test_function_name,Type::function(Type::int32(build_context.context()),&[],false));
        build_context.builder().build_function(build_context.context(),test_function,|builder,bb|{
            let stack = Stack::<u32>::new(test_function,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![]);
            let stack = set_global(&build_context,0,stack)?;
            let stack = get_global(&build_context,0,stack)?;
            build_context.builder().build_ret(stack.values.last().ok_or(NotExistValue)?);
            Ok(())
        })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_int(false));
            Ok(())
        })
    }


    #[test]
    pub fn get_local_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("get_local_works",&context);
        let expected = 22;
        let test_function_name = "test_function";
        let test_function = build_context.module().set_declare_function(test_function_name,Type::function(Type::int32(build_context.context()),&[],false));
        build_context.builder().build_function(build_context.context(),test_function,|builder,bb| {
            let stack =  Stack::<u32>::new(test_function,vec![],vec![

                frame::test_utils::new_test_frame(vec![Value::const_int(Type::int32(build_context.context()), expected as u64, false)], vec![], vec![], vec![])
            ]);

            let mut stack = get_local(&build_context,0,stack)?;
            build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
            Ok(())
        })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn set_local_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("set_local_works",&context);
        let expected = 35;
        let test_function_name = "test_function";
        let test_function = build_context.module().set_declare_function(test_function_name,Type::function(Type::int32(build_context.context()),&[],false));
        build_context.builder().build_function(build_context.context(),test_function,|builder,bb| {
            let stack =  Stack::<u32>::new(test_function,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![

                frame::test_utils::new_test_frame(vec![Value::const_int(Type::int32(build_context.context()), 0, false)], vec![], vec![], vec![])
            ]);

            let stack = set_local(&build_context,0,stack)?;
            let mut stack = get_local(&build_context,0,stack)?;
            build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
            Ok(())
        })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_int(false));
            Ok(())
        })

    }

    #[test]
    pub fn tee_local_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("tee_local_works",&context);
        let expected = 35;
        let test_function_name = "test_function";
        let test_function = build_context.module().set_declare_function(test_function_name,Type::function(Type::int32(build_context.context()),&[],false));
        build_context.builder().build_function(build_context.context(),test_function,|builder,bb| {
            let stack =  Stack::<u32>::new(test_function,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![

                frame::test_utils::new_test_frame(vec![Value::const_int(Type::int32(build_context.context()), 0, false)], vec![], vec![], vec![])
            ]);

            let mut stack = tee_local(&build_context,0,stack)?;
            build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
            Ok(())
        })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_int(false));
            Ok(())
        })

    }

    #[test]
    pub fn store_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("load_and_store_works",&context);
        let expected = 3000;
        let test_function_name = "test_function";
        let test_function = build_context.module().set_declare_function(test_function_name,Type::function(Type::int32(build_context.context()),&[],false));
        build_context.builder().build_function(build_context.context(),test_function,|builder,bb| {
            let stack =  Stack::<u32>::new(test_function,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![

                frame::test_utils::new_test_frame(vec![], vec![], vec![], vec![])
            ]);

            let mut stack = store(&build_context,500,4,stack)?;
            build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
            Ok(())
        })?;

        let  int_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;
        let linear_memory_compiler = LinearMemoryCompiler::<u32>::new();
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),&int_memory_function_name,&[])?;
            assert_eq!(1,ret.to_int(false));
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_int(false));
            let memory_ptr:*mut u8 = *engine.get_global_value_ref_from_address(&linear_memory_compiler.get_memory_name(0));
            unsafe{
               assert_eq!(expected as u16,*( memory_ptr.add(500) as *mut u16));
            }
            Ok(())
        })
    }

    #[test]
    fn i32_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("i32_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::I32,Type::int32(build_context.context()));
    }

    #[test]
    fn i64_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("i64_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::I64,Type::int64(build_context.context()));
    }

    #[test]
    fn f32_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("f32_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::F32,Type::float32(build_context.context()));
    }

    #[test]
    fn f64_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("f64_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::F64,Type::float64(build_context.context()));
    }

    fn test_value_type_to_type(build_context:&BuildContext, value_type:&ValueType,expected:&Type){
        let actual = value_type_to_type(build_context,value_type);
        use llvm_sys::prelude::LLVMTypeRef;
        let expected_ptr:LLVMTypeRef = expected.into();
        let actual_ptr:LLVMTypeRef = expected.into();
        assert_eq!(  expected_ptr,actual_ptr.into());

    }

}