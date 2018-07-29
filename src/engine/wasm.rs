

use parity_wasm::elements::Module as ParityWasmModule;
use super::llvm::*;
use super::types::*;
use super::linear_memory::*;
use failure::Error;
use std::str;
use parity_wasm::elements::{DataSegment,Instruction,External,GlobalType,ValueType,GlobalEntry};
use error::RuntimeError::*;

pub const WASM_FUNCTION_PREFIX:&str = "__WASM_FUNCTION_";
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


    pub fn compile<'c>(&self, module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {
        let builder = Builder::new(context);
        let module = Module::new(module_id, context);
        self.build_init_data_sections_function(wasm_module,&module,&builder)?;
        Ok(module)
    }

    pub fn set_init_data_sections_function<'c>(&self,module:&'c Module)->&'c Value{
        let context = module.context();
        let param_types:[&Type;0] = [];
        module.set_function("init_data_sections",Type::function(Type::int8(context),&param_types,true))
    }

    fn build_init_global_sections(&self,wasm_module:&ParityWasmModule,module:&Module,builder:&Builder)->Result<(),Error>{
        let import_global_count = wasm_module.import_section().map_or(0,|section|{
           section.entries().iter().filter(|entry|match entry.external() {
               External::Global(_)=>true,
               _=>false,
           }).count()
        });
        wasm_module.global_section().map_or(Ok(()),|section|{

            for (index,entry) in section.entries().iter().enumerate(){
                self.set_global(module,index + import_global_count,entry.global_type());
            }
            Ok(())
        })
    }

    fn build_global_entry<'a>(& self,module:&'a Module,builder:&Builder,index:usize,entry:&GlobalEntry)->Result<(),Error>{
        let global_value = self.set_global(module,index,&entry.global_type());
        let context = module.context();
        let instruction = entry.init_expr().code().first().ok_or(NotExistGlobalInitializerInstruction)?;
        let initialize_value = self.build_instruction(module,builder,instruction);
        global_value.set_initializer(initialize_value);
        global_value.set_global_const( entry.global_type().is_mutable());
        Ok(())
    }

    fn set_global<'a>(& self,module:&'a Module,index:usize,global_type:&GlobalType)->&'a Value{

        module.set_global([WASM_GLOBAL_PREFIX,index.to_string().as_ref()].concat().as_ref(),Self::value_type_to_type(&global_type.content_type(),module.context()))
    }

    fn value_type_to_type<'a>(value_type:&'a ValueType,context:&'a Context)->&'a Type{
        match value_type{
            ValueType::I32 => Type::int32(context),
            ValueType::I64 => Type::int64(context),
            ValueType::F32 => Type::float32(context),
            ValueType::F64 => Type::float64(context),
        }
    }

    fn build_instruction<'a>(&self,module:&'a Module,builder:&'a Builder,instruction:&Instruction)->&'a Value{
        let context = module.context();
        match instruction{
            Instruction::I64Const(v) => Value::const_int(Type::int64(context),*v as u64,true),
            Instruction::I32Const(v) => Value::const_int(Type::int32(context),*v as u64,true),
            Instruction::F32Const(v) => Value::const_real(Type::float32(context),f32_reinterpret_i32(*v as i32) as ::libc::c_double),
            Instruction::F64Const(v) => Value::const_real(Type::float64(context),f64_reinterpret_i64(*v as i64) as ::libc::c_double),
            _ => Value::const_int(Type::int64(context),0,true),
        }
    }

    fn build_init_data_sections_function(&self,wasm_module:&ParityWasmModule,module:&Module,builder:&Builder)->Result<(),Error>{
        let function = self.set_init_data_sections_function(module);
        let context = module.context();
        builder.build_function(context,function,|builder,bb|{
            wasm_module.data_section().map_or(Ok(()),|data_section|{
               for segment in data_section.entries(){
                    self.build_data_segment(segment,module,builder)?;
               }
                Ok(())
            })
        })
    }

    fn build_data_segment(&self,segment:&DataSegment,module:&Module,builder:&Builder)->Result<(),Error>{
        let context = module.context();

        let convert_offset_const = |v:u64|{
            Value::const_int(Type::int_wasm_ptr::<T>(context),v,true)
        };
        let instruction = segment.offset().code().first().ok_or(NotExistDataSectionOffset)?;
        let offset = self.build_instruction(module,builder,instruction);
        let dest = self.linear_memory_compiler.build_get_real_address(module,builder,offset,"dest",segment.index() as usize);
        let c_args = segment.value().iter().map(|v|Value::const_int(Type::int8(context),*v as ::libc::c_ulonglong,false)).collect::<Vec<&Value>>();
        let src = Value::const_vector( &c_args);
        let n = Value::const_int(Type::int_ptr(context),segment.value().len() as u64,false);
        build_call_and_set_memcpy(module,builder,dest,src,n,"");
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
        let context =  Context::new();
        let (compiler,module,builder) = init_test(&context);
    }

    fn init_test<'a>(context:&'a Context)->(WasmCompiler<u32>,ModuleGuard<'a>,BuilderGuard<'a>){
        let compiler = WasmCompiler::<u32>::new();
        let module = Module::new("wasm_test_module",context);
        let builder = Builder::new(&context);
        (compiler,module,builder)
    }
}