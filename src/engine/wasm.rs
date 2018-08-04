

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
        self.build_init_global_sections(wasm_module,&build_context)?;
        self.build_init_data_sections_function(wasm_module,&build_context)?;
        Ok(build_context.move_module())
    }

    pub fn set_declare_init_module_function<'c>(&self, build_context:&'c BuildContext) ->&'c Value{
        let void_type = Type::void(build_context.context());
        build_context.module().set_declare_function("init_module", Type::function(void_type, &[void_type], false))
    }

    pub fn set_declare_init_data_sections_function<'c>(&self, build_context:&'c BuildContext) ->&'c Value{
        build_context.module().set_declare_function("init_data_sections", Type::function(Type::int8(build_context.context()), & [], false))
    }

    fn build_init_global_sections(&self,wasm_module:&WasmModule, build_context:&BuildContext)->Result<(),Error>{
        let import_global_count = wasm_module.import_section().map_or(0,|section|{
           section.entries().iter().filter(|entry|is_match_case!( entry.external(),External::Global(_))).count() as u32
        });
        wasm_module.global_section().map_or(Ok(()),|section|{
            self.build_global_entries(section.entries(),import_global_count,build_context)
        })
    }

    fn build_global_entries(&self,entries:&[GlobalEntry],import_global_count:u32,build_context:&BuildContext)->Result<(),Error>{
        for (index,entry) in entries.iter().enumerate(){
            self.build_const_initialize_global( index as u32 + import_global_count, entry,build_context)?;
        }
        Ok(())
    }

    fn build_const_initialize_global<'a>(&self, index:u32, global_entry:&GlobalEntry,build_context:&'a BuildContext)->Result<&'a Value,Error>{
        let g = self.set_declare_global(index, global_entry.global_type(), build_context);
        let instruction = global_entry.init_expr().code().first().ok_or(NotExistGlobalInitializerInstruction)?;
        match instruction{
            Instruction::I32Const(v) => Some(instructions::i32_const(build_context,*v)),
            Instruction::I64Const(v)=>Some(instructions::i64_const(build_context,*v)),
            Instruction::F32Const(v) => Some(instructions::f32_const(build_context,f32_reinterpret_i32(*v as i32))),
            Instruction::F64Const(v)=>Some(instructions::f64_const(build_context,f64_reinterpret_i64(*v as i64))),
            _=>None,

        }.map(|const_initializer|{
            g.set_initializer(const_initializer);
            g.set_global_const(!global_entry.global_type().is_mutable());
        });
        Ok(g)
    }


    fn set_declare_global<'a>(& self, index:u32, global_type:&GlobalType, build_context:&'a BuildContext) ->&'a Value{
        build_context.module().set_declare_global(instructions::get_global_name(index).as_ref(), Self::value_type_to_type(&global_type.content_type(), build_context.context()))
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
        let function = self.set_declare_init_data_sections_function(build_context);
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
    use parity_wasm::elements::GlobalSection;
    use parity_wasm::elements::Section;
    use parity_wasm::elements::InitExpr;

    #[test]
    pub fn build_global_entries_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_global_entries_works",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_global_entries( &[
            GlobalEntry::new(GlobalType::new(ValueType::I32,false),InitExpr::new(vec![
                Instruction::I32Const(33),
            ]))
        ],0,&build_context)?;
        test_initializer(get_global(&build_context,0)?,33,true,|initializer|initializer.const_int_get_sign_extended_value());
        Ok(())

    }

    fn test_initializer<T:WasmNumberType  + PartialEq + ::std::fmt::Debug,F:FnOnce(&Value)->T>(value:&Value,expected:T,constant:bool,actual_func:F){
        let initializer = value.get_initializer();
        assert!(initializer.is_some());
        let initializer= initializer.unwrap();
        assert_eq!(expected,actual_func(initializer));
        assert_eq!(constant,value.is_global_const());
    }

    fn get_global<'a>(build_context:&'a BuildContext,index:u32)->Result<&'a Value,Error>{
        let global_name = instructions::get_global_name(0);
        Ok(build_context.module().get_named_global(global_name.as_ref()).ok_or(NoSuchLLVMGlobalValue {name:global_name})?)
    }

    #[test]
    pub fn build_const_initialize_global_works_i32()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_i32",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(0,&GlobalEntry::new(GlobalType::new(ValueType::I32,false),InitExpr::new(vec![
            Instruction::I32Const(22),
        ])),&&build_context)?;
        test_initializer(get_global(&build_context,0)?,22,true,|initializer|initializer.const_int_get_sign_extended_value());
        Ok(())
    }

    #[test]
    pub fn build_const_initialize_global_works_i64()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_i64",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(0,&GlobalEntry::new(GlobalType::new(ValueType::I64,true),InitExpr::new(vec![
            Instruction::I64Const(5667),
        ])),&&build_context)?;
        test_initializer(get_global(&build_context,0)?,5667,false,|initializer|initializer.const_int_get_sign_extended_value());
        Ok(())
    }

    #[test]
    pub fn build_const_initialize_global_works_f32()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_f32",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(0,&GlobalEntry::new(GlobalType::new(ValueType::F32,false),InitExpr::new(vec![
            Instruction::F32Const(i32_reinterpret_f32(4.00) as u32),
        ])),&&build_context)?;

        test_initializer(get_global(&build_context,0)?,4.00,true,|initializer|{
            let mut loses_info = false;
            initializer.const_real_get_double().result
        });

        Ok(())
    }

    #[test]
    pub fn build_const_initialize_global_works_f64()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_f64",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(0,&GlobalEntry::new(GlobalType::new(ValueType::F64,true),InitExpr::new(vec![
            Instruction::F64Const(i64_reinterpret_f64(4.00) as u64),
        ])),&&build_context)?;

        test_initializer(get_global(&build_context,0)?,4.00,false,|initializer|{
            let mut loses_info = false;
            initializer.const_real_get_double().result
        });

        Ok(())
    }

    #[test]
    pub fn build_data_segment_works()->Result<(),Error>{

        let context = Context::new();

        let build_context = BuildContext::new("build_data_segment_works",&context);
        Ok(())
    }


}