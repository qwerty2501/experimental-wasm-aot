

use parity_wasm::elements::Module as WasmModule;
use super::*;
use failure::Error;
use std::str;
use parity_wasm::elements::{DataSegment,Instruction,ImportCountType,GlobalType,ValueType,GlobalEntry,External};
use parity_wasm::elements;
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
        self.build_init_global_sections(&build_context,wasm_module)?;
        self.build_init_data_sections_function(&build_context,wasm_module)?;
        Ok(build_context.move_module())
    }

    pub fn set_declare_init_module_function<'c>(&self, build_context:&'c BuildContext) ->&'c Value{
        let void_type = Type::void(build_context.context());
        build_context.module().set_declare_function("init_module", Type::function(void_type, &[void_type], false))
    }

    pub fn set_declare_init_data_sections_function<'c>(&self, build_context:&'c BuildContext) ->&'c Value{
        build_context.module().set_declare_function("init_data_sections", Type::function(Type::int8(build_context.context()), & [], false))
    }

    fn build_init_global_sections(&self,build_context:&BuildContext,wasm_module:&WasmModule )->Result<(),Error>{
        let import_global_count = wasm_module.import_count(ImportCountType::Global) as u32;
        wasm_module.global_section().map_or(Ok(()),|section|{
            self.build_global_entries(build_context,section.entries(),import_global_count)
        })
    }

    fn build_global_entries(&self,build_context:&BuildContext,entries:&[GlobalEntry],import_global_count:u32)->Result<(),Error>{
        for (index,entry) in entries.iter().enumerate(){
            self.build_const_initialize_global( build_context,index as u32 + import_global_count, entry)?;
        }
        Ok(())
    }

    fn build_functions(&self,build_context:&BuildContext,wasm_module:&WasmModule)->Result<(),Error>{
        wasm_module.type_section().map_or(Ok(()),|type_section|{
            let types = self.set_declare_types(build_context,type_section.types());
            let import_functions:Vec<&Value> = wasm_module.import_section().map_or(Ok(vec![]),|import_section|{
                import_section.entries().iter().filter_map(|import_entry |{
                    if let External::Function(type_index) = import_entry.external(){
                        Some(self.set_declare_function(build_context,import_entry.field(),*type_index,&types))
                    } else{
                        None
                    }
                }).collect()
            })?;


            Ok(())
        })
    }

    fn set_declare_function<'a>(&self,build_context:&'a BuildContext,name:&str,type_index:u32,types:&[&Type])->Result<&'a Value,Error>{
        let function_type = types.get(type_index as usize).ok_or(NoSuchTypeIndex{index:type_index})?;
        Ok(build_context.module().set_declare_function(&Self::function_name_to_wasm_function_name(name),function_type))
    }
    fn set_declare_types<'a>(&self,build_context:&'a BuildContext,types:&[elements::Type])->Vec<&'a Type>{
        types.iter().map(|ty|{
            match ty {
                elements::Type::Function(function_type)=>{
                    let param_types = function_type.params().iter().map(|value_type|Self::value_type_to_type(build_context,&value_type)).collect::<Vec<_>>();
                    Type::function(
                        function_type.return_type().map(|value_type|Self::value_type_to_type(build_context,&value_type)).unwrap_or(Type::void(build_context.context())),
                        &param_types,
                        false
                    )
                }
            }
        }).collect()
    }
    fn function_name_to_wasm_function_name(name:&str)->String{
        ["WASM_FUNCTION_",name].concat()
    }


    fn build_const_initialize_global<'a>(&self, build_context:&'a BuildContext,index:u32, global_entry:&GlobalEntry)->Result<&'a Value,Error>{
        let g = self.set_declare_global(build_context,index, global_entry.global_type());
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


    fn set_declare_global<'a>(& self, build_context:&'a BuildContext, index:u32, global_type:&GlobalType) ->&'a Value{
        build_context.module().set_declare_global(instructions::get_global_name(index).as_ref(), Self::value_type_to_type(build_context,&global_type.content_type()))
    }

    fn value_type_to_type<'a>(build_context:&'a BuildContext, value_type:&ValueType)->&'a Type{
        match value_type{
            ValueType::I32 => Type::int32(build_context.context()),
            ValueType::I64 => Type::int64(build_context.context()),
            ValueType::F32 => Type::float32(build_context.context()),
            ValueType::F64 => Type::float64(build_context.context()),
        }
    }

    fn build_init_data_sections_function(&self,build_context:&BuildContext,wasm_module:&WasmModule)->Result<(),Error>{
        let function = self.set_declare_init_data_sections_function(build_context);
        build_context.builder().build_function(build_context.context(),function,|builder,bb|{
            wasm_module.data_section().map_or(Ok(()),|data_section|{
               for segment in data_section.entries(){
                    self.build_data_segment(build_context,segment)?;
               }
                Ok(())
            })
        })
    }

    fn build_data_segment(&self,build_context:&BuildContext,segment:&DataSegment)->Result<(),Error>{
        let instruction = segment.offset().code().first().ok_or(NotExistDataSectionOffset)?;
        let offset = match instruction {
            Instruction::I64Const(v)=>Ok(instructions::i64_const(build_context,*v)),
            Instruction::I32Const(v)=>Ok(instructions::i32_const(build_context,*v )),
            Instruction::GetGlobal(v)=>instructions::get_global(build_context,*v ),
            invalid_instruction => Err(InvalidInstruction {instruction:invalid_instruction.clone()})?,
        }?;
        let dest = self.linear_memory_compiler.build_get_real_address(build_context,segment.index(),offset,"");

        let int8 = Type::int8(build_context.context());
        let c_args = segment.value().iter().map(|v|Value::const_int(int8,*v as ::libc::c_ulonglong,false)).collect::<Vec<&Value>>();
        let src = Value::const_array( int8,&c_args);
        let dest  = build_context.builder().build_pointer_cast(dest, Type::ptr(Type::type_of(src),0),"");
        build_context.builder().build_store(src,dest);

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
    use parity_wasm::elements::ResizableLimits;

    #[test]
    pub fn build_global_entries_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_global_entries_works",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_global_entries( &build_context,&[
            GlobalEntry::new(GlobalType::new(ValueType::I32,false),InitExpr::new(vec![
                Instruction::I32Const(33),
            ]))
        ],0)?;
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
        compiler.build_const_initialize_global(&build_context,0,&GlobalEntry::new(GlobalType::new(ValueType::I32,false),InitExpr::new(vec![
            Instruction::I32Const(22),
        ])),)?;
        test_initializer(get_global(&build_context,0)?,22,true,|initializer|initializer.const_int_get_sign_extended_value());
        Ok(())
    }

    #[test]
    pub fn build_const_initialize_global_works_i64()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_i64",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(&build_context,0,&GlobalEntry::new(GlobalType::new(ValueType::I64,true),InitExpr::new(vec![
            Instruction::I64Const(5667),
        ])),)?;
        test_initializer(get_global(&build_context,0)?,5667,false,|initializer|initializer.const_int_get_sign_extended_value());
        Ok(())
    }

    #[test]
    pub fn build_const_initialize_global_works_f32()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_f32",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(&build_context,0,&GlobalEntry::new(GlobalType::new(ValueType::F32,false),InitExpr::new(vec![
            Instruction::F32Const(i32_reinterpret_f32(4.00) as u32),
        ])),)?;

        test_initializer(get_global(&build_context,0)?,4.00,true,|initializer|{
            initializer.const_real_get_double().result
        });

        Ok(())
    }

    #[test]
    pub fn build_const_initialize_global_works_f64()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_const_initialize_global_works_f64",&context);
        let compiler = WasmCompiler::<u32>::new();
        compiler.build_const_initialize_global(&build_context,0,&GlobalEntry::new(GlobalType::new(ValueType::F64,true),InitExpr::new(vec![
            Instruction::F64Const(i64_reinterpret_f64(4.00) as u64),
        ])),)?;

        test_initializer(get_global(&build_context,0)?,4.00,false,|initializer|{
            initializer.const_real_get_double().result
        });

        Ok(())
    }

    #[test]
    pub fn build_data_segment_works()->Result<(),Error>{

        let context = Context::new();

        let build_context = BuildContext::new("build_data_segment_works",&context);

        let compiler = WasmCompiler::<u32>::new();
        compiler.linear_memory_compiler.build_init_function(&build_context, 0, &[&ResizableLimits::new(17, Some(25))])?;

        let offset = 1024;
        let expected_values:Vec<u8> =vec![221, 22, 254];
        let data_segment = DataSegment::new(0,InitExpr::new(vec![
            Instruction::I32Const(offset),
        ]),expected_values.clone());

        let function_name = "build_data_segment_works";
        build_test_function(&build_context,function_name,&[],|builder,bb|{
            compiler.build_data_segment(&build_context,&data_segment,)?;
            build_context.builder().build_ret_void();
            Ok(())
        })?;

        analysis::verify_module(build_context.module(),analysis::VerifierFailureAction::LLVMPrintMessageAction)?;

        test_module_in_engine(build_context.module(),|engine|{

            let result = run_test_function_with_name(&engine, build_context.module(), &compiler.linear_memory_compiler.get_init_function_name(), &[])?;
            assert_eq!(1,result.int_width());
            run_test_function_with_name(engine,build_context.module(),function_name,&[])?;
            let linear_memory =  engine.get_global_value_ref_from_address::<*mut u8>(&compiler.linear_memory_compiler.get_memory_name(0));
            for (index,expected) in expected_values.iter().enumerate(){
                assert_eq!(*expected,unsafe{*linear_memory.add(offset as usize +index)});
            }
            Ok(())
        })?;

        Ok(())
    }


}