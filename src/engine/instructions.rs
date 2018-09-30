
use super::*;
use failure::Error;
use error::RuntimeError::*;
use parity_wasm::elements::{Instruction,ValueType,MemorySection,MemoryType};
use parity_wasm::elements::Module as WasmModule;

const WASM_GLOBAL_PREFIX:&str = "__WASM_GLOBAL_";

 fn i64_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:i64,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(i64_const_internal(build_context,v));
    Ok(stack)
}

pub fn i64_const_internal<'c>(build_context:&'c BuildContext, v:i64) ->&'c Value{
    Value::const_int(Type::int64(build_context.context()),v as u64,true)
}

 fn i32_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:i32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(i32_const_internal(build_context,v));
    Ok(stack)
}

pub fn i32_const_internal<'c>(build_context:&'c BuildContext, v:i32) ->&'c Value{
    Value::const_int(Type::int32(build_context.context()),v as u64,true)
}

 fn f64_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:f64,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(f64_const_internal(build_context,v));
    Ok(stack)
}

pub fn f64_const_internal<'c>(build_context:&'c BuildContext, v:f64) ->&'c Value{
    Value::const_real(Type::float64(build_context.context()),v)
}

 fn f32_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:f32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
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

 fn get_global<'c,T:WasmIntType>(build_context:&'c BuildContext,index:u32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    let global_value = get_global_internal(build_context,index)?;
    stack.values.push(build_context.builder().build_load(global_value,""));
    Ok(stack)
}

 fn set_global<'c,T:WasmIntType>(build_context:&'c BuildContext,index:u32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    let global_value = get_global_internal(build_context,index)?;
    build_context.builder().build_store( stack.values.pop().ok_or(NotExistValue)?,global_value);
    Ok(stack)
}

 fn get_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current()?;
        stack.values.push( current_frame.locals.get(index as usize).ok_or(NoSuchLocalValue{index})?.value.ok_or(NoSuchLocalValue {index})?);
    }
    Ok(stack)
}

 fn set_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  mut v = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        v.value = Some(stack.values.pop().ok_or(NotExistValue)?);
    }
    Ok(stack)
}

 fn tee_local<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  mut v = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        v.value = Some(stack.values.last().ok_or(NotExistValue)?);

    }
    Ok(stack)
}

 fn store<'a,T:WasmIntType>(build_context:&'a BuildContext,offset:u32,align:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
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

 fn load<'a,T:WasmIntType>(build_context:&'a BuildContext,offset:u32,align:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
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

 fn end<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    Ok(stack)
}

 fn current_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;

        stack.values.push(current_frame.module_instance.linear_memory_compiler.build_get_memory_size(build_context,index as u32));
    }
    Ok(stack)

}

 fn grow_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let grow_memory_function_name = current_frame.module_instance.linear_memory_compiler.get_grow_function_name(0);
        let grow_memory_function =  build_context.module().get_named_function(&grow_memory_function_name).ok_or(NoSuchLLVMFunction {name:grow_memory_function_name})?;
        let grow_memory_size = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push(build_context.builder().build_call(grow_memory_function,&[grow_memory_size],""));
    }
    Ok(stack)
}

fn clz_int32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack, |x| build_call_and_set_ctlz_i32(build_context.module(),build_context.builder(),x,""))
}

fn ctz_int32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_cttz_i32(build_context.module(),build_context.builder(),x,""))
}

fn clz_int64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_ctlz_i64(build_context.module(),build_context.builder(),x,""))
}

fn ctz_int64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_cttz_i64(build_context.module(),build_context.builder(),x,""))
}

fn popcnt_int32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_ctpop_i32(build_context.module(),build_context.builder(),x,""))
}

fn popcnt_int64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_ctpop_i64(build_context.module(),build_context.builder(),x,""))
}

fn abs_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_fabs_f32(build_context.module(),build_context.builder(),x,""))
}

fn abs_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_fabs_f64(build_context.module(),build_context.builder(),x,""))
}

fn neg_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_context.builder().build_fneg(x,""))
}

fn neg_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_context.builder().build_fneg(x,""))
}


fn sqrt_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_sqrt_f32(build_context.module(),build_context.builder(),x,""))
}

fn sqrt_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_sqrt_f64(build_context.module(),build_context.builder(),x,""))
}

fn ceil_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_ceil_f32(build_context.module(),build_context.builder(),x,""))
}

fn ceil_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_ceil_f64(build_context.module(),build_context.builder(),x,""))
}


fn floor_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_floor_f32(build_context.module(),build_context.builder(),x,""))
}

fn floor_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_floor_f64(build_context.module(),build_context.builder(),x,""))
}

fn trunc_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_trunc_f32(build_context.module(),build_context.builder(),x,""))
}

fn trunc_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_trunc_f64(build_context.module(),build_context.builder(),x,""))
}

fn nearest_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_nearbyint_f32(build_context.module(),build_context.builder(),x,""))
}

fn nearest_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    unop(build_context,stack,|x| build_call_and_set_nearbyint_f64(build_context.module(),build_context.builder(),x,""))
}


fn unop<'a,T:WasmIntType,F:Fn(&'a Value)->&'a Value>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,on_unop:F)->Result<Stack<'a,T>,Error>{
    {
        let x = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push(on_unop(x))
    }
    Ok(stack)
}

 fn add_int<'a,T:WasmIntType>(build_context:&'a BuildContext, mut stack:Stack<'a,T>) ->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_add(lhs, rhs, name))
}

 fn add_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fadd(lhs, rhs, name))
}

 fn mul_int<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_mul(lhs, rhs, name))
}

 fn mul_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fmul(lhs, rhs, name))
}

 fn sub_int<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_sub(lhs, rhs, name))
}

 fn sub_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fsub(lhs, rhs, name))
}

 fn div_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_udiv(lhs, rhs, name))
}

 fn div_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_sdiv(lhs, rhs, name))
}

 fn div_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fdiv(lhs, rhs, name))
}

fn min_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fminf(build_context.module(),build_context.builder(),lhs,rhs,name))
}

fn min_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fmin(build_context.module(), build_context.builder(), lhs, rhs, name))
}

fn max_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fmaxf(build_context.module(),build_context.builder(),lhs,rhs,name))
}

fn max_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fmax(build_context.module(), build_context.builder(), lhs, rhs, name))
}

fn copysign_float32<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_copysignf(build_context.module(),build_context.builder(),lhs,rhs,name))
}

fn copysign_float64<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_copysign(build_context.module(), build_context.builder(), lhs, rhs, name))
}

fn rem_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_srem(lhs,rhs,name))
}

fn rem_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_urem(lhs,rhs,name))
}

fn and<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_and(lhs,rhs,name))
}

fn or<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_or(lhs,rhs,name))
}

fn xor<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_xor(lhs,rhs,name))
}

fn shl<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_shl(lhs,rhs,name))
}

fn shr_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_ashr(lhs,rhs,name))
}

fn shr_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_lshr(lhs,rhs,name))
}

fn rotl<'a,T:WasmIntType,W:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|{
        let bw = bit_width::<W>();
        let size_type = Type::int(build_context.context(),bw as u32);
        let mask = Value::const_int(size_type,(bw-1)  as u64,false);

        build_context.builder().build_or(
            build_context.builder().build_shl(lhs,rhs,""),
            build_context.builder().build_lshr(lhs,
                                               build_context.builder().build_and(
                                                   build_context.builder().build_sub(
                                                       Value::const_int(size_type,0,false),
                                                       rhs,""),
                                                   mask,"")
                                               ,"" ),
            name)
    })
}

fn rotr<'a,T:WasmIntType,W:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|{
        let bw = bit_width::<W>();
        let size_type = Type::int(build_context.context(),bw as u32);
        let mask = Value::const_int(size_type,(bw-1)  as u64,false);
        build_context.builder().build_or(
            build_context.builder().build_lshr(lhs,rhs,""),
            build_context.builder().build_shl(lhs,
                                               build_context.builder().build_and(
                                                   build_context.builder().build_sub(
                                                       Value::const_int(size_type,0,false),
                                                       rhs,""),
                                                   mask,""),
                                              "" ),
        name)
    })
}

fn binop<'a,T:WasmIntType,F:Fn(&'a Value,&'a Value,&'a str)->&'a Value>(build_context:&'a BuildContext, mut stack:Stack<'a,T>, on_binop:F) ->Result<Stack<'a,T>,Error>{
    {
        let rhs = stack.values.pop().ok_or(NotExistValue)?;
        let lhs = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push( on_binop(lhs,rhs,""));
    }
    Ok(stack)
}


 fn  eqz32<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error> {
    eqz(build_context,Type::int32(build_context.context()),stack)
}

 fn eqz64<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    eqz(build_context,Type::int64(build_context.context()),stack)
}
fn eqz<'a,T:WasmIntType>(build_context:&'a BuildContext,type_ref:&'a Type,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    let i = stack.values.pop().ok_or(NotExistValue)?;
    stack.values.push(build_context.builder().build_icmp(IntPredicate::LLVMIntEQ,i,Value::const_int(type_ref,0,false),""));
    Ok(stack)
}

 fn eq_int<'a,T:WasmIntType>(build_context:&'a BuildContext, mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntEQ,lhs,rhs,name))
}

 fn eq_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOEQ,lhs,rhs,name))
}

 fn ne_int<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntNE,lhs,rhs,name))
}

fn ne_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealONE,lhs,rhs,name))
}

fn lt_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSLT,lhs,rhs,name))
}

fn lt_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntULT,lhs,rhs,name))
}

fn lt_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOLT,lhs,rhs,name))
}


fn le_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSLE,lhs,rhs,name))
}

fn le_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntULE,lhs,rhs,name))
}

fn le_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOLE,lhs,rhs,name))
}


fn gt_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSGT,lhs,rhs,name))
}

fn gt_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntUGT,lhs,rhs,name))
}

fn gt_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOGT,lhs,rhs,name))
}


fn ge_sint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSGE,lhs,rhs,name))
}

fn ge_uint<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntUGE,lhs,rhs,name))
}

fn ge_float<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOGE,lhs,rhs,name))
}



fn relop<'a,T:WasmIntType,F:Fn(&'a Value,&'a Value,&'a str)->&'a Value>(build_context:&'a BuildContext, mut stack:Stack<'a,T>, on_relop:F) ->Result<Stack<'a,T>,Error>{
    {
        let rhs = stack.values.pop().ok_or(NotExistValue)?;
        let lhs = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push(build_context.builder().build_zext_or_bit_cast(on_relop(lhs,rhs,""),Type::int32(build_context.context()),""));
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
        Instruction::GrowMemory(v)=>grow_memory(build_context,v,stack),

        Instruction::I32Clz => clz_int32(build_context,stack),
        Instruction::I32Ctz => ctz_int32(build_context,stack),
        Instruction::I64Clz => clz_int64(build_context,stack),
        Instruction::I64Ctz => ctz_int64(build_context,stack),

        Instruction::I32Popcnt => popcnt_int32(build_context,stack),
        Instruction::I64Popcnt => popcnt_int64(build_context,stack),

        Instruction::F32Abs => abs_float32(build_context,stack),
        Instruction::F64Abs => abs_float64(build_context,stack),

        Instruction::F32Neg => neg_float32(build_context,stack),
        Instruction::F64Neg => neg_float64(build_context,stack),

        Instruction::F32Sqrt => sqrt_float32(build_context,stack),
        Instruction::F64Sqrt => sqrt_float64(build_context,stack),

        Instruction::F32Ceil => ceil_float32(build_context,stack),
        Instruction::F64Ceil => ceil_float64(build_context,stack),

        Instruction::F32Floor => floor_float32(build_context,stack),
        Instruction::F64Floor => floor_float64(build_context,stack),

        Instruction::F32Trunc => trunc_float32(build_context,stack),
        Instruction::F64Trunc => trunc_float64(build_context,stack),

        Instruction::F32Nearest => nearest_float32(build_context,stack),
        Instruction::F64Nearest => nearest_float64(build_context,stack),

        Instruction::I32Add => add_int(build_context, stack),
        Instruction::I64Add => add_int(build_context, stack),
        Instruction::F32Add => add_float(build_context,stack),
        Instruction::F64Add => add_float(build_context,stack),

        Instruction::I32Mul => mul_int(build_context,stack),
        Instruction::I64Mul => mul_int(build_context,stack),
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

        Instruction::I32RemS => rem_sint(build_context,stack),
        Instruction::I32RemU => rem_uint(build_context,stack),
        Instruction::I64RemS => rem_sint(build_context,stack),
        Instruction::I64RemU => rem_uint(build_context,stack),

        Instruction::I32And => and(build_context,stack),
        Instruction::I64And => and(build_context,stack),

        Instruction::I32Or => and(build_context,stack),
        Instruction::I64Or => and(build_context,stack),

        Instruction::I32Xor => xor(build_context,stack),
        Instruction::I64Xor => xor(build_context,stack),

        Instruction::I32Shl => shl(build_context,stack),
        Instruction::I64Shl => shl(build_context,stack),

        Instruction::I32ShrS => shr_sint(build_context,stack),
        Instruction::I32ShrU => shr_uint(build_context,stack),
        Instruction::I64ShrS => shr_sint(build_context,stack),
        Instruction::I64ShrU => shr_uint(build_context,stack),

        Instruction::I32Rotl => rotl::<T,u32>(build_context,stack),
        Instruction::I32Rotr => rotr::<T,u32>(build_context,stack),
        Instruction::I64Rotl => rotl::<T,u64>(build_context,stack),
        Instruction::I64Rotr => rotr::<T,u64>(build_context,stack),

        Instruction::F32Min => min_float32(build_context,stack),
        Instruction::F64Min => min_float64(build_context,stack),
        Instruction::F32Max => max_float32(build_context,stack),
        Instruction::F64Max => max_float64(build_context,stack),

        Instruction::F32Copysign => copysign_float32(build_context,stack),
        Instruction::F64Copysign => copysign_float64(build_context,stack),

        Instruction::I32Eqz => eqz32(build_context,stack),
        Instruction::I64Eqz => eqz64(build_context,stack),
        Instruction::I32Eq => eq_int(build_context,stack),
        Instruction::I64Eq => eq_int(build_context,stack),
        Instruction::F32Eq => eq_float(build_context,stack),
        Instruction::F64Eq => eq_float(build_context,stack),

        Instruction::I32Ne => ne_int(build_context,stack),
        Instruction::I64Ne => ne_int(build_context,stack),
        Instruction::F32Ne => ne_float(build_context,stack),
        Instruction::F64Ne => ne_float(build_context,stack),


        Instruction::I32LtS => lt_sint(build_context,stack),
        Instruction::I32LtU => lt_uint(build_context,stack),
        Instruction::I64LtS => lt_sint(build_context,stack),
        Instruction::I64LtU => lt_uint(build_context,stack),
        Instruction::F32Lt => lt_float(build_context,stack),
        Instruction::F64Lt => lt_float(build_context,stack),


        Instruction::I32GtS => gt_sint(build_context,stack),
        Instruction::I32GtU => gt_uint(build_context,stack),
        Instruction::I64GtS => gt_sint(build_context,stack),
        Instruction::I64GtU => gt_uint(build_context,stack),
        Instruction::F32Gt => gt_float(build_context,stack),
        Instruction::F64Gt => gt_float(build_context,stack),

        Instruction::I32LeS => le_sint(build_context,stack),
        Instruction::I32LeU => le_uint(build_context,stack),
        Instruction::I64LeS => le_sint(build_context,stack),
        Instruction::I64LeU => le_uint(build_context,stack),
        Instruction::F32Le => le_float(build_context,stack),
        Instruction::F64Le => le_float(build_context,stack),

        Instruction::I32GeS => ge_sint(build_context,stack),
        Instruction::I32GeU => ge_uint(build_context,stack),
        Instruction::I64GeS => ge_sint(build_context,stack),
        Instruction::I64GeU => ge_uint(build_context,stack),
        Instruction::F32Ge => ge_float(build_context,stack),
        Instruction::F64Ge => ge_float(build_context,stack),


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


    macro_rules! unop_u32_works {
        ($expected:expr,$x:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("unop_u32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$x as u64,false)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;
                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! unop_s32_works {
        ($expected:expr,$x:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("unop_s32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$x as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;
                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(true) as i32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! unop_u64_works {
        ($expected:expr,$x:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("unop_u64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![Value::const_int(Type::int64(build_context.context()),$x as u64,false)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;
                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u64);
                    Ok(())
                })
            }
        )
    }


    macro_rules! unop_s64_works {
        ($expected:expr,$x:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("unop_s64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![Value::const_int(Type::int64(build_context.context()),$x as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;
                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(true) as u64);
                    Ok(())
                })
            }
        )
    }

    macro_rules! unop_f32_works {
        ($expected:expr,$x:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("unop_f32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![Value::const_real(Type::float32(build_context.context()),$x as f64)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_float(Type::float32(build_context.context())) as f32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! unop_f64_works {
        ($expected:expr,$x:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("unop_f64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![Value::const_real(Type::float64(build_context.context()),$x as f64)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;
                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_float(Type::float64(build_context.context())) as f64);
                    Ok(())
                })
            }
        )
    }


    macro_rules! binop_u32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_u32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$lhs as u64,false),Value::const_int(Type::int32(build_context.context()),$rhs as u64,false)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! binop_u64_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_u64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),$lhs as u64,false),Value::const_int(Type::int64(build_context.context()),$rhs as u64,false)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false));
                    Ok(())
                })
            }
        )
    }



    macro_rules! binop_s32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_s32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$lhs as u64,true),Value::const_int(Type::int32(build_context.context()),$rhs as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(true) as i32);
                    Ok(())
                })
            }

        )
    }



    macro_rules! binop_s64_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_s64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),$lhs as u64,true),Value::const_int(Type::int64(build_context.context()),$rhs as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(true) as i64);
                    Ok(())
                })
            }
        )
    }


    macro_rules! binop_f32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_f32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()),test_function_name,vec![Value::const_real(Type::float32(build_context.context()),$lhs as f64),Value::const_real(Type::float32(build_context.context()),$rhs as f64)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_float(Type::float32(build_context.context())) as f32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! binop_f64_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_f32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()),test_function_name,vec![Value::const_real(Type::float64(build_context.context()),$lhs as f64),Value::const_real(Type::float64(build_context.context()),$rhs as f64)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_float(Type::float64(build_context.context())));
                    Ok(())
                })
            }
        )
    }


    macro_rules! binop_s32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("binop_s32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$lhs as u64,true),Value::const_int(Type::int32(build_context.context()),$rhs as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(true) as i32);
                    Ok(())
                })
            }

        )
    }

    macro_rules! relop_u32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("relop_u32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$lhs as u64,false),Value::const_int(Type::int32(build_context.context()),$rhs as u64,false)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;
                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! relop_s32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("relop_s32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int32(build_context.context()),$lhs as u64,true),Value::const_int(Type::int32(build_context.context()),$rhs as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! relop_u64_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("relop_u64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int64(build_context.context()),$lhs as u64,false),Value::const_int(Type::int64(build_context.context()),$rhs as u64,false)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! relop_s64_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("relop_s64_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_int(Type::int64(build_context.context()),$lhs as u64,true),Value::const_int(Type::int64(build_context.context()),$rhs as u64,true)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }


    macro_rules! relop_f32_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("relop_f32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_real(Type::float32(build_context.context()),$lhs as f64),Value::const_real(Type::float32(build_context.context()),$rhs as f64)],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    macro_rules! relop_f64_works {
        ($expected:expr,$lhs:expr,$rhs:expr,$instruction:expr) => (
            {
                let context = Context::new();
                let build_context = BuildContext::new("relop_f32_works",&context);
                let (ft,lt) = new_compilers();
                let test_function_name = "test_function";

                build_test_instruction_function(&build_context,test_function_name,vec![Value::const_real(Type::float64(build_context.context()),$lhs ),Value::const_real(Type::float64(build_context.context()),$rhs )],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let mut stack = progress_instruction(&build_context,$instruction, stack)?;
                        build_context.builder().build_ret(stack.values.pop().ok_or(NotExistValue)?);
                        Ok(())
                    })?;

                test_module_in_engine(build_context.module(),|engine|{

                    let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
                    assert_eq!($expected ,ret.to_int(false) as u32);
                    Ok(())
                })
            }
        )
    }

    #[test]
    pub fn clz_i32_works()->Result<(),Error>{
        unop_u32_works!(30,2,Instruction::I32Clz)
    }

    #[test]
    pub fn ctz_i32_works()->Result<(),Error>{
        unop_u32_works!(1,2,Instruction::I32Ctz)
    }

    #[test]
    pub fn clz_i64_works()->Result<(),Error>{
        unop_u64_works!(62,2,Instruction::I64Clz)
    }

    #[test]
    pub fn ctz_i64_works()->Result<(),Error>{
        unop_u64_works!(1,2,Instruction::I64Ctz)
    }

    #[test]
    pub fn popcnt_i32_works()->Result<(),Error>{
        unop_u32_works!(16,0xFF_00_FF_00,Instruction::I32Popcnt)
    }

    #[test]
    pub fn popcnt_i64_works()->Result<(),Error>{
        unop_u64_works!(32,0xFF_00_FF_00_FF_00_FF_00,Instruction::I64Popcnt)
    }


    #[test]
    pub fn abs_f32_negative_works()->Result<(),Error>{
        unop_f32_works!(3.5,-3.5_f32,Instruction::F32Abs)
    }

    #[test]
    pub fn abs_f32_positive_works()->Result<(),Error>{
        unop_f32_works!(3.5,3.5_f32,Instruction::F32Abs)
    }

    #[test]
    pub fn abs_f64_negative_works()->Result<(),Error>{
        unop_f64_works!(4.402823e+38,-4.402823e+38_f64,Instruction::F64Abs)
    }

    #[test]
    pub fn abs_f64_positive_works()->Result<(),Error>{
        unop_f64_works!(4.402823e+38,4.402823e+38,Instruction::F64Abs)
    }

    #[test]
    pub fn neg_f32_works()->Result<(),Error>{
        unop_f32_works!(-3.5_f32,3.5,Instruction::F32Neg)
    }

    #[test]
    pub fn neg_f64_works()->Result<(),Error>{
        unop_f64_works!(-4.402823e+38_f64,4.402823e+38,Instruction::F64Neg)
    }

    #[test]
    pub fn sqrt_f32_works()->Result<(),Error>{
        unop_f32_works!(1.4142135,2.0,Instruction::F32Sqrt)
    }

    #[test]
    pub fn sqrt_f64_works()->Result<(),Error>{
        unop_f64_works!(1.4142135623730951,2.0,Instruction::F64Sqrt)
    }


    #[test]
    pub fn ceil_f32_works()->Result<(),Error>{
        unop_f32_works!(3.0,2.1,Instruction::F32Ceil)
    }

    #[test]
    pub fn ceil_f64_works()->Result<(),Error>{
        unop_f64_works!(3.0,2.1,Instruction::F64Ceil)
    }


    #[test]
    pub fn floor_f32_works()->Result<(),Error>{
        unop_f32_works!(2.0,2.9,Instruction::F32Floor)
    }

    #[test]
    pub fn floor_f64_works()->Result<(),Error>{
        unop_f64_works!(2.0,2.9,Instruction::F64Floor)
    }

    #[test]
    pub fn floor_f32_negative_works()->Result<(),Error>{
        unop_f32_works!(-1.0_f32,-0.9_f32,Instruction::F32Floor)
    }

    #[test]
    pub fn floor_f64_negative_works()->Result<(),Error>{
        unop_f64_works!(-1.0_f64,-0.9_f64,Instruction::F64Floor)
    }

    #[test]
    pub fn trunc_f32_works()->Result<(),Error>{
        unop_f32_works!(2.0,2.9,Instruction::F32Trunc)
    }

    #[test]
    pub fn trunc_f64_works()->Result<(),Error>{
        unop_f64_works!(2.0,2.9,Instruction::F64Trunc)
    }

    #[test]
    pub fn trunc_f32_negative_works()->Result<(),Error>{
        unop_f32_works!(-2.0_f32,-2.9_f32,Instruction::F32Trunc)
    }

    #[test]
    pub fn trunc_f64_negative_works()->Result<(),Error>{
        unop_f64_works!(-2.0_f64,-2.9_f64,Instruction::F64Trunc)
    }

    #[test]
    pub fn nearest_f32_works()->Result<(),Error>{
        unop_f32_works!(2.0_f32,2.5_f32,Instruction::F32Nearest)
    }

    #[test]
    pub fn nearest_f64_works()->Result<(),Error>{
        unop_f64_works!(2.0_f64,2.5_f64,Instruction::F64Nearest)
    }

    #[test]
    pub fn nearest_f32_odd_works()->Result<(),Error>{
        unop_f32_works!(4.0_f32,3.5_f32,Instruction::F32Nearest)
    }

    #[test]
    pub fn nearest_f64_odd_works()->Result<(),Error>{
        unop_f64_works!(4.0_f64,3.5_f64,Instruction::F64Nearest)
    }









    #[test]
    pub fn add_i32_works()->Result<(),Error>{
        binop_u32_works!(5,2,3,Instruction::I32Add)
    }

    #[test]
    pub fn add_i64_works()->Result<(),Error>{
        binop_u64_works!(5,2,3,Instruction::I64Add)
    }

    #[test]
    pub fn add_f32_works()->Result<(),Error>{
        binop_f32_works!(5.5,2.25,3.25,Instruction::F32Add)
    }

    #[test]
    pub fn add_f64_works()->Result<(),Error>{
        binop_f64_works!(5.5,2.25,3.25,Instruction::F64Add)
    }

    #[test]
    pub fn mul_i32_works()->Result<(),Error>{
        binop_u32_works!(6,2,3,Instruction::I32Mul)
    }

    #[test]
    pub fn mul_i64_works()->Result<(),Error>{
        binop_u64_works!(6,2,3,Instruction::I64Mul)
    }

    #[test]
    pub fn mul_f32_works()->Result<(),Error>{
        binop_f32_works!(7.0,2.0,3.5,Instruction::F32Mul)
    }

    #[test]
    pub fn mul_f64_works()->Result<(),Error>{
        binop_f64_works!(7.0,2.0,3.5,Instruction::F64Mul)
    }

    #[test]
    pub fn sub_i32_works()->Result<(),Error>{
        binop_s32_works!(-1_i32,2,3,Instruction::I32Sub)
    }

    #[test]
    pub fn sub_i64_works()->Result<(),Error>{
        binop_s64_works!(-1_i64,2,3,Instruction::I64Sub)
    }

    #[test]
    pub fn sub_f32_works()->Result<(),Error>{
        binop_f32_works!(-1.5_f32,2.0,3.5,Instruction::F32Sub)
    }

    #[test]
    pub fn sub_f64_works()->Result<(),Error>{
        binop_f64_works!(-1.5_f64,2.0,3.5,Instruction::F64Sub)
    }

    #[test]
    pub fn div_u32_works()->Result<(),Error>{
        binop_u32_works!(2,6,3,Instruction::I32DivU)
    }

    #[test]
    pub fn div_u64_works()->Result<(),Error>{
        binop_u64_works!(2,4,2,Instruction::I64DivU)
    }

    #[test]
    pub fn div_s32_works()->Result<(),Error>{
        binop_s32_works!(-2_i32,-4_i32,2,Instruction::I32DivS)
    }

    #[test]
    pub fn div_s64_works()->Result<(),Error>{
        binop_s64_works!(-2_i64,-4_i64,2,Instruction::I64DivS)
    }

    #[test]
    pub fn div_f32_works()->Result<(),Error>{
        binop_f32_works!(2.0,7.0,3.5,Instruction::F32Div)
    }

    #[test]
    pub fn div_f64_works()->Result<(),Error>{
        binop_f64_works!(2.0,7.0,3.5,Instruction::F64Div)
    }


    #[test]
    pub fn min_float32_right_works()->Result<(),Error>{
        binop_f32_works!(2.0,3.0,2.0,Instruction::F32Min)
    }

    #[test]
    pub fn min_float32_left_works()->Result<(),Error>{
        binop_f32_works!(2.0,2.0,3.0,Instruction::F32Min)
    }

    #[test]
    pub fn min_float64_right_works()->Result<(),Error>{
        binop_f64_works!(2.0,3.0,2.0,Instruction::F64Min)
    }

    #[test]
    pub fn min_float64_left_works()->Result<(),Error>{
        binop_f64_works!(2.0,2.0,3.0,Instruction::F64Min)
    }


    #[test]
    pub fn max_float32_right_works()->Result<(),Error>{
        binop_f32_works!(3.0,2.0,3.0,Instruction::F32Max)
    }

    #[test]
    pub fn max_float32_left_works()->Result<(),Error>{
        binop_f32_works!(3.0,3.0,2.0,Instruction::F32Max)
    }

    #[test]
    pub fn max_float64_right_works()->Result<(),Error>{
        binop_f64_works!(3.0,2.0,3.0,Instruction::F64Max)
    }

    #[test]
    pub fn max_float64_left_works()->Result<(),Error>{
        binop_f64_works!(3.0,3.0,2.0,Instruction::F64Max)
    }


    #[test]
    pub fn copysign_float32_copy_works()->Result<(),Error>{
        binop_f32_works!(-2.0_f32,2.0,-1.0_f32,Instruction::F32Copysign)
    }

    #[test]
    pub fn copysign_float32_not_copy_works()->Result<(),Error>{
        binop_f32_works!(2.0,2.0,1.0,Instruction::F32Copysign)
    }

    #[test]
    pub fn copysign_float64_copy_works()->Result<(),Error>{
        binop_f64_works!(-2.0_f64,2.0,-1.0_f64,Instruction::F64Copysign)
    }

    #[test]
    pub fn copysign_float64_not_copy_works()->Result<(),Error>{
        binop_f64_works!(2.0,2.0,1.0,Instruction::F64Copysign)
    }



    #[test]
    pub fn rem_sint32_1_works()->Result<(),Error>{
        binop_s32_works!(0,9,3,Instruction::I32RemS)
    }

    #[test]
    pub fn rem_sint32_2_works()->Result<(),Error>{
        binop_s32_works!(1,4,3,Instruction::I32RemS)
    }


    #[test]
    pub fn rem_sint32_3_works()->Result<(),Error>{
        binop_s32_works!(-1_i32,-1_i32,3,Instruction::I32RemS)
    }


    #[test]
    pub fn rem_uint32_1_works()->Result<(),Error>{
        binop_u32_works!(0,9,3,Instruction::I32RemU)
    }

    #[test]
    pub fn rem_uint32_2_works()->Result<(),Error>{
        binop_u32_works!(1,4,3,Instruction::I32RemU)
    }


    #[test]
    pub fn rem_uint32_3_works()->Result<(),Error>{
        binop_u32_works!(0,-1_i32 as u32,3,Instruction::I32RemU)
    }



    #[test]
    pub fn rem_sint64_1_works()->Result<(),Error>{
        binop_s64_works!(0,9,3,Instruction::I64RemS)
    }

    #[test]
    pub fn rem_sint64_2_works()->Result<(),Error>{
        binop_s64_works!(1,4,3,Instruction::I64RemS)
    }


    #[test]
    pub fn rem_sint64_3_works()->Result<(),Error>{
        binop_s64_works!(-1_i64,-1_i64,3,Instruction::I64RemS)
    }


    #[test]
    pub fn rem_uint64_1_works()->Result<(),Error>{
        binop_u64_works!(0,9,3,Instruction::I64RemU)
    }

    #[test]
    pub fn rem_uint64_2_works()->Result<(),Error>{
        binop_u64_works!(1,4,3,Instruction::I64RemU)
    }


    #[test]
    pub fn rem_uint64_3_works()->Result<(),Error>{
        binop_u64_works!(0,-1_i64 as u64,3,Instruction::I64RemU)
    }


    #[test]
    pub fn and32_works()->Result<(),Error>{
        binop_u32_works!(3,-1_i32 as u32,3,Instruction::I32And)
    }


    #[test]
    pub fn and64_works()->Result<(),Error>{
        binop_u64_works!(3,-1_i64 as u64,3,Instruction::I64And)
    }


    #[test]
    pub fn or32_works()->Result<(),Error>{
        binop_u32_works!(3,-1_i32 as u32,3,Instruction::I32Or)
    }


    #[test]
    pub fn or64_works()->Result<(),Error>{
        binop_u64_works!(3,-1_i64 as u64,3,Instruction::I64Or)
    }


    #[test]
    pub fn xor32_works()->Result<(),Error>{
        binop_u32_works!(-4_i32 as u32,-1_i32 as u32,3,Instruction::I32Xor)
    }


    #[test]
    pub fn xor64_works()->Result<(),Error>{
        binop_u64_works!(-4_i64 as u64,-1_i64 as u64,3,Instruction::I64Xor)
    }


    #[test]
    pub fn shl32_works()->Result<(),Error>{
        binop_u32_works!(24,3,3,Instruction::I32Shl)
    }


    #[test]
    pub fn shl64_works()->Result<(),Error>{
        binop_u64_works!(24,3,3,Instruction::I64Shl)
    }

    #[test]
    pub fn shr_s32_works()->Result<(),Error>{
        binop_s32_works!(6,24,2,Instruction::I32ShrS)
    }

    #[test]
    pub fn shr_s32_sign_works()->Result<(),Error>{
        binop_s32_works!(-6_i32,-24_i32,2,Instruction::I32ShrS)
    }



    #[test]
    pub fn shr_s64_works()->Result<(),Error>{
        binop_s64_works!(6,24,2,Instruction::I64ShrS)
    }


    #[test]
    pub fn shr_s64_sign_works()->Result<(),Error>{
        binop_s64_works!(-6_i64,-24_i64,2,Instruction::I64ShrS)
    }



    #[test]
    pub fn shr_u32_works()->Result<(),Error>{
        binop_u32_works!(6,24,2,Instruction::I32ShrU)
    }

    #[test]
    pub fn shr_u32_unsigned_works()->Result<(),Error>{
        binop_u32_works!(1073741823,-1_i32 as u32,2,Instruction::I32ShrU)
    }



    #[test]
    pub fn shr_u64_works()->Result<(),Error>{
        binop_u64_works!(6,24,2,Instruction::I64ShrU)
    }


    #[test]
    pub fn shr_u64_unsigned_works()->Result<(),Error>{
        binop_u64_works!(4611686018427387903,-1_i64 as u64,2,Instruction::I64ShrU)
    }


    #[test]
    pub fn rotl_u32_works()->Result<(),Error>{
        binop_u32_works!(4294967295,4294967295,5,Instruction::I32Rotl)
    }

    #[test]
    pub fn rotl_u32_max_works()->Result<(),Error>{
        binop_u32_works!(1,0x80_00_00_00,1,Instruction::I32Rotl)
    }

    #[test]
    pub fn rotr_u32_works()->Result<(),Error>{
        binop_u32_works!(4294967295,4294967295,5,Instruction::I32Rotr)
    }

    #[test]
    pub fn rotr_u32_max_works()->Result<(),Error>{
        binop_u32_works!(0x80_00_00_00,1,1,Instruction::I32Rotr)
    }




    #[test]
    pub fn rotl_u64_works()->Result<(),Error>{
        binop_u64_works!(0xFFFFFFFF_FFFFFFFF,0xFFFFFFFF_FFFFFFFF,5,Instruction::I64Rotl)
    }

    #[test]
    pub fn rotl_u64_max_works()->Result<(),Error>{
        binop_u64_works!(1,0x80000000_00000000,1,Instruction::I64Rotl)
    }

    #[test]
    pub fn rotr_u64_works()->Result<(),Error>{
        binop_u64_works!(0xFFFFFFFF_FFFFFFFF,0xFFFFFFFF_FFFFFFFF,5,Instruction::I64Rotr)
    }

    #[test]
    pub fn rotr_u64_max_works()->Result<(),Error>{
        binop_u64_works!(0x80000000_00000000,1,1,Instruction::I64Rotr)
    }




    #[test]
    pub fn eq_i32_true_works() -> Result<(),Error>{
        relop_u32_works!(1,2,2,Instruction::I32Eq)
    }

    #[test]
    pub fn eq_i32_false_works() -> Result<(),Error>{
        relop_u32_works!(0,3,2,Instruction::I32Eq)
    }

    #[test]
    pub fn eq_i64_true_works() -> Result<(),Error>{
        relop_u64_works!(1,2,2,Instruction::I64Eq)
    }

    #[test]
    pub fn eq_i64_false_works() -> Result<(),Error>{
        relop_u64_works!(0,3,2,Instruction::I64Eq)
    }

    #[test]
    pub fn eq_f32_true_works() -> Result<(),Error>{
        relop_f32_works!(1,2.0,2.0,Instruction::F32Eq)
    }

    #[test]
    pub fn eq_f32_false_works() -> Result<(),Error>{
        relop_f32_works!(0,3.0,2.0,Instruction::F32Eq)
    }

    #[test]
    pub fn eq_f64_true_works() -> Result<(),Error>{
        relop_f64_works!(1,2.0,2.0,Instruction::F64Eq)
    }

    #[test]
    pub fn eq_f64_false_works() -> Result<(),Error>{
        relop_f64_works!(0,3.0,2.0,Instruction::F64Eq)
    }


    #[test]
    pub fn ne_i32_true_works() -> Result<(),Error>{
        relop_u32_works!(1,3,2,Instruction::I32Ne)
    }

    #[test]
    pub fn ne_i32_false_works() -> Result<(),Error>{
        relop_u32_works!(0,2,2,Instruction::I32Ne)
    }

    #[test]
    pub fn ne_i64_true_works() -> Result<(),Error>{
        relop_u64_works!(1,3,2,Instruction::I64Ne)
    }

    #[test]
    pub fn ne_i64_false_works() -> Result<(),Error>{
        relop_u64_works!(0,2,2,Instruction::I64Ne)
    }

    #[test]
    pub fn ne_f32_true_works() -> Result<(),Error>{
        relop_f32_works!(1,3.0,2.0,Instruction::F32Ne)
    }

    #[test]
    pub fn ne_f32_false_works() -> Result<(),Error>{
        relop_f32_works!(0,2.0,2.0,Instruction::F32Ne)
    }

    #[test]
    pub fn ne_f64_true_works() -> Result<(),Error>{
        relop_f64_works!(1,3.0,2.0,Instruction::F64Ne)
    }

    #[test]
    pub fn ne_f64_false_works() -> Result<(),Error>{
        relop_f64_works!(0,2.0,2.0,Instruction::F64Ne)
    }


    #[test]
    pub fn lt_s32_true_works() -> Result<(),Error>{
        relop_s32_works!(1,-1_i32,2,Instruction::I32LtS)
    }

    #[test]
    pub fn lt_s32_eq_works() -> Result<(),Error>{
        relop_s32_works!(0,2,2,Instruction::I32LtS)
    }

    #[test]
    pub fn lt_s32_gt_works() -> Result<(),Error>{
        relop_s32_works!(0,3,2,Instruction::I32LtS)
    }

    #[test]
    pub fn lt_u32_true_works() -> Result<(),Error>{
        relop_u32_works!(1,1,2,Instruction::I32LtU)
    }

    #[test]
    pub fn lt_u32_eq_works() -> Result<(),Error>{
        relop_u32_works!(0,2,2,Instruction::I32LtU)
    }

    #[test]
    pub fn lt_u32_gt_works() -> Result<(),Error>{
        relop_u32_works!(0,-1_i32 as u32,2,Instruction::I32LtU)
    }

    #[test]
    pub fn lt_s64_true_works() -> Result<(),Error>{
        relop_s64_works!(1,-1_i64,2,Instruction::I64LtS)
    }

    #[test]
    pub fn lt_s64_eq_works() -> Result<(),Error>{
        relop_s64_works!(0,2,2,Instruction::I64LtS)
    }

    #[test]
    pub fn lt_s64_gt_works() -> Result<(),Error>{
        relop_s64_works!(0,3,2,Instruction::I64LtS)
    }

    #[test]
    pub fn lt_u64_true_works() -> Result<(),Error>{
        relop_u64_works!(1,1,2,Instruction::I64LtU)
    }

    #[test]
    pub fn lt_u64_eq_works() -> Result<(),Error>{
        relop_u64_works!(0,2,2,Instruction::I64LtU)
    }

    #[test]
    pub fn lt_u64_gt_works() -> Result<(),Error>{
        relop_u64_works!(0,-1_i64 as u64,2,Instruction::I64LtU)
    }



    #[test]
    pub fn lt_f32_true_works() -> Result<(),Error>{
        relop_f32_works!(1,1.0,2.0,Instruction::F32Lt)
    }

    #[test]
    pub fn lt_f32_eq_works() -> Result<(),Error>{
        relop_f32_works!(0,2.0,2.0,Instruction::F32Lt)
    }

    #[test]
    pub fn lt_f32_gt_works() -> Result<(),Error>{
        relop_f32_works!(0,3.0,2.0,Instruction::F32Lt)
    }

    #[test]
    pub fn lt_f64_true_works() -> Result<(),Error>{
        relop_f64_works!(1,1.0,2.0,Instruction::F64Lt)
    }

    #[test]
    pub fn lt_f64_eq_works() -> Result<(),Error>{
        relop_f64_works!(0,2.0,2.0,Instruction::F64Lt)
    }

    #[test]
    pub fn lt_f64_gt_works() -> Result<(),Error>{
        relop_f64_works!(0,3.0,2.0,Instruction::F64Lt)
    }


    #[test]
    pub fn le_s32_true_works() -> Result<(),Error>{
        relop_s32_works!(1,-1_i32,2,Instruction::I32LeS)
    }

    #[test]
    pub fn le_s32_eq_works() -> Result<(),Error>{
        relop_s32_works!(1,2,2,Instruction::I32LeS)
    }

    #[test]
    pub fn le_s32_gt_works() -> Result<(),Error>{
        relop_s32_works!(0,3,2,Instruction::I32LeS)
    }

    #[test]
    pub fn le_u32_true_works() -> Result<(),Error>{
        relop_u32_works!(1,1,2,Instruction::I32LeU)
    }

    #[test]
    pub fn le_u32_eq_works() -> Result<(),Error>{
        relop_u32_works!(1,2,2,Instruction::I32LeU)
    }

    #[test]
    pub fn le_u32_gt_works() -> Result<(),Error>{
        relop_u32_works!(0,-1_i32 as u32,2,Instruction::I32LeU)
    }

    #[test]
    pub fn le_s64_true_works() -> Result<(),Error>{
        relop_s64_works!(1,-1_i64,2,Instruction::I64LeS)
    }

    #[test]
    pub fn le_s64_eq_works() -> Result<(),Error>{
        relop_s64_works!(1,2,2,Instruction::I64LeS)
    }

    #[test]
    pub fn le_s64_gt_works() -> Result<(),Error>{
        relop_s64_works!(0,3,2,Instruction::I64LeS)
    }

    #[test]
    pub fn le_u64_true_works() -> Result<(),Error>{
        relop_u64_works!(1,1,2,Instruction::I64LeU)
    }

    #[test]
    pub fn le_u64_eq_works() -> Result<(),Error>{
        relop_u64_works!(1,2,2,Instruction::I64LeU)
    }

    #[test]
    pub fn le_u64_gt_works() -> Result<(),Error>{
        relop_u64_works!(0,-1_i64 as u64,2,Instruction::I64LeU)
    }
    

    #[test]
    pub fn le_f32_true_works() -> Result<(),Error>{
        relop_f32_works!(1,1.0,2.0,Instruction::F32Le)
    }

    #[test]
    pub fn le_f32_eq_works() -> Result<(),Error>{
        relop_f32_works!(1,2.0,2.0,Instruction::F32Le)
    }

    #[test]
    pub fn le_f32_gt_works() -> Result<(),Error>{
        relop_f32_works!(0,3.0,2.0,Instruction::F32Le)
    }

    #[test]
    pub fn le_f64_true_works() -> Result<(),Error>{
        relop_f64_works!(1,1.0,2.0,Instruction::F64Le)
    }

    #[test]
    pub fn le_f64_eq_works() -> Result<(),Error>{
        relop_f64_works!(1,2.0,2.0,Instruction::F64Le)
    }

    #[test]
    pub fn le_f64_gt_works() -> Result<(),Error>{
        relop_f64_works!(0,3.0,2.0,Instruction::F64Le)
    }


    #[test]
    pub fn gt_s32_true_works() -> Result<(),Error>{
        relop_s32_works!(1,3,-1_i32,Instruction::I32GtS)
    }

    #[test]
    pub fn gt_s32_eq_works() -> Result<(),Error>{
        relop_s32_works!(0,2,2,Instruction::I32GtS)
    }

    #[test]
    pub fn gt_s32_lt_works() -> Result<(),Error>{
        relop_s32_works!(0,-1_i32,2,Instruction::I32GtS)
    }

    #[test]
    pub fn gt_u32_true_works() -> Result<(),Error>{
        relop_u32_works!(1,3,2,Instruction::I32GtU)
    }

    #[test]
    pub fn gt_u32_eq_works() -> Result<(),Error>{
        relop_u32_works!(0,2,2,Instruction::I32GtU)
    }

    #[test]
    pub fn gt_u32_lt_works() -> Result<(),Error>{
        relop_u32_works!(0,2,-1_i32 as u32,Instruction::I32GtU)
    }

    #[test]
    pub fn gt_s64_true_works() -> Result<(),Error>{
        relop_s64_works!(1,2,-1_i64, Instruction::I64GtS)
    }

    #[test]
    pub fn gt_s64_eq_works() -> Result<(),Error>{
        relop_s64_works!(0,2,2,Instruction::I64GtS)
    }

    #[test]
    pub fn gt_s64_lt_works() -> Result<(),Error>{
        relop_s64_works!(0,-1_i64,2,Instruction::I64GtS)
    }

    #[test]
    pub fn gt_u64_true_works() -> Result<(),Error>{
        relop_u64_works!(1,-1_i64 as u64,1,Instruction::I64GtU)
    }

    #[test]
    pub fn gt_u64_eq_works() -> Result<(),Error>{
        relop_u64_works!(0,2,2,Instruction::I64GtU)
    }

    #[test]
    pub fn gt_u64_lt_works() -> Result<(),Error>{
        relop_u64_works!(0,2,-1_i64 as u64,Instruction::I64GtU)
    }



    #[test]
    pub fn gt_f32_true_works() -> Result<(),Error>{
        relop_f32_works!(1,3.0,2.0,Instruction::F32Gt)
    }

    #[test]
    pub fn gt_f32_eq_works() -> Result<(),Error>{
        relop_f32_works!(0,2.0,2.0,Instruction::F32Gt)
    }

    #[test]
    pub fn gt_f32_lt_works() -> Result<(),Error>{
        relop_f32_works!(0,1.0,2.0,Instruction::F32Gt)
    }

    #[test]
    pub fn gt_f64_true_works() -> Result<(),Error>{
        relop_f64_works!(1,3.0,2.0,Instruction::F64Gt)
    }

    #[test]
    pub fn gt_f64_eq_works() -> Result<(),Error>{
        relop_f64_works!(0,2.0,2.0,Instruction::F64Gt)
    }

    #[test]
    pub fn gt_f64_lt_works() -> Result<(),Error>{
        relop_f64_works!(0,1.0,2.0,Instruction::F64Gt)
    }


    #[test]
    pub fn ge_s32_true_works() -> Result<(),Error>{
        relop_s32_works!(1,2,-1_i32,Instruction::I32GeS)
    }

    #[test]
    pub fn ge_s32_eq_works() -> Result<(),Error>{
        relop_s32_works!(1,2,2,Instruction::I32GeS)
    }

    #[test]
    pub fn ge_s32_lt_works() -> Result<(),Error>{
        relop_s32_works!(0,2,3,Instruction::I32GeS)
    }

    #[test]
    pub fn ge_u32_true_works() -> Result<(),Error>{
        relop_u32_works!(1,2,1,Instruction::I32GeU)
    }

    #[test]
    pub fn ge_u32_eq_works() -> Result<(),Error>{
        relop_u32_works!(1,2,2,Instruction::I32GeU)
    }

    #[test]
    pub fn ge_u32_lt_works() -> Result<(),Error>{
        relop_u32_works!(0,2,-1_i32 as u32,Instruction::I32GeU)
    }

    #[test]
    pub fn ge_s64_true_works() -> Result<(),Error>{
        relop_s64_works!(1,2,-1_i64,Instruction::I64GeS)
    }

    #[test]
    pub fn ge_s64_eq_works() -> Result<(),Error>{
        relop_s64_works!(1,2,2,Instruction::I64GeS)
    }

    #[test]
    pub fn ge_s64_lt_works() -> Result<(),Error>{
        relop_s64_works!(0,2,3,Instruction::I64GeS)
    }

    #[test]
    pub fn ge_u64_true_works() -> Result<(),Error>{
        relop_u64_works!(1,2,1,Instruction::I64GeU)
    }

    #[test]
    pub fn ge_u64_eq_works() -> Result<(),Error>{
        relop_u64_works!(1,2,2,Instruction::I64GeU)
    }

    #[test]
    pub fn ge_u64_lt_works() -> Result<(),Error>{
        relop_u64_works!(0,2,-1_i64 as u64,Instruction::I64GeU)
    }


    #[test]
    pub fn ge_f32_true_works() -> Result<(),Error>{
        relop_f32_works!(1,3.0,2.0,Instruction::F32Ge)
    }

    #[test]
    pub fn ge_f32_eq_works() -> Result<(),Error>{
        relop_f32_works!(1,2.0,2.0,Instruction::F32Ge)
    }

    #[test]
    pub fn ge_f32_lt_works() -> Result<(),Error>{
        relop_f32_works!(0,1.0,2.0,Instruction::F32Ge)
    }

    #[test]
    pub fn ge_f64_true_works() -> Result<(),Error>{
        relop_f64_works!(1,3.0,2.0,Instruction::F64Ge)
    }

    #[test]
    pub fn ge_f64_eq_works() -> Result<(),Error>{
        relop_f64_works!(1,2.0,2.0,Instruction::F64Ge)
    }

    #[test]
    pub fn ge_f64_lt_works() -> Result<(),Error>{
        relop_f64_works!(0,1.0,2.0,Instruction::F64Ge)
    }



    #[test]
    pub fn eqz32_false_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("eqz32_false_works",&context);
        let (ft,lt ) = new_compilers();
        let expected =0;
        let test_function_name = "test_function";
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![Value::const_int(Type::int32(build_context.context()),22,false)],
            vec![],|stack:Stack<u32>,_|{
                let mut stack = eqz32(&build_context,stack)?;
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
    pub fn eqz32_true_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("eqz32_true_works",&context);
        let (ft,lt ) = new_compilers();
        let expected =1;
        let test_function_name = "test_function";
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![Value::const_int(Type::int32(build_context.context()),0,false)],
                                                  vec![],|stack:Stack<u32>,_|{
                let mut stack = eqz32(&build_context,stack)?;
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
    pub fn eqz64_false_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("eqz64_false_works",&context);
        let (ft,lt ) = new_compilers();
        let expected =0;
        let test_function_name = "test_function";
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),22,false)],
                                                  vec![],|stack:Stack<u32>,_|{
                let mut stack = eqz64(&build_context,stack)?;
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
    pub fn eqz64_true_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("eqz64_true_works",&context);
        let (ft,lt ) = new_compilers();
        let expected =1;
        let test_function_name = "test_function";
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![Value::const_int(Type::int64(build_context.context()),0,false)],
                                                  vec![],|stack:Stack<u32>,_|{
                let mut stack = eqz64(&build_context,stack)?;
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