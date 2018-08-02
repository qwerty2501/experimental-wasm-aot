

use parity_wasm::elements::Module as WasmModule;
use super::*;
use failure::Error;
use std::str;
use parity_wasm::elements::{DataSegment,Instruction,External,GlobalType,ValueType,GlobalEntry};
use error::RuntimeError::*;

const WASM_FUNCTION_PREFIX:&str = "__WASM_FUNCTION_";
const WASM_GLOBAL_PREFIX:&str = "__WASM_GLOBAL_";
pub struct WasmCompiler<T: WasmIntType>{
    linear_memory_compiler: LinearMemoryCompiler<T>
}

impl<T:WasmIntType> WasmCompiler<T>{

    pub fn new()->WasmCompiler<T>{
        WasmCompiler{ linear_memory_compiler: LinearMemoryCompiler::<T>::new()}
    }
    fn wasm_call_name(name:&str) ->String{
        [WASM_FUNCTION_PREFIX,name].concat()
    }


    pub fn compile<'c>(&self, module_id:&str,wasm_module:&WasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {
        let build_context = BuildContext::new(module_id,context);
        self.build_init_data_sections_function(wasm_module,&build_context)?;
        Ok(build_context.move_module())
    }

    pub fn set_init_module_function<'c>(&self,build_context:&'c BuildContext)->&'c Value{
        let void_type = Type::void(build_context.context());
        build_context.module().set_function("init_module",Type::function(void_type,&[void_type],false))
    }

    pub fn set_init_data_sections_function<'c>(&self,build_context:&'c BuildContext)->&'c Value{
        build_context.module().set_function("init_data_sections",Type::function(Type::int8(build_context.context()),& [],false))
    }

    fn build_init_global_sections(&self,wasm_module:&WasmModule,build_context:&BuildContext)->Result<(),Error>{
        let import_global_count = wasm_module.import_section().map_or(0,|section|{
           section.entries().iter().filter(|entry|is_match_case!( entry.external(),External::Global(_))).count() as u32
        });
        wasm_module.global_section().map_or(Ok(()),|section|{

            for (index,entry) in section.entries().iter().enumerate(){
                self.build_const_initialize_global(build_context, index as u32 + import_global_count, entry)?;
            }
            Ok(())
        })
    }

    fn build_const_initialize_global<'a>(&self, build_context:&'a BuildContext, index:u32, global_entry:&GlobalEntry)->Result<&'a Value,Error>{
        let g = self.declare_global(build_context,index,global_entry.global_type());
        let instruction = global_entry.init_expr().code().first().ok_or(NotExistGlobalInitializerInstruction)?;
        match instruction{
            Instruction::I32Const(v) => Some(instructions::i32_const(build_context,*v)),
            Instruction::I64Const(v)=>Some(instructions::i64_const(build_context,*v)),
            Instruction::F32Const(v) => Some(instructions::f32_const(build_context,f32_reinterpret_i32(*v as i32))),
            Instruction::F64Const(v)=>Some(instructions::f64_const(build_context,f64_reinterpret_i64(*v as i64))),
            _=>None,

        }.map(|const_initializer|{
            g.set_initializer(const_initializer);
            g.set_global_const(global_entry.global_type().is_mutable());
        });
        Ok(g)
    }


    fn declare_global<'a>(& self, build_context:&'a BuildContext, index:u32, global_type:&GlobalType) ->&'a Value{
        build_context.module().set_global(instructions::get_global_name(index).as_ref(),Self::value_type_to_type(&global_type.content_type(),build_context.context()))
    }

    fn value_type_to_type<'a>(value_type:&'a ValueType,context:&'a Context)->&'a Type{
        match value_type{
            ValueType::I32 => Type::int32(context),
            ValueType::I64 => Type::int64(context),
            ValueType::F32 => Type::float32(context),
            ValueType::F64 => Type::float64(context),
        }
    }

    fn build_init_data_sections_function(&self,wasm_module:&WasmModule,build_context:&BuildContext)->Result<(),Error>{
        let function = self.set_init_data_sections_function(build_context);
        build_context.builder().build_function(build_context.context(),function,|builder,bb|{
            wasm_module.data_section().map_or(Ok(()),|data_section|{
               for segment in data_section.entries(){
                    self.build_data_segment(segment,build_context)?;
               }
                Ok(())
            })
        })
    }

    fn build_data_segment(&self,segment:&DataSegment,build_context:&BuildContext)->Result<(),Error>{
        let instruction = segment.offset().code().first().ok_or(NotExistDataSectionOffset)?;
        let offset = match instruction {
            Instruction::I64Const(v)=>Ok(instructions::i64_const(build_context,*v)),
            Instruction::I32Const(v)=>Ok(instructions::i32_const(build_context,*v )),
            Instruction::GetGlobal(v)=>instructions::get_global(build_context,*v ),
            invalid_instruction => Err(InvalidInstruction {instruction:invalid_instruction.clone()})?,
        }?;
        let dest = self.linear_memory_compiler.build_get_real_address(build_context,offset,"dest",segment.index() as usize);
        let c_args = segment.value().iter().map(|v|Value::const_int(Type::int8(build_context.context()),*v as ::libc::c_ulonglong,false)).collect::<Vec<&Value>>();
        let src = Value::const_vector( &c_args);
        let n = Value::const_int(Type::int_ptr(build_context.context()),segment.value().len() as u64,false);
        build_call_and_set_memcpy(build_context.module(),build_context.builder(),dest,src,n,"");
        Ok(())
    }
}


#[inline]
fn i32_reinterpret_f32(v: f32) -> i32 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[inline]
fn i64_reinterpret_f64(v: f64) -> i64 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[inline]
fn f32_reinterpret_i32(v: i32) -> f32 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[inline]
fn f64_reinterpret_i64(v: i64) -> f64 {
    unsafe {
        ::std::mem::transmute(v)
    }
}

#[cfg(test)]
mod tests{

    use super::*;
    #[test]
    pub fn build_data_segment_works(){

        let context = Context::new();

        let build_context = BuildContext::new("build_data_segment_works",&context);
    }


}