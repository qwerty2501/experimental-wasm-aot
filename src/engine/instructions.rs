
use super::*;
use failure::Error;
use error::RuntimeError::*;
use parity_wasm::elements::{Instruction,ValueType};
use parity_wasm::elements::BlockType;
use std::slice::Iter;

const WASM_GLOBAL_PREFIX:&str = "__WASM_GLOBAL_";

 fn i64_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:i64,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push( WasmValue::new_value( i64_const_internal(build_context,v)));
    Ok(stack)
}

pub fn i64_const_internal<'c>(build_context:&'c BuildContext, v:i64) ->&'c Value{
    Value::const_int(Type::int64(build_context.context()),v as u64,true)
}

 fn i32_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:i32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(WasmValue::new_value( i32_const_internal(build_context,v)));
    Ok(stack)
}

pub fn i32_const_internal<'c>(build_context:&'c BuildContext, v:i32) ->&'c Value{
    Value::const_int(Type::int32(build_context.context()),v as u64,true)
}

 fn f64_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:f64,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(WasmValue::new_value(f64_const_internal(build_context,v)));
    Ok(stack)
}

pub fn f64_const_internal<'c>(build_context:&'c BuildContext, v:f64) ->&'c Value{
    Value::const_real(Type::float64(build_context.context()),v)
}

 fn f32_const<'c,T:WasmIntType>(build_context:&'c BuildContext,v:f32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    stack.values.push(WasmValue::new_value(f32_const_internal(build_context,v)));
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
    stack.values.push(WasmValue::new_value( build_context.builder().build_load(global_value,"")));
    Ok(stack)
}

 fn set_global<'c,T:WasmIntType>(build_context:&'c BuildContext,index:u32,mut stack:Stack<'c,T>)->Result<Stack<'c,T>,Error>{
    let global_value = get_global_internal(build_context,index)?;
    build_context.builder().build_store( stack.values.pop().ok_or(NotExistValue)?.to_value(build_context),global_value);
    Ok(stack)
}

 fn get_local<'a,T:WasmIntType>(_build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current()?;
        let v = current_frame.locals.get(index as usize).ok_or(NoSuchLocalValue {index})?.clone().value.ok_or(NoSuchLocalValue {index})?.clone();
        stack.values.push( v);
    }
    Ok(stack)
}

 fn set_local<'a,T:WasmIntType>(_build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  v:&mut LocalValue = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        v.value = Some(stack.values.pop().ok_or(NotExistValue)?);
    }
    Ok(stack)
}

 fn tee_local<'a,T:WasmIntType>(_build_context:&'a BuildContext,index:u32,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let  v:&mut LocalValue = current_frame.locals.get_mut(index as usize).ok_or(NoSuchLocalValue{index})?;
        v.value = Some(stack.values.last().ok_or(NotExistValue)?.clone());

    }
    Ok(stack)
}

 fn store<'a,T:WasmIntType>(build_context:&'a BuildContext,align:u32,offset:u32,mut stack:Stack<'a,T>,value_type:&Type)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        build_check_memory_size_const(build_context, 0, offset,  current_frame.module_instance.linear_memory_compiler)?;
        let v = stack.values.pop().ok_or(NotExistValue)?;
        let memory = current_frame.module_instance.linear_memory_compiler.build_get_real_address(build_context,0,Value::const_int(Type::int32(build_context.context()),offset as u64,false),"");
        let v = build_context.builder().build_cast(Opcode::LLVMTrunc,v.to_value(&build_context),value_type,"");
        v.set_alignment(u32::pow(2,align));
        let memory = build_context.builder().build_bit_cast(memory,Type::ptr(value_type,0),"");
        build_context.builder().build_store(v,memory);
    }
    Ok(stack)

}

 fn load<'a,T:WasmIntType>(build_context:&'a BuildContext,align:u32,offset:u32,mut stack:Stack<'a,T>,value_type:&Type)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        build_check_memory_size_const(build_context,0,offset,current_frame.module_instance.linear_memory_compiler)?;
        let memory = current_frame.module_instance.linear_memory_compiler.build_get_real_address(build_context,0,Value::const_int(Type::int32(build_context.context()),offset as u64,false),"");
        let memory = build_context.builder().build_bit_cast(memory,Type::ptr(value_type,0),"");
        let v = build_context.builder().build_load(memory,"");
        v.set_alignment(u32::pow(2,align));
        stack.values.push(WasmValue::new_value( v));
    }
    Ok(stack)
}

 fn end<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
     {
         let previous_instruction = stack.activations.current()?.previous_instruction.clone();
         if let Some(label) = stack.labels.pop(){
             let need_br_next =  if let Some(pi) = previous_instruction.clone(){
                 match pi{
                     Instruction::Br(_) | Instruction::BrTable(_,_) => false,
                     _ => true,
                 }
             } else{
                 true
             };


             let next = match label.label_type {
                 LabelType::If {start:_,else_block:_,next} => { next },
                 LabelType::Block {start:_,next} => next,
                 LabelType::Loop {start:_,next} => next,
             };
             if need_br_next {
                 if let Some(return_value) = label.return_value {
                     if let Some(Instruction::Unreachable) = previous_instruction.clone(){
                         if let Some(ret_value) = stack.values.pop() {
                             return_value.store(build_context, ret_value.to_value(build_context));
                         }
                     } else{
                         let ret_value = stack.values.pop().ok_or(NotExistValue)?.to_value(build_context);
                         return_value.store(build_context, ret_value);
                     }
                     stack.values.push(WasmValue::new_block_return_value(return_value));
                 }
                 build_context.builder().build_br(next);
             }

             let last_basic_block = stack.current_function.get_last_basic_block();
             if let Some(last_basic_block) = last_basic_block {
                 next.move_after(last_basic_block);
             }

             build_context.builder().position_builder_at_end(next);
             let current_frame = stack.activations.current()?;

         } else{
             if let Some(Instruction::Unreachable) = previous_instruction.clone(){
                 let function_type = Type::type_of(stack.current_function);
                 let return_type = function_type.get_return_type();
                 if let Some(ret_value) = stack.values.pop() {
                     build_context.builder().build_ret(ret_value.to_value(build_context));
                 } else{
                     if return_type == Type::void(build_context.context()){
                         build_context.builder().build_ret_void();
                     } else if return_type == Type::int32(build_context.context()){
                         build_context.builder().build_ret(Value::const_int(Type::int32(build_context.context()),0,false));
                     } else if return_type == Type::int64(build_context.context()){
                         build_context.builder().build_ret(Value::const_int(Type::int64(build_context.context()),0,false));
                     } else if return_type == Type::float32(build_context.context()){
                         build_context.builder().build_ret(Value::const_real(Type::float32(build_context.context()),0.0));
                     } else if return_type == Type::float64(build_context.context()){
                         build_context.builder().build_ret(Value::const_real(Type::float64(build_context.context()),0.0));
                     }
                 }
             } else{
                 if let Some(ret_value) = stack.values.pop() {
                    build_context.builder().build_ret(ret_value.to_value(build_context));
                 } else{
                     build_context.builder().build_ret_void();
                 }
             }
         }




     }
    Ok(stack)
}

 fn current_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;

        stack.values.push(WasmValue::new_value( current_frame.module_instance.linear_memory_compiler.build_get_memory_size(build_context,index as u32)));
    }
    Ok(stack)

}

 fn grow_memory<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u8,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let current_frame = stack.activations.current_mut()?;
        let grow_memory_function_name = current_frame.module_instance.linear_memory_compiler.get_grow_function_name(index as u32);
        let grow_memory_function =  build_context.module().get_named_function(&grow_memory_function_name).ok_or(NoSuchLLVMFunction {name:grow_memory_function_name})?;
        let grow_memory_size = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push( WasmValue::new_value( build_context.builder().build_call(grow_memory_function,&[grow_memory_size.to_value(&build_context)],"")));
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
        stack.values.push(WasmValue::new_value( on_unop(x.to_value(&build_context))))
    }
    Ok(stack)
}

 fn add_int<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>) ->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_add(lhs, rhs, name))
}

 fn add_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fadd(lhs, rhs, name))
}

 fn mul_int<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_mul(lhs, rhs, name))
}

 fn mul_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fmul(lhs, rhs, name))
}

 fn sub_int<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_sub(lhs, rhs, name))
}

 fn sub_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fsub(lhs, rhs, name))
}

 fn div_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_udiv(lhs, rhs, name))
}

 fn div_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_sdiv(lhs, rhs, name))
}

 fn div_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context, stack, |lhs, rhs, name|build_context.builder().build_fdiv(lhs, rhs, name))
}

fn min_float32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fminf(build_context.module(),build_context.builder(),lhs,rhs,name))
}

fn min_float64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fmin(build_context.module(), build_context.builder(), lhs, rhs, name))
}

fn max_float32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fmaxf(build_context.module(),build_context.builder(),lhs,rhs,name))
}

fn max_float64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_fmax(build_context.module(), build_context.builder(), lhs, rhs, name))
}

fn copysign_float32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_copysignf(build_context.module(),build_context.builder(),lhs,rhs,name))
}

fn copysign_float64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name| build_call_and_set_copysign(build_context.module(), build_context.builder(), lhs, rhs, name))
}

fn rem_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_srem(lhs,rhs,name))
}

fn rem_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_urem(lhs,rhs,name))
}

fn and<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_and(lhs,rhs,name))
}

fn or<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_or(lhs,rhs,name))
}

fn xor<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_xor(lhs,rhs,name))
}

fn shl<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_shl(lhs,rhs,name))
}

fn shr_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_ashr(lhs,rhs,name))
}

fn shr_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    binop(build_context,stack,|lhs,rhs,name|build_context.builder().build_lshr(lhs,rhs,name))
}

fn rotl<'a,T:WasmIntType,W:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
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

fn rotr<'a,T:WasmIntType,W:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
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
        stack.values.push(  WasmValue::new_value( on_binop(lhs.to_value(&build_context),rhs.to_value(&build_context),"")));
    }
    Ok(stack)
}


 fn  eqz32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error> {
    eqz(build_context,Type::int32(build_context.context()),stack)
}

 fn eqz64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    eqz(build_context,Type::int64(build_context.context()),stack)
}
fn eqz<'a,T:WasmIntType>(build_context:&'a BuildContext,type_ref:&'a Type,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    let i = stack.values.pop().ok_or(NotExistValue)?;
    stack.values.push( WasmValue::new_value( build_context.builder().build_icmp(IntPredicate::LLVMIntEQ,i.to_value(&build_context),Value::const_int(type_ref,0,false),"")));
    Ok(stack)
}

 fn eq_int<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntEQ,lhs,rhs,name))
}

 fn eq_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOEQ,lhs,rhs,name))
}

 fn ne_int<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntNE,lhs,rhs,name))
}

fn ne_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealONE,lhs,rhs,name))
}

fn lt_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSLT,lhs,rhs,name))
}

fn lt_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntULT,lhs,rhs,name))
}

fn lt_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOLT,lhs,rhs,name))
}


fn le_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSLE,lhs,rhs,name))
}

fn le_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntULE,lhs,rhs,name))
}

fn le_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOLE,lhs,rhs,name))
}


fn gt_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSGT,lhs,rhs,name))
}

fn gt_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntUGT,lhs,rhs,name))
}

fn gt_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOGT,lhs,rhs,name))
}


fn ge_sint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntSGE,lhs,rhs,name))
}

fn ge_uint<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_icmp(IntPredicate::LLVMIntUGE,lhs,rhs,name))
}

fn ge_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    relop(build_context,stack,|lhs,rhs,name|build_context.builder().build_fcmp(RealPredicate::LLVMRealOGE,lhs,rhs,name))
}



fn relop<'a,T:WasmIntType,F:Fn(&'a Value,&'a Value,&'a str)->&'a Value>(build_context:&'a BuildContext, mut stack:Stack<'a,T>, on_relop:F) ->Result<Stack<'a,T>,Error>{
    {
        let rhs = stack.values.pop().ok_or(NotExistValue)?;
        let lhs = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push( WasmValue::new_value( build_context.builder().build_zext_or_bit_cast(  on_relop(lhs.to_value(&build_context),rhs.to_value(&build_context),""),Type::int32(build_context.context()),"")));
    }
    Ok(stack)
}

fn wrap_i64_to_i32<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>) -> Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name|build_context.builder().build_trunc(x,Type::int32(build_context.context()),name))
}

fn extend_u32_to_i64<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>) -> Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_zext(x,Type::int64(build_context.context()),name))
}

fn extend_s32_to_i64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>) -> Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_sext(x,Type::int64(build_context.context()),name))
}

fn trunc_float_to_s32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)-> Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_fp_to_si(x,Type::int32(build_context.context()),name))
}

fn trunc_float_to_u32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_fp_to_ui(x,Type::int32(build_context.context()),name))
}

fn trunc_float_to_s64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_fp_to_si(x,Type::int64(build_context.context()),name))
}


fn trunc_float_to_u64<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_fp_to_ui(x,Type::int64(build_context.context()),name))
}

fn demote_float<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_fp_trunc(x,Type::float32(build_context.context()),name))
}

fn promote_float<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_fp_ext(x,Type::float64(build_context.context()),name))
}


fn convert_sint_to_f32<'a,T:WasmIntType>(build_context:&'a BuildContext,  stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_si_to_fp(x,Type::float32(build_context.context()),name))
}

fn convert_sint_to_f64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_si_to_fp(x,Type::float64(build_context.context()),name))
}

fn convert_uint_to_f32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_ui_to_fp(x,Type::float32(build_context.context()),name))
}

fn convert_uint_to_f64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_ui_to_fp(x,Type::float64(build_context.context()),name))
}

fn reinter_pret_int_to_f32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_bit_cast(x,Type::float32(build_context.context()),name))
}

fn reinter_pret_int_to_f64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name| build_context.builder().build_bit_cast(x,Type::float64(build_context.context()),name))
}

fn reinter_pret_float_to_i32<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>) -> Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name|build_context.builder().build_bit_cast(x,Type::int32(build_context.context()),name) )
}


fn reinter_pret_float_to_i64<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>) -> Result<Stack<'a,T>,Error>{
    cutop(build_context,stack,|x,name|build_context.builder().build_bit_cast(x,Type::int64(build_context.context()),name) )
}




fn cutop<'a,T:WasmIntType,F:Fn(&'a Value,&'a str)->&'a Value>(build_context:&'a BuildContext, mut stack:Stack<'a,T>, on_cutop:F) ->Result<Stack<'a,T>,Error>{
    {
        let x = stack.values.pop().ok_or(NotExistValue)?;
        stack.values.push(WasmValue::new_value( on_cutop(x.to_value(&build_context),"")))
    }
    Ok(stack)
}

pub fn get_global_name(index:u32) -> String {
    [WASM_GLOBAL_PREFIX,index.to_string().as_ref()].concat()
}

fn block<'a,T:WasmIntType>(build_context:&'a BuildContext, mut stack:Stack<'a,T>,block_type: BlockType)->Result<Stack<'a,T>,Error>{
    {
        let block_return_value = if let BlockType::Value(value_type)  = block_type {
            Some(BlockReturnValue::new(build_context,value_type))
        } else{
            None
        };
        let start = stack.current_function.append_basic_block(build_context.context(),"");
        let next = stack.current_function.append_basic_block(build_context.context(),"");
        build_context.builder().build_br(start);
        build_context.builder().position_builder_at_end(start);
        stack.labels.push( Label::new_block(start,next,block_return_value))
    }
    Ok(stack)
}

fn loop_instruction<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,block_type:BlockType)->Result<Stack<'a,T>,Error>{
    {
        let block_return_value = if let BlockType::Value(value_type) = block_type {
            Some(BlockReturnValue::new(build_context,value_type))
        } else{
            None
        };
        let start = stack.current_function.append_basic_block(build_context.context(),"");
        let next = stack.current_function.append_basic_block(build_context.context(),"");
        build_context.builder().build_br(start);
        build_context.builder().position_builder_at_end(start);
        stack.labels.push( Label::new_loop(start,next,block_return_value))
    }
    Ok(stack)
}

fn if_instruction<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,block_type:BlockType)-> Result<Stack<'a,T>,Error>{
    {
        let block_return_value = if let BlockType::Value(value_type) = block_type {
            Some(BlockReturnValue::new(build_context,value_type))
        } else{
            None
        };

        let start = stack.current_function.append_basic_block(build_context.context(),"");
        let else_block = stack.current_function.append_basic_block(build_context.context(),"");
        let next = stack.current_function.append_basic_block(build_context.context(),"");
        let cond = stack.values.pop().ok_or(NotExistValue)?;
        build_cond_br(build_context.builder(),build_context.context(),cond.to_value(build_context),start,else_block);
        build_context.builder().position_builder_at_end(start);
        stack.labels.push( Label::new_if(start,else_block,next,block_return_value))
    }
    Ok(stack)
}

fn else_instruction<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    {
        let label = stack.labels.last().ok_or(NotExistLabel)?;
        if let LabelType::If {start:_,else_block,next} = label.label_type{
            if let Some(return_value) = label.return_value.clone() {
                return_value.store(build_context,stack.values.pop().ok_or(NotExistValue)?.to_value(build_context));
            }
            build_context.builder().build_br(next);
            build_context.builder().position_builder_at_end(else_block);
        } else{
            Err(InvalidLabelType)?
        }
    }
    Ok(stack)
}

fn br<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,label_index:u32)->Result<Stack<'a,T>,Error>{
    {
        let label = stack.labels.get(label_index as usize).ok_or(NoSuchLabel{index:label_index})?.clone();


        if let Some(return_value) = label.return_value.clone() {
            return_value.store(build_context,stack.values.last().ok_or(NotExistValue)?.to_value(build_context));
        }

        let br_block = match label.label_type {
            LabelType::Loop {start,next:_} => {start}
            LabelType::Block {start:_,next} => {next}
            LabelType::If {start:_,else_block:_,next} => { next }
        };

        build_context.builder().build_br(br_block);
    }
    Ok(stack)

}

fn br_if<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,label_index:u32)->Result<Stack<'a,T>,Error>{
    Ok({
        let cond = stack.values.pop().ok_or(NotExistValue)?;
        let br_block = stack.current_function.append_basic_block(build_context.context(),"");
        let nothing_br_block = stack.current_function.append_basic_block(build_context.context(),"");
        build_cond_br(build_context.builder(),build_context.context(),cond.to_value(build_context),br_block,nothing_br_block);
        build_context.builder().position_builder_at_end(br_block);
        let stack =br(build_context,stack,label_index)?;
        build_context.builder().position_builder_at_end(nothing_br_block);
        stack
    })
}

fn br_table<'a,T:WasmIntType>(build_context:&'a BuildContext,mut stack:Stack<'a,T>,label_indexes:Box<[u32]>,label_index:u32)->Result<Stack<'a,T>,Error>{
    Ok({
        {
            let tl = stack.values.pop().ok_or(NotExistValue)?;
            let label_indexes = label_indexes.as_ref();
            for l in label_indexes.iter() {
                let target_label = Value::const_int(Type::int32(build_context.context()), *l as u64, false);
                let cond = build_context.builder().build_icmp(IntPredicate::LLVMIntEQ, tl.to_value(build_context), target_label, "");
                let br_block = stack.current_function.append_basic_block(build_context.context(), "");
                let else_block = stack.current_function.append_basic_block(build_context.context(), "");
                build_cond_br(build_context.builder(),build_context.context(),cond,br_block,else_block);
                build_context.builder().position_builder_at_end(br_block);
                stack = br(build_context, stack, *l)?;
                build_context.builder().position_builder_at_end(else_block);
            }
        }
        let stack = br(build_context,stack,label_index)?;
        stack
    })
}

fn nop<'a,T:WasmIntType>(build_context:&'a BuildContext, stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    build_call_and_set_donothing(build_context.module(),build_context.builder(),"");
    Ok(stack)
}

fn unreachable<'a,T:WasmIntType>(build_context:&'a BuildContext,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    build_call_and_set_trap(build_context.module(),build_context.builder(),"");
    Ok(stack)
}

pub fn progress_instruction<'a,T:WasmIntType>(build_context:&'a BuildContext, instruction:Instruction,stack:Stack<'a,T>)->Result<Stack<'a,T>,Error>{
    let mut stack = match instruction.clone(){
        Instruction::I32Const(v)=> i32_const(build_context, v,stack),
        Instruction::I64Const(v)=> i64_const(build_context, v,stack),
        Instruction::F32Const(v)=> f32_const(build_context, f32_reinterpret_i32(v),stack),
        Instruction::F64Const(v)=> f64_const(build_context, f64_reinterpret_i64(v),stack),
        Instruction::GetGlobal(index)=> get_global(build_context,index,stack),
        Instruction::SetGlobal(index)=> set_global(build_context,index,stack),
        Instruction::GetLocal(index)=>get_local(build_context, index,stack),
        Instruction::SetLocal(index)=>set_local(build_context,index,stack),
        Instruction::TeeLocal(index)=>tee_local(build_context,index,stack),
        Instruction::F32Store(align,offset)=>store(build_context,align,offset,stack,Type::float32(build_context.context())),
        Instruction::F64Store(align,offset)=>store(build_context,align,offset,stack,Type::float64(build_context.context())),
        Instruction::I32Store(align,offset)=>store(build_context,align,offset,stack, Type::int32(build_context.context())),
        Instruction::I64Store(align,offset)=>store(build_context,align,offset,stack,Type::int64(build_context.context())),
        Instruction::I32Store8(align,offset)=>store(build_context,align,offset,stack, Type::int8(build_context.context())),
        Instruction::I32Store16(align,offset)=>store(build_context,align,offset,stack, Type::int16(build_context.context())),
        Instruction::I64Store8(align,offset)=>store(build_context,align,offset,stack, Type::int8(build_context.context())),
        Instruction::I64Store16(align,offset)=>store(build_context,align,offset,stack,Type::int16(build_context.context())),
        Instruction::I64Store32(align,offset)=>store(build_context,align,offset,stack,Type::int32(build_context.context())),
        Instruction::I32Load8S(align,offset)=>load(build_context,align,offset,stack, Type::int8(build_context.context())),
        Instruction::I32Load8U(align,offset)=>load(build_context,align,offset,stack, Type::int8(build_context.context())),
        Instruction::I32Load16S(align,offset)=>load(build_context,align,offset,stack , Type::int16(build_context.context())),
        Instruction::I32Load16U(align,offset)=>load(build_context,align,offset,stack,Type::int16(build_context.context())),
        Instruction::I32Load(align,offset)=>load(build_context,align,offset,stack, Type::int32(build_context.context())),
        Instruction::I64Load(align,offset)=>load(build_context,align,offset,stack, Type::int64(build_context.context())),
        Instruction::I64Load8S(align,offset)=>load(build_context,align,offset,stack,Type::int8(build_context.context())),
        Instruction::I64Load8U(align,offset)=>load(build_context,align,offset,stack,Type::int8(build_context.context())),
        Instruction::I64Load16S(align,offset)=>load(build_context,align,offset,stack,Type::int16(build_context.context())),
        Instruction::I64Load16U(align,offset)=>load(build_context,align,offset,stack, Type::int16(build_context.context())),
        Instruction::I64Load32S(align,offset)=>load(build_context,align,offset,stack,Type::int32(build_context.context())),
        Instruction::I64Load32U(align,offset)=>load(build_context,align,offset,stack,Type::int32(build_context.context())),
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

        Instruction::I32Or => or(build_context,stack),
        Instruction::I64Or => or(build_context,stack),

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
        Instruction::I64ExtendSI32 => extend_s32_to_i64(build_context,stack),
        Instruction::I64ExtendUI32 => extend_u32_to_i64(build_context,stack),

        Instruction::I32WrapI64 => wrap_i64_to_i32(build_context,stack),

        Instruction::I32TruncSF32 => trunc_float_to_s32(build_context,stack),
        Instruction::I32TruncSF64 => trunc_float_to_s32(build_context,stack),
        Instruction::I32TruncUF32 => trunc_float_to_u32(build_context,stack),
        Instruction::I32TruncUF64 => trunc_float_to_u32(build_context,stack),
        Instruction::I64TruncSF32 => trunc_float_to_s64(build_context,stack),
        Instruction::I64TruncSF64 => trunc_float_to_s64(build_context,stack),
        Instruction::I64TruncUF32 => trunc_float_to_u64(build_context,stack),
        Instruction::I64TruncUF64 => trunc_float_to_u64(build_context,stack),

        Instruction::F32DemoteF64 => demote_float(build_context,stack),
        Instruction::F64PromoteF32 => promote_float(build_context,stack),

        Instruction::F32ConvertSI32 => convert_sint_to_f32(build_context,stack),
        Instruction::F32ConvertSI64 => convert_sint_to_f32(build_context,stack),
        Instruction::F32ConvertUI32 => convert_uint_to_f32(build_context,stack),
        Instruction::F32ConvertUI64 => convert_uint_to_f32(build_context,stack),

        Instruction::F64ConvertSI32 => convert_sint_to_f64(build_context,stack),
        Instruction::F64ConvertSI64 => convert_sint_to_f64(build_context,stack),
        Instruction::F64ConvertUI32 => convert_uint_to_f64(build_context,stack),
        Instruction::F64ConvertUI64 => convert_uint_to_f64(build_context,stack),
        Instruction::F32ReinterpretI32 =>reinter_pret_int_to_f32(build_context,stack),
        Instruction::F64ReinterpretI64 => reinter_pret_int_to_f64(build_context,stack),
        Instruction::I32ReinterpretF32 => reinter_pret_float_to_i32(build_context,stack),
        Instruction::I64ReinterpretF64 => reinter_pret_float_to_i64(build_context,stack),
        Instruction::Block(block_type) =>block(build_context,stack,block_type),
        Instruction::Loop(block_type) => loop_instruction(build_context,stack,block_type),
        Instruction::If(block_type) => if_instruction(build_context,stack,block_type),
        Instruction::Br(label_index) => br(build_context,stack,label_index),
        Instruction::BrIf(label_index) => br_if(build_context,stack,label_index),
        Instruction::BrTable(label_indexes,label_index) => br_table(build_context,stack,label_indexes,label_index),
        Instruction::Else => else_instruction(build_context,stack),
        Instruction::End=>end(build_context,stack),
        Instruction::Nop => nop(build_context,stack),
        Instruction::Unreachable => unreachable(build_context,stack),
        instruction=>Err(InvalidInstruction {instruction})?,
    }?;
    {
        let current_frame = stack.activations.current_mut()?;
        current_frame.previous_instruction = Some(instruction);
    }
    Ok(stack)
}
pub fn filter_label_block_types(instructions:Iter<Instruction>)->Vec<ValueType>{
    instructions.filter(|i| i.is_block() ).map(|i| match i {
        Instruction::Loop(block_type) | Instruction::If(block_type) | Instruction::Block(block_type) => block_type,
        _ => panic!("invalid instruction")
    }).filter(|bt| match bt {
        BlockType::Value(_) => true,
        _ => false,
    }).map(|bt| match bt{
        BlockType::Value(value_type) => *value_type,
        _ => panic!("invalid block type"),
    }).collect()
}


#[inline]
fn build_check_memory_size_const<'a,T:WasmIntType>(build_context:&'a BuildContext, index:u32, target:u32, linear_memory_compiler:&LinearMemoryCompiler<T>)->Result<(),Error>{
    build_check_memory_size(build_context,index,Value::const_int(Type::int32(build_context.context()),target as ::libc::c_ulonglong,false),linear_memory_compiler)
}


#[inline]
fn build_check_memory_size<'a,T:WasmIntType>(build_context:&'a BuildContext,index:u32, target:&'a Value, linear_memory_compiler:&LinearMemoryCompiler<T>)->Result<(),Error>{
    let check_function_name = linear_memory_compiler.get_memory_real_check_name(index);
    let check_function = build_context.module().get_named_function(&check_function_name).ok_or(NoSuchLLVMFunction {name:check_function_name})?;
    build_context.builder().build_call(check_function,&[target],"");
    Ok(())
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

fn build_cond_br<'a>(builder:&'a Builder, context:&'a Context, value:&'a Value,then_block:&'a BasicBlock,else_block:&'a BasicBlock)->&'a Value{
    builder.build_cond_br(builder.build_icmp( IntPredicate::LLVMIntNE, value,Value::const_int(Type::type_of(value),0,false),""),then_block,else_block)
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
        build_test_instruction_function(&build_context, test_function_name, vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()), expected, false))], vec![], |stack:Stack<u32>, _bb|{
            let stack = set_global(&build_context,0,stack)?;
            let stack = get_global(&build_context,0,stack)?;
            build_context.builder().build_ret(stack.values.last().ok_or(NotExistValue)?.to_value(&build_context));
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
                                              &[], &[],
                                              &ft,
                                              &lt)
        ], |stack,_bb|{
            let stack = get_local(&build_context,0,stack)?;
            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        build_test_instruction_function(&build_context, test_function_name, vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()), expected, false))], vec![
            frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), 0, false))],
                                              &[], &[],
                                              &ft,&lt)], |stack,_bb|{
            let stack = set_local(&build_context,0,stack)?;
            let stack = get_local(&build_context,0,stack)?;
            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        build_test_instruction_function(&build_context, test_function_name, vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()), expected, false))], vec![

            frame::test_utils::new_test_frame(vec![LocalValue::from_value(Value::const_int(Type::int32(build_context.context()), 0, false))],
                                              &[], &[],
                                              &ft,
                                              &lt)
        ], |stack,_bb|{
            let stack = tee_local(&build_context,0,stack)?;
            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        let  init_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;
        build_test_instruction_function(&build_context, test_function_name, vec![], vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                                                                                                                                  &ft,
                                                                                                                                                                                                  &lt)],
                                        |stack,_bb|{
                                            let stack = progress_instruction(&build_context,Instruction::I32Const(3000),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Store(2,500),stack)?;
                                            let  mut stack = progress_instruction(&build_context,Instruction::I32Load(2,500),stack)?;
                                            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
            Ok(())
        })?;
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
        build_test_instruction_function(&build_context, test_function_name, vec![], vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                                                           &ft,
                                                                                                                           &lt)],
                                        |stack,_bb|{
            let stack = current_memory(&build_context,0,stack)?;
            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),1,false))],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                               &ft,
                                                                               &lt)],|stack,_bb|{

                let stack = grow_memory(&build_context,0,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),4,false))],
                                        vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                               &ft,
                                                                               &lt)],|stack,_bb|{

                let stack = grow_memory(&build_context,0,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),$x as u64,false))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),$x as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),$x as u64,false))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),$x as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),$x as f64))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),$x as f64))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),$lhs as u64,false)),WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),$rhs as u64,false))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),$lhs as u64,false)),WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),$rhs as u64,false))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(  Value::const_int(Type::int32(build_context.context()),$lhs as u64,true)),WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),$rhs as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[], vec![],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),$lhs as u64,true)),WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),$rhs as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),$lhs as f64)),WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),$rhs as f64))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),$lhs as f64)),WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),$rhs as f64))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),$lhs as u64,true)),WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),$rhs as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),$lhs as u64,false)),WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),$rhs as u64,false))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),$lhs as u64,true)),WasmValue::new_value(Value::const_int(Type::int32(build_context.context()),$rhs as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int64(build_context.context()),$lhs as u64,false)),WasmValue::new_value(Value::const_int(Type::int64(build_context.context()),$rhs as u64,false))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int64(build_context.context()),$lhs as u64,true)),WasmValue::new_value(Value::const_int(Type::int64(build_context.context()),$rhs as u64,true))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value(Value::const_real(Type::float32(build_context.context()),$lhs as f64)),WasmValue::new_value(Value::const_real(Type::float32(build_context.context()),$rhs as f64))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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

                build_test_instruction_function(&build_context,test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),$lhs )),WasmValue::new_value(Value::const_real(Type::float64(build_context.context()),$rhs ))],
                                                vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                       &ft,
                                                                                       &lt)],|stack,_|{

                        let stack = progress_instruction(&build_context,$instruction, stack)?;
                        let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        binop_u32_works!(-1_i32 as u32,-1_i32 as u32,3,Instruction::I32Or)
    }


    #[test]
    pub fn or64_works()->Result<(),Error>{
        binop_u64_works!(-1_i64 as u64,-1_i64 as u64,3,Instruction::I64Or)
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
        let expected =0;
        let test_function_name = "test_function";
        let (ft,lt) = new_compilers();
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),22,false))],
            vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                   &ft,
                                                   &lt)],|stack:Stack<u32>,_|{
                let stack = progress_instruction(&build_context,Instruction::I32Eqz,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        let expected =1;
        let test_function_name = "test_function";
        let (ft,lt) = new_compilers();
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),0,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack:Stack<u32>,_|{
                let stack = progress_instruction(&build_context,Instruction::I32Eqz,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        let expected =0;
        let test_function_name = "test_function";
        let (ft,lt) = new_compilers();
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),22,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack:Stack<u32>,_|{
                let stack = progress_instruction(&build_context,Instruction::I64Eqz,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
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
        let expected =1;
        let test_function_name = "test_function";
        let (ft,lt) = new_compilers();
        build_test_instruction_function_with_type(&build_context,Type::int1(build_context.context()),test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),0,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack:Stack<u32>,_|{
                let stack = progress_instruction(&build_context,Instruction::I64Eqz,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;

            assert_eq!(expected,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn wrap_i64_to_i32_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("wrap_i64_to_i32_works",&context);
        let expected = 0x0F_FF_FF_FF;
        let (ft,lt) = new_compilers();
        let test_function_name = "test_function";

        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),0x0F_FF_FF_FF_0F_FF_FF_FF as u64,true))],
        vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                               &ft,
                                               &lt)],|stack,_|{

            let stack = progress_instruction(&build_context,Instruction::I32WrapI64, stack)?;
            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
            Ok(())
        })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) );
            Ok(())
        })
    }

    #[test]
    pub fn extend_s32_to_i64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("extend_s32_to_i64_works",&context);
        let expected = 0xFF_FF_FF_FF_FF_FF_FF_FF;
        let (ft,lt) = new_compilers();
        let test_function_name = "extend_u32_to_i64_works";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),0xFF_FF_FF_FF as u64,true))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I64ExtendSI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) );
            Ok(())
        })
    }


    #[test]
    pub fn extend_u32_to_i64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("extend_u32_to_i64_works",&context);
        let expected = 0xFF_FF_FF_FF;
        let (ft,lt) = new_compilers();
        let test_function_name = "extend_u32_to_i64_works";

        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),0xFF_FF_FF_FF as u64,true))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I64ExtendUI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) );
            Ok(())
        })
    }

    #[test]
    pub fn trunc_f32_to_s32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f32_to_s32_works",&context);
        let (ft,lt) = new_compilers();
        let expected = -3_i32;
        let test_function_name = "trunc_f32_to_s32_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),-3.5_f32 as f64))],
        vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                               &ft,
                                               &lt)],|stack,_|{

            let stack = progress_instruction(&build_context,Instruction::I32TruncSF32, stack)?;
            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
            Ok(())
        })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) as i32 );
            Ok(())
        })
    }

    #[test]
    pub fn trunc_f32_to_s64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f32_to_s64_works",&context);
        let (ft,lt) = new_compilers();
        let expected = -3_i64;
        let test_function_name = "trunc_f32_to_s64_works";
        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),-3.5_f32 as f64))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I64TruncSF32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) as i64 );
            Ok(())
        })
    }


    #[test]
    pub fn trunc_f32_to_u32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f32_to_u32_works",&context);
        let (ft,lt) = new_compilers();
        let expected =3;
        let test_function_name = "trunc_f32_to_u32_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),3.5 as f64))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I32TruncUF32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as u32 );
            Ok(())
        })
    }

    #[test]
    pub fn trunc_f32_to_u64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f32_to_u64_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "trunc_f32_to_u64_works";
        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),3.5 as f64))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I64TruncUF32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }



    #[test]
    pub fn trunc_f64_to_s32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f64_to_s32_works",&context);
        let (ft,lt) = new_compilers();
        let expected = -3_i32;
        let test_function_name = "trunc_f64_to_s32_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),-3.5_f64))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I32TruncSF64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) as i32  );
            Ok(())
        })
    }

    #[test]
    pub fn trunc_f64_to_s64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f64_to_s64_works",&context);
        let (ft,lt) = new_compilers();
        let expected = -3_i64;
        let test_function_name = "trunc_f64_to_s64_works";
        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),-3.5_f64))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I64TruncSF64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(true) as i64 );
            Ok(())
        })
    }


    #[test]
    pub fn trunc_f64_to_u32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f64_to_u32_works",&context);
        let (ft,lt) = new_compilers();
        let expected =3;
        let test_function_name = "trunc_f64_to_u32_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),3.5 as f64))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I32TruncUF64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false) as u32 );
            Ok(())
        })
    }

    #[test]
    pub fn trunc_f64_to_u64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("trunc_f64_to_u64_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "trunc_f64_to_u64_works";
        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),3.5))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I64TruncUF64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }


    #[test]
    pub fn demote_float_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("demote_float_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 5.5;
        let test_function_name = "demote_float_works";
        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()),5.5))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F32DemoteF64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float32(build_context.context())) );
            Ok(())
        })
    }

    #[test]
    pub fn promote_float_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("demote_float_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 5.5;
        let test_function_name = "demote_float_works";
        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),5.5))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F64PromoteF32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float64(build_context.context())) );
            Ok(())
        })
    }


    #[test]
    pub fn convert_s32_to_f32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_s32_to_f32_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f32 = -5.0;
        let test_function_name = "convert_s32_to_f32_works";
        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),-5_i32 as u64,true))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F32ConvertSI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float32(build_context.context())) as f32 );
            Ok(())
        })
    }


    #[test]
    pub fn convert_s64_to_f32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_s64_to_f32_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f32 = -5.0;
        let test_function_name = "convert_s64_to_f32_works";
        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),-5_i64 as u64,true))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F32ConvertSI64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float32(build_context.context())) as f32 );
            Ok(())
        })
    }

    #[test]
    pub fn convert_u32_to_f32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_u32_to_f32_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f32 = 5.0;
        let test_function_name = "convert_u32_to_f32_works";
        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),5 as u64,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F32ConvertUI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float32(build_context.context())) as f32 );
            Ok(())
        })
    }


    #[test]
    pub fn convert_u64_to_f32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_u64_to_f32_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f32 = 5.0;
        let test_function_name = "convert_u64_to_f32_works";
        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()), 5,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F32ConvertUI64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float32(build_context.context())) as f32 );
            Ok(())
        })
    }


    #[test]
    pub fn convert_s32_to_f64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_s32_to_f64_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f64 = -5.0;
        let test_function_name = "convert_s32_to_f64_works";
        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),-5_i32 as u64,true))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F64ConvertSI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float64(build_context.context())) as f64 );
            Ok(())
        })
    }


    #[test]
    pub fn convert_s64_to_f64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_s64_to_f64_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f64 = -5.0;
        let test_function_name = "convert_s64_to_f64_works";
        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()),-5_i64 as u64,true))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F64ConvertSI64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float64(build_context.context())) as f64 );
            Ok(())
        })
    }

    #[test]
    pub fn convert_u32_to_f64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_u32_to_f64_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f64 = 5.0;
        let test_function_name = "convert_u32_to_f64_works";
        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()),5 as u64,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F64ConvertUI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float64(build_context.context())) as f64 );
            Ok(())
        })
    }


    #[test]
    pub fn convert_u64_to_f64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_u64_to_f64_works",&context);
        let (ft,lt) = new_compilers();
        let expected:f64 = 5.0;
        let test_function_name = "convert_u64_to_f64_works";
        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int64(build_context.context()), 5,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F64ConvertUI64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_float(Type::float64(build_context.context())) as f64 );
            Ok(())
        })
    }


    #[test]
    pub fn convert_i32_to_f32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_i32_to_f32_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 65535;
        let test_function_name = "convert_i32_to_f32_works";
        build_test_instruction_function_with_type(&build_context,Type::float32(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_int(Type::int32(build_context.context()), expected,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let  stack = progress_instruction(&build_context,Instruction::F32ReinterpretI32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected as u32,i32_reinterpret_f32(ret.to_float(Type::float32(build_context.context())) as f32));
            Ok(())
        })
    }

    #[test]
    pub fn convert_i64_to_f64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_i64_to_f64_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 65535;
        let test_function_name = "convert_i64_to_f64_works";
        build_test_instruction_function_with_type(&build_context,Type::float64(build_context.context()), test_function_name,vec![WasmValue::new_value(Value::const_int(Type::int64(build_context.context()), expected,false))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::F64ReinterpretI64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,i64_reinterpret_f64(ret.to_float(Type::float64(build_context.context()))));
            Ok(())
        })
    }


    #[test]
    pub fn convert_f32_to_i32_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_f32_to_i32_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 5.0;
        let test_function_name = "convert_f32_to_i32_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![ WasmValue::new_value( Value::const_real(Type::float32(build_context.context()),expected ))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::I32ReinterpretF32, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected as f32,f32_reinterpret_i32(ret.to_int(false) as u32));
            Ok(())
        })
    }

    #[test]
    pub fn convert_f64_to_i64_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("convert_f64_to_i64_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 5.0;
        let test_function_name = "convert_f64_to_i64_works";
        build_test_instruction_function_with_type(&build_context,Type::int64(build_context.context()), test_function_name,vec![WasmValue::new_value( Value::const_real(Type::float64(build_context.context()), expected))],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let mut stack = progress_instruction(&build_context,Instruction::I64ReinterpretF64, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected , f64_reinterpret_i64(ret.to_int(false)));
            Ok(())
        })
    }


    #[test]
    pub fn block_return_i32()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("block_return_i32",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "block_return_i32";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn br_block_return_i32()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("block_return_i32",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "block_return_i32";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![],&[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::NoResult), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::Br(0),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(5),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn br_block_return_nothing()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("block_return_i32",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "block_return_i32";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![],&[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{


                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::NoResult), stack)?;
                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::NoResult), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(1),stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Add,stack)?;
                let stack = progress_instruction(&build_context,Instruction::Br(0),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn br_block_return_triple()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("block_return_i32",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "block_return_i32";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![],&[], &[],
                                                                                                                              &ft,
                                                                                                                              &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::F32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::Br(0),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::F32Const(i32_reinterpret_f32(3.21)),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(22),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine_optional_analysis(build_context.module(),|| Ok(()),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn br_if_block_return_i32()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("block_return_i32",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "block_return_i32";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::NoResult), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(1),stack)?;
                let stack = progress_instruction(&build_context,Instruction::BrIf(0),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(2),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;

        test_module_in_engine_optional_analysis(build_context.module(),||Ok(()),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn loop_return_i32()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("loop_return_i32",&context);
        let expected = 5;
        let (ft,lt) = new_compilers();
        let test_function_name = "loop_return_i32";
        let  init_memory_function_name = memory::test_utils::init_test_memory(&build_context)?;
        build_test_instruction_function(&build_context, test_function_name, vec![], vec![frame::test_utils::new_test_frame(vec![],&[], &[],
                                                                                                                           &ft,
                                                                                                                           &lt)],
                                        |stack,_bb|{
                                            let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;

                                            let stack = progress_instruction(&build_context,Instruction::I32Const(0),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Store(2,500),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::Loop(BlockType::Value(ValueType::I32)), stack)?;

                                            let stack = progress_instruction(&build_context,Instruction::I32Load(2,500),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Const(1),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Add,stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Store(2,500),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Load(2,500),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Load(2,500),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Const(4),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32GtS,stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::BrIf(0),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Const(12),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::Br(1),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::I32Const(2),stack)?;
                                            let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                                            let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                                            Ok(())
                                        })?;

        test_module_in_engine(build_context.module(),|engine|{

            let ret = run_test_function_with_name(engine,build_context.module(),&init_memory_function_name,&[])?;
            assert_eq!(1,ret.to_int(false));

            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));

            Ok(())
        })

    }

    #[test]
    pub fn block_if_else_i32()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("block_if_else_i32",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "block_if_else_i32";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![],&[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{
                let stack = progress_instruction(&build_context,Instruction::I32Const(0),stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Eqz,stack)?;
                let stack = progress_instruction(&build_context,Instruction::If(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::Else,stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(2),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }

    #[test]
    pub fn br_table_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("br_table_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "br_table_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![],&[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{

                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::Block(BlockType::Value(ValueType::I32)), stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(0),stack)?;
                let stack = progress_instruction(&build_context,Instruction::BrTable(Box::new( [0]),1),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let stack = progress_instruction(&build_context,Instruction::I32Const(22),stack)?;
                let stack = progress_instruction(&build_context,Instruction::End,stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })
    }


    #[test]
    pub fn nop_works()-> Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("nop_works",&context);
        let (ft,lt) = new_compilers();
        let expected = 3;
        let test_function_name = "nop_works";
        build_test_instruction_function_with_type(&build_context,Type::int32(build_context.context()), test_function_name,vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)],|stack,_|{


                let stack = progress_instruction(&build_context,Instruction::I32Const(3),stack)?;
                let stack = progress_instruction(&build_context,Instruction::Nop, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })?;
        test_module_in_engine(build_context.module(),|engine|{
            let ret = run_test_function_with_name(engine,build_context.module(),test_function_name,&[])?;
            assert_eq!(expected ,ret.to_int(false));
            Ok(())
        })

    }

    #[test]
    pub fn unreachable_works()-> Result<(),Error> {
        let context = Context::new();
        let build_context = BuildContext::new("unreachable_works", &context);
        let (ft, lt) = new_compilers();
        let expected = 3;
        let test_function_name = "unreachable_works";
        build_test_instruction_function_with_type(&build_context, Type::int32(build_context.context()), test_function_name, vec![],
                                                  vec![frame::test_utils::new_test_frame(vec![], &[], &[],
                                                                                         &ft,
                                                                                         &lt)], |stack, _| {
                let stack = progress_instruction(&build_context, Instruction::I32Const(3), stack)?;
                let stack = progress_instruction(&build_context, Instruction::Unreachable, stack)?;
                let _ = progress_instruction(&build_context,Instruction::End, stack)?;
                Ok(())
            })
        // TODO: this test case is uncompleted. because unreachable send SIGILL. should implement run test.
    }
}