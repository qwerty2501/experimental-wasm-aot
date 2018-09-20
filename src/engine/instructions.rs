
use super::*;
use failure::Error;
use error::RuntimeError::*;
use parity_wasm::elements::{Instruction,ValueType,MemorySection,MemoryType};
use parity_wasm::elements::Module as WasmModule;

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

pub fn current_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;

        stack.values.push(current_frame.module_instance.linear_memory_compiler.build_get_memory_size(build_context,index as u32));
    }
    Ok(stack)

}

pub fn grow_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let grow_memory_function_name = current_frame.module_instance.linear_memory_compiler.get_grow_function_name(0);
        let grow_memory_function =  build_context.module().get_named_function(&grow_memory_function_name).ok_or(NoSuchLLVMFunction {name:grow_memory_function_name})?;
        let grow_memory_size = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push(build_context.builder().build_call(grow_memory_function,&[grow_memory_size],""));
    }
    Ok(stack)
}

pub fn add_int<'a,T:WasmIntType>(build_context:&'a BuildContext, mut stack:Stack<'a,T>) ->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_add(lhs,rhs,name))
}

pub fn add_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_fadd(lhs,rhs,name))
}

pub fn mul_int<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_mul(lhs,rhs,name))
}

pub fn mul_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_fmul(lhs,rhs,name))
}

pub fn sub_int<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_sub(lhs,rhs,name))
}

pub fn sub_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_fsub(lhs,rhs,name))
}

pub fn div_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_udiv(lhs,rhs,name))
}

pub fn div_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_sdiv(lhs,rhs,name))
}

pub fn div_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    calc(build_context,stack,|lhs,rhs,name|build_context.builder().build_fdiv(lhs,rhs,name))
}


fn calc<'a,T:WasmIntType,F:Fn(&'a Value,&'a Value,&'a str)->&'a Value>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,on_calc:F)->Result<Stack<'a,T>,Error>{
    {
        let rhs = stack.values.pop().ok_or(NotExistValue)?;
        let lhs = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push( on_calc(lhs,rhs,""));
    }
    Ok(stack)
}

pub fn get_global_name(index:u32) -> String {
    [WASM_GLOBAL_PREFIX,index.to_string().as_ref()].concat()
}

pub fn progress_instruction<'a,T:WasmIntType>(build_context:&'a BuildContext, instruction:Instruction,stack:Stack<'a,T>,wasm_module:&'a WasmModule)->Result<Stack<'a,T>,Error>{
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
        Instruction::GrowMemory(v)=>grow_memory(build_context,v,stack),
        Instruction::I32Add => add_int(build_context, stack),
        Instruction::I64Add => add_int(build_context, stack),
        Instruction::F32Add => add_float(build_context,stack),
        Instruction::F64Add => add_float(build_context,stack),
        Instruction::I32Mul => mul_int(build_context,stack),
        Instruction::I64Add => mul_int(build_context,stack),
        Instruction::F32Mul => mul_float(build_context,stack),
        Instruction::F64Mul => mul_float(build_context,stack),
        Instruction::I32Sub => sub_int(build_context,stack),
        Instruction::I64Sub => sub_int(build_context,stack),
        Instruction::F32Sub => sub_float(build_context,stack),
        Instruction::F64Sub => sub_float(build_context,stack),
        Instruction::I32DivS => div_sint(build_context,stack),
        Instruction::I32DivU => div_uint(build_context,stack),
        Instruction::I64DivS => div_sint(build_context,stack),
        Instruction::I64DivU => div_uint(build_context,stack),
        Instruction::F32Div => div_float(build_context,stack),
        Instruction::F64Div => div_float(build_context,stack),
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
    use parity_wasm::elements::Section;

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
        let (ft,lt) = new_compilers();
        let test_function_name = "test_function";
        build_test_instruction_function(&build_context, test_function_name, vec![Value::const_int(Type::int32(build_context.context()), expected, false)], vec![], |stack:Stack<u32>, bb|{
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
        let (ft,lt) = new_compilers();
        build_test_instruction_function(&build_context, test_function_name, vec![], vec![

            frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), expected as u64, false))],
                                              &[], &[], vec![],
                                              &ft,
                                              &lt)
        ], |stack,bb|{
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
        let (ft,lt) = new_compilers();
        build_test_instruction_function(&build_context, test_function_name, vec![Value::const_int(Type::int32(build_context.context()), expected, false)], vec![
            frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), 0, false))],
                                              &[], &[], vec![],
                                              &ft,&lt)], |stack,bb|{
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
        let (ft,lt) = new_compilers();
        build_test_instruction_function(&build_context, test_function_name, vec![Value::const_int(Type::int32(build_context.context()), expected, false)], vec![

            frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), 0, false))],
                                              &[], &[], vec![],
                                              &ft,
                                              &lt)
        ], |stack,bb|{
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
        build_test_instruction_function(&build_context, test_function_name, vec![Value::const_int(Type::int32(build_context.context()), expected, false)], vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
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
        lt.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(expected, None)])?;
        let test_function_name = "test_function";
        build_test_instruction_function(&build_context, test_function_name, vec![], vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
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

            let memory_size:u32 = *engine.get_global_value_ref_from_address(&lt.get_memory_size_name(0));
            assert_eq!(memory_size as u64,ret.to_int(false));

            Ok(())
        })
    }

    #[test]
    pub fn grow_memory_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("current_memory_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 18;
        let expected_ret = 17;
        lt.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(expected_ret, None)])?;
        let test_function_name = "test_function";
        build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),1,false)],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                               &ft,
                                                                               &lt)],|stack,bb|{

                let mut stack = grow_memory(&build_context,0,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;


        let  init_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),&init_memory_function_name,&[])?;
            assert_eq!(1,ret.to_int(false));


            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected_ret as u64,ret.to_int(false));

            let memory_size:u32 = *engine.get_global_value_ref_from_address(&lt.get_memory_size_name(0));
            assert_eq!(expected ,memory_size);
            Ok(())
        })
    }

    #[test]
    pub fn grow_memory_not_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("current_memory_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 17;
        let expected_ret = -1_i32;
        lt.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(expected, Some(20))])?;
        let test_function_name = "test_function";
        build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),4,false)],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                               &ft,
                                                                               &lt)],|stack,bb|{

                let mut stack = grow_memory(&build_context,0,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;


        let  init_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),&init_memory_function_name,&[])?;
            assert_eq!(1,ret.to_int(false));


            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected_ret ,ret.to_int(false) as i32);

            let memory_size:u32 = *engine.get_global_value_ref_from_address(&lt.get_memory_size_name(0));
            assert_eq!(expected ,memory_size);
            Ok(())
        })
    }

    #[test]
    pub fn add_i32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("addi32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 5;
        let test_function_name = "test_function";

        build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),2,false),Value::const_int(Type::int32(build_context.context()),3,false)],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                               &ft,
                                                                               &lt)],|stack,_|{

                let mut stack = add_int(&build_context, stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i32);
            Ok(())
        })
    }

    #[test]
    pub fn add_i64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("addi64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 5;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name, vec![Value::const_int(Type::int64(build_context.context()),2,false),Value::const_int(Type::int64(build_context.context()),3,false)],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                               &ft,
                                                                               &lt)],|stack,_|{

                let mut stack = add_int(&build_context, stack)?;
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
    pub fn add_f32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("add_f32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 5.5;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()),test_function_name,vec![Value::const_real(Type::float32(build_context.context()),2.25),Value::const_real(Type::float32(build_context.context()),3.25)],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                               &ft,
                                                                               &lt)],|stack,_|{

                let mut stack = add_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float32(build_context.context())));
            Ok(())
        })
    }

    #[test]
    pub fn add_f64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("add_f64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 5.5;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()),test_function_name,vec![Value::const_real(Type::float64(build_context.context()),2.25),Value::const_real(Type::float64(build_context.context()),3.25)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = add_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float64(build_context.context())));
            Ok(())
        })
    }

    #[test]
    pub fn mul_i32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("mul_i32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 6;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()),test_function_name,vec![Value::const_int(Type::int32(build_context.context()),2,false),Value::const_int(Type::int32(build_context.context()),3,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = mul_int(&build_context,stack)?;
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
    pub fn mul_i64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("mul_i64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 6;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),2,false),Value::const_int(Type::int64(build_context.context()),3,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = mul_int(&build_context,stack)?;
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
    pub fn mul_f32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("mul_f32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 7.0;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()),test_function_name,vec![Value::const_real(Type::float32(build_context.context()),2.0),Value::const_real(Type::float32(build_context.context()),3.5)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = mul_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float32(build_context.context())));
            Ok(())
        })
    }

    #[test]
    pub fn mul_f64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("mul_f64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 7.0;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()),test_function_name,vec![Value::const_real(Type::float64(build_context.context()),2.0),Value::const_real(Type::float64(build_context.context()),3.5)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = mul_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float64(build_context.context())));
            Ok(())
        })
    }

    #[test]
    pub fn sub_i32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("sub_i32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = -1_i32;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()),test_function_name,vec![Value::const_int(Type::int32(build_context.context()),2,false),Value::const_int(Type::int32(build_context.context()),3,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = sub_int(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i32);
            Ok(())
        })
    }

    #[test]
    pub fn sub_i64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("sub_i64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = -1_i64;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),2,false),Value::const_int(Type::int64(build_context.context()),3,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = sub_int(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i64);
            Ok(())
        })
    }

    #[test]
    pub fn sub_f32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("mul_f32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = -1.5_f32;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()),test_function_name,vec![Value::const_real(Type::float32(build_context.context()),2.0),Value::const_real(Type::float32(build_context.context()),3.5)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = sub_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float32(build_context.context())) as f32);
            Ok(())
        })
    }

    #[test]
    pub fn sub_f64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("sub_f64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = -1.5_f64;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()),test_function_name,vec![Value::const_real(Type::float64(build_context.context()),2.0),Value::const_real(Type::float64(build_context.context()),3.5)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = sub_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float64(build_context.context())));
            Ok(())
        })
    }

    #[test]
    pub fn div_u32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("div_u32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 2;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()),test_function_name,vec![Value::const_int(Type::int32(build_context.context()),4,false),Value::const_int(Type::int32(build_context.context()),2,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = div_uint(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i32);
            Ok(())
        })
    }

    #[test]
    pub fn div_u64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("div_u64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 2;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),4,false),Value::const_int(Type::int64(build_context.context()),2,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = div_uint(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i64);
            Ok(())
        })
    }

    #[test]
    pub fn div_s32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("div_s32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = -2_i32;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()),test_function_name,vec![Value::const_int(Type::int32(build_context.context()),-4_i32 as u64,false),Value::const_int(Type::int32(build_context.context()),2,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = div_sint(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i32);
            Ok(())
        })
    }

    #[test]
    pub fn div_s64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("div_s64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = -2_i64;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),-4_i64 as u64,false),Value::const_int(Type::int64(build_context.context()),2,false)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = div_sint(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as i64);
            Ok(())
        })
    }

    #[test]
    pub fn div_f32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("div_f32_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 2.0;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()),test_function_name,vec![Value::const_real(Type::float32(build_context.context()),7.0),Value::const_real(Type::float32(build_context.context()),3.5)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = div_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float32(build_context.context())) as f32);
            Ok(())
        })
    }

    #[test]
    pub fn div_f64_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("div_f64_works",&context);
        let (ft,lt) = new_compilers();

        let expected = 2.0;
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()),test_function_name,vec![Value::const_real(Type::float64(build_context.context()),7.0),Value::const_real(Type::float64(build_context.context()),3.5)],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = div_float(&build_context,stack)?;
                build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected,ret.to_float(Type::float64(build_context.context())));
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