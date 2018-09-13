
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
    Value::const_int(Type::int32(build_context.context()),v as u64,true)
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
        stack.values.push( current_frame.locals.get(index as usize).ok_or(NoSuchLocalValue{index})?.value.ok_or(NoSuchLocalValue {index})?);
    }
    Ok(stack)
}

pub fn set_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  mut v = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        v.value = Some(stack.values.pop().ok_or(NotExistValue)?);
    }
    Ok(stack)
}

pub fn tee_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  mut v = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        v.value = Some(stack.values.last().ok_or(NotExistValue)?);

    }
    Ok(stack)
}

pub fn store<'a,T:WasmIntType>(build_context:&'a BuildContext,offset:u32,align:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {

        let current_frame = stack.activations.current_mut()?;
        build_check_memory_size_const(build_context, 0, offset, stack.current_function, current_frame.module_instance.linear_memory_compiler);
        let v = stack.values.pop().ok_or(NotExistValue)?;
        let memory = current_frame.module_instance.linear_memory_compiler.build_get_real_address(build_context,0,Value::const_int(Type::int32(build_context.context()),offset as u64,false),"");
        let value_type =get_value_type_from_align( build_context,align)?;
        let v = build_context.builder().build_cast(Opcode::LLVMTrunc,v,value_type,"");
        let memory = build_context.builder().build_bit_cast(memory,Type::ptr(value_type,0),"");
        build_context.builder().build_store(v,memory);
    }
    Ok(stack)

}

pub fn load<'a,T:WasmIntType>(build_context:&'a BuildContext,offset:u32,align:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        build_check_memory_size_const(build_context,0,offset,stack.current_function,current_frame.module_instance.linear_memory_compiler);
        let memory = current_frame.module_instance.linear_memory_compiler.build_get_real_address(build_context,0,Value::const_int(Type::int32(build_context.context()),offset as u64,false),"");
        let value_type = get_value_type_from_align(build_context,align)?;

        let memory = build_context.builder().build_bit_cast(memory,Type::ptr(value_type,0),"");

        let v = build_context.builder().build_load(memory,"");
        stack.values.push(v);
    }
    Ok(stack)
}

pub fn end<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    Ok(stack)
}

pub fn current_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,v:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;

        stack.values.push(current_frame.module_instance.linear_memory_compiler.build_get_memory_size(build_context,v as u32));
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
        Instruction::I32Load8S(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I32Load8U(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I32Load16S(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I32Load16U(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I32Load(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load8S(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load8U(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load16S(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load16U(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load32S(offset,align)=>load(build_context,offset,align,stack),
        Instruction::I64Load32U(offset,align)=>load(build_context,offset,align,stack),
        Instruction::CurrentMemory(v)=>current_memory(build_context,v,stack),
        Instruction::End=>end(build_context,stack),
        instruction=>Err(InvalidInstruction {instruction})?,
    }
}

#[inline]
fn get_value_type_from_align<'a>(build_context:&'a BuildContext, align:u32)->Result<&'a Type,Error>{
    Ok(match align{
        1 => Type::int8(build_context.context()),
        2 => Type::int16(build_context.context()),
        4 => Type::int32(build_context.context()),
        8 => Type::int64(build_context.context()),
        _=>Err(InCorrectAlign{align})?,
    })
}


#[inline]
fn build_check_memory_size_const<'a,T:WasmIntType>(build_context:&'a BuildContext, index:u32, target:u32, function:&'a Value, linear_memory_compiler:&LinearMemoryCompiler<T>){
    build_check_memory_size(build_context,index,Value::const_int(Type::int32(build_context.context()),target as ::libc::c_ulonglong,false),function,linear_memory_compiler);
}


#[inline]
fn build_check_memory_size<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32, target:&'a Value, function:&'a Value,linear_memory_compiler:&LinearMemoryCompiler<T>){
    let memory_size = linear_memory_compiler.build_get_real_memory_size(build_context, index);
    let cmp_ret = build_context.builder().build_icmp(IntPredicate::LLVMIntULT,target,memory_size,"");
    let then_bb = function.append_basic_block(build_context.context(),"");
    let else_bb = function.append_basic_block(build_context.context(),"");
    build_context.builder().build_cond_br(cmp_ret,then_bb,else_bb);
    build_context.builder().position_builder_at_end(else_bb);
    build_call_and_set_raise_const(build_context.module(),build_context.builder(),::libc::SIGSEGV);
    build_call_and_set_raise_const(build_context.module(),build_context.builder(),::libc::SIGSEGV); // for test on JIT. It need to send it twice why.
    build_context.builder().build_br(then_bb);
    build_context.builder().position_builder_at_end(then_bb);
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
    use parity_wasm::elements::ResizableLimits;

    fn new_compilers()->(FunctionTableCompiler<u32> ,LinearMemoryCompiler<u32>){
        ( FunctionTableCompiler::<u32>::new(),LinearMemoryCompiler::<u32>::new())
    }
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
            let (ft,lt) = new_compilers();
            let stack =  Stack::<u32>::new(test_function,vec![],vec![

                frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), expected as u64, false))],
                                                  &[], &[], vec![],
                                                    &ft,
                                                    &lt)
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
            let (ft,lt) = new_compilers();
            let stack =  Stack::<u32>::new(test_function,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![
                frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), 0, false))],
                                                  &[], &[], vec![],
                                                  &ft,
                                                  &lt)
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
            let (ft,lt) = new_compilers();
            let stack =  Stack::<u32>::new(test_function,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![

                frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), 0, false))],
                                                  &[], &[], vec![],
                                                  &ft,
                                                  &lt)
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
    pub fn store_and_load_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("load_and_store_works",&context);
        let expected = 3000;
        let (ft,lt) = new_compilers();
        let test_function_name = "test_function";
        build_test_run_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),expected,false)],vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                                 &ft,
                                                                                                 &lt)],
        |stack,bb|{
            let mut stack = store(&build_context,500,4,stack)?;
            let mut stack = load(&build_context,500,4,stack)?;
            build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
            Ok(())
        })?;
        let  init_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;

        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),&init_memory_function_name,&[])?;
            assert_eq!(1,ret.to_int(false));


            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_int(false));


            let memory_ptr:*mut u8 = *engine.get_global_value_ref_from_address(&lt.get_memory_name(0));
            unsafe{
               assert_eq!(expected as u16,*( memory_ptr.add(500) as *mut u16));
            }

            Ok(())
        })
    }

    #[test]
    pub fn current_memory_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("current_memory_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 17;
        lt.build_init_function(&build_context,0,&[&ResizableLimits::new(expected,None)])?;
        let test_function_name = "test_function";
        build_test_run_function(&build_context,test_function_name,vec![],vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                                                &ft,
                                                                                                                &lt)],
        |stack,bb|{
            let mut stack = current_memory(&build_context,0,stack)?;
            build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
            Ok(())
        })?;

        let  init_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;

        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),&init_memory_function_name,&[])?;
            assert_eq!(1,ret.to_int(false));


            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected as u64,ret.to_int(false));

            Ok(())
        })
    }


    #[test]
    pub fn i32_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("i32_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::I32,Type::int32(build_context.context()));
    }

    #[test]
    pub fn i64_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("i64_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::I64,Type::int64(build_context.context()));
    }

    #[test]
    pub fn f32_value_type_to_type_works(){
        let context = Context::new();
        let build_context = BuildContext::new("f32_value_type_to_type",&context);
        test_value_type_to_type(&build_context,&ValueType::F32,Type::float32(build_context.context()));
    }

    #[test]
    pub fn f64_value_type_to_type_works(){
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