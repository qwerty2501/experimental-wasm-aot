

use parity_wasm::elements::Module as WasmModule;
use super::*;
use failure::Error;
use std::str;
use parity_wasm::elements::{DataSegment,Instruction,ImportCountType,GlobalType,GlobalEntry,External};
use parity_wasm::elements;
use error::RuntimeError::*;
use parity_wasm::elements::Internal;
use parity_wasm::elements::InitExpr;
const WASM_FUNCTION_PREFIX:&str = "__WASM_FUNCTION_";
const WASM_GLOBAL_PREFIX:&str = "__WASM_GLOBAL_";
pub struct WasmCompiler<T: WasmIntType>{
    linear_memory_compiler: LinearMemoryCompiler<T>,
    table_compiler:FunctionTableCompiler<T>,
}

impl<T:WasmIntType> WasmCompiler<T>{

    pub fn new()->WasmCompiler<T>{
        WasmCompiler{ linear_memory_compiler: LinearMemoryCompiler::<T>::new(),table_compiler:FunctionTableCompiler::<T>::new()}
    }
    fn wasm_function_name(name:&str) ->String{
        [WASM_FUNCTION_PREFIX,name].concat()
    }


    pub fn compile<'c>(&self,module_id:&str,wasm_module:&WasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error>{
        let build_context = BuildContext::new(module_id,context);
        {
            self.build_main_function(&build_context, module_id,wasm_module,Self::set_declare_main_function(&build_context),||{
                let entry_function_name = Self::wasm_function_name("wasm_main");
                let entry_function = build_context.module().get_named_function(&entry_function_name).ok_or(NoSuchLLVMFunction {name:entry_function_name})?;
                build_context.builder().build_ret(build_context.builder().build_call(entry_function,&[],""));
                Ok(())
            })?;
        }

        Ok(build_context.move_module())

    }

    fn set_declare_main_function<'a>(build_context:&'a BuildContext)->&'a Value{
        let int32_type = Type::int32(build_context.context());
        let str_ptr_type = Type::ptr(Type::int8(build_context.context()),0);
        let main_function_type = Type::function(int32_type,&[int32_type,str_ptr_type],false);
        build_context.module().set_declare_function("main",main_function_type)
    }

    pub fn build_init_instance_functions<'c>(&self, _module_id:&str, build_context:&BuildContext<'c>, wasm_module:&WasmModule) ->Result<(),Error> {
        self.linear_memory_compiler.compile(build_context,wasm_module)?;
        self.build_init_global_sections(build_context,wasm_module)?;
        self.build_init_data_sections_function(build_context,wasm_module)?;
        self.build_functions(&build_context,wasm_module)?;
        Ok(())
    }

    fn build_main_function<F:FnOnce()->Result<(),Error>>(&self, build_context:&BuildContext,module_id:&str, wasm_module:&WasmModule,main_function:&Value,on_build_entry:F) -> Result<(),Error>{

        self.build_init_instance_functions(module_id, build_context, wasm_module)?;
        build_context.builder().build_function(build_context.context(),main_function,|builder,_bb|{
            let failed_init_block = main_function.append_basic_block(build_context.context(),"");
            let init_table_block = main_function.append_basic_block(build_context.context(),"");
            let init_memory_function_name = self.linear_memory_compiler.get_init_function_name();

            if let Some(init_memory_function) = build_context.module().get_named_function(&init_memory_function_name){
                let ret_init_memory = builder.build_call(init_memory_function,&[],"");
                builder.build_cond_br(ret_init_memory,init_table_block,failed_init_block);
            } else{
                builder.build_br(init_table_block);
            }



            builder.position_builder_at_end(failed_init_block);
            let void_type = Type::void(build_context.context());
            let abort_function_type = Type::function(void_type,&[],false);
            let abort_function = build_context.module().set_declare_function("abort",abort_function_type);
            builder.build_call(abort_function,&[],"");
            builder.build_ret(Value::const_int(Type::int32(build_context.context()),-1_i64 as u64,true));


            builder.position_builder_at_end(init_table_block);
            let init_table_function_name = self.table_compiler.get_init_function_name();
            let init_data_section_block = main_function.append_basic_block(build_context.context(),"");
            if let Some(init_table_function) = build_context.module().get_named_function(&init_table_function_name){
                let ret_init_table = builder.build_call(init_table_function,&[],"");
                builder.build_cond_br(ret_init_table,init_data_section_block,failed_init_block);
            } else{
                builder.build_br(init_data_section_block);
            }


            builder.position_builder_at_end(init_data_section_block);
            if let Some(init_data_sections_function) = build_context.module().get_named_function(Self::INIT_DATA_SECTIONS_NAME){
                builder.build_call(init_data_sections_function,&[],"");
            }

            on_build_entry()
        })
    }

    const INIT_DATA_SECTIONS_NAME:&'static str = "init_data_sections";
    pub fn set_declare_init_data_sections_function<'c>(&self, build_context:&'c BuildContext) ->&'c Value{
        build_context.module().set_declare_function(Self::INIT_DATA_SECTIONS_NAME, Type::function(Type::int8(build_context.context()), & [], false))
    }

    fn build_init_global_sections(&self,build_context:&BuildContext,wasm_module:&WasmModule )->Result<(),Error>{
        let import_global_count = wasm_module.import_count(ImportCountType::Global) as u32;
        if let Some(section) = wasm_module.global_section(){
            self.build_global_entries(build_context,section.entries(),import_global_count)
        } else{
            Ok(())
        }
    }

    fn build_global_entries(&self,build_context:&BuildContext,entries:&[GlobalEntry],import_global_count:u32)->Result<(),Error>{
        for (index,entry) in entries.iter().enumerate(){
            self.build_const_initialize_global( build_context,index as u32 + import_global_count, entry)?;
        }
        Ok(())
    }

    fn build_functions(&self,build_context:&BuildContext,wasm_module:&WasmModule)->Result<(),Error>{
        if let Some(type_section) = wasm_module.type_section(){
            let types = self.set_declare_types(build_context,type_section.types());

            let imported_functions = self.set_declare_imported_functions(build_context, wasm_module, &types)?;

            let exported_function_pairs = self.get_exported_function_pairs(build_context,wasm_module);

            let imported_count= imported_functions.len() as u32;

            let current_module_functions = self.set_declare_current_module_functions(build_context, wasm_module, &exported_function_pairs, &types, imported_count)?;

            let functions:Vec<&Value> = [(&imported_functions) as &[&Value],(&current_module_functions) as &[&Value]].concat();

            self.build_init_table_function(build_context,wasm_module,&functions,imported_count)?;

            self.build_function_codes(build_context,wasm_module,&functions,&types,imported_count)?;
        }
        Ok(())
    }

    fn build_function_codes(&self,build_context:&BuildContext,wasm_module:&WasmModule,functions:&[&Value],types:&[&Type],import_count:u32)->Result<(),Error>{
        if let Some(code_section) = wasm_module.code_section(){
            for (index,function_body) in code_section.bodies().iter().enumerate() {
                let index = index +import_count as usize;
                let current_function:&Value = functions.get(index).ok_or(NoSuchFunctionIndex {index :index as u32 })?;

                let locals= (0..current_function.count_params()).map(|i|->Result<LocalValue,Error> {
                    Ok(LocalValue::from_value(current_function.get_param(i).ok_or(NotExistValue)?))
                }).into_iter().chain(function_body.locals().iter().map(|local|{
                    Ok(LocalValue::from_value_type(instructions::value_type_to_type(build_context,&local.value_type())))
                })).collect::<Result<Vec<LocalValue>,Error>>()?;

                build_context.builder().build_function(build_context.context(),current_function,|builder,_bb|{
                    let label_block_types = instructions::filter_label_block_types(function_body.code().elements().iter());
                    let stack = Stack::new(current_function,vec![],vec![],vec![
                        Frame::new(locals,BlockReturnValue::from_block_types(build_context,&label_block_types) ,ModuleInstance::new(types,functions,&self.table_compiler,&self.linear_memory_compiler))
                    ]);

                    let  stack = function_body.code().elements().iter().try_fold(stack,|stack,instruction|{
                        instructions::progress_instruction(build_context,instruction.clone(),stack)
                    })?;

                    if let Some(result) = stack.values.last(){
                        builder.build_ret(result.to_value(build_context));
                    } else{
                        builder.build_ret_void();
                    }

                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    fn set_declare_current_module_functions<'a>(&self, build_context:&'a BuildContext, wasm_module:&WasmModule, exported_function_pairs:&[(u32, &str)], types:&[&Type], imported_count:u32) ->Result<Vec<&'a Value>,Error>{
        if let Some(function_section) = wasm_module.function_section(){
            function_section.entries().iter().enumerate().map(|(index, function_entry)|{
                let function_index = imported_count + index as u32;
                let internal_name = ["internal",&function_index.to_string()].concat();
                let name = exported_function_pairs.iter().filter(|v|v.0 ==function_index ).map(|v|v.1).last().unwrap_or(&internal_name);
                self.set_declare_function(build_context,&name,function_entry.type_ref(),&types)
            }).collect::<Result<Vec<&Value>,Error>>()
        } else{
            Ok(vec![])
        }
    }

    fn set_declare_imported_functions<'a>(&self, build_context:&'a BuildContext, wasm_module:&WasmModule, types:&[&Type]) ->Result<Vec<&'a Value>,Error>{
        if let Some(import_section) = wasm_module.import_section(){
            import_section.entries().iter().filter_map(|import_entry |{
                if let External::Function(type_index) = import_entry.external(){
                    Some(self.set_declare_function(build_context,import_entry.field(),*type_index,types))
                } else{
                    None
                }
            }).collect()
        } else{
            Ok(vec![])
        }
    }

    fn get_exported_function_pairs<'a>(&self, _build_context:&BuildContext,wasm_module:&'a WasmModule)->Vec<(u32,&'a str)>{
        if let Some(export_section) = wasm_module.export_section(){
            export_section.entries().iter().filter_map(|entry|{
                if let Internal::Function(function_index) = *entry.internal(){
                    Some((function_index,entry.field()))
                } else {
                    None
                }
            }).collect()
        } else{
            vec![]
        }
    }

    fn build_init_table_function(&self, build_context:&BuildContext,wasm_module:&WasmModule,functions:&[&Value],imported_count:u32)->Result<(),Error>{
        if let Some(table_section) = wasm_module.table_section()  {
            if let Some(elements_section) = wasm_module.elements_section(){
                let table_import_count = wasm_module.import_count(ImportCountType::Table);
                let table_initializers = elements_section.entries().iter().map(|element_segment|{
                    let offset = Self::segment_init_expr_to_value(build_context ,element_segment.offset())?;
                    let members = element_segment.members().iter().map(|member_index|{
                        Ok(*functions.get((*member_index)as usize).ok_or(NoSuchFunctionIndex{index:*member_index + table_import_count as u32})?)
                    }).collect::<Result<Vec<_>,Error>>()?;
                    Ok(TableInitializer::new(element_segment.index() ,offset,members))
                }).collect::<Result<Vec<_>,Error>>()?;
                self.table_compiler.build_init_function(build_context,table_section.entries(),&table_initializers,imported_count )?;
            }
        }
        Ok(())
    }

    fn set_declare_function<'a>(&self,build_context:&'a BuildContext,name:&str,type_index:u32,types:&[&Type])->Result<&'a Value,Error>{
        let function_type = types.get(type_index as usize).ok_or(NoSuchTypeIndex{index:type_index})?;
        Ok(build_context.module().set_declare_function(&Self::wasm_function_name(name),function_type))
    }
    fn set_declare_types<'a>(&self,build_context:&'a BuildContext,types:&[elements::Type])->Vec<&'a Type>{
        types.iter().map(|ty|{
            match ty {
                elements::Type::Function(function_type)=>{
                    let param_types = function_type.params().iter().map(|value_type|instructions::value_type_to_type(build_context,&value_type)).collect::<Vec<_>>();
                    Type::function(
                        function_type.return_type().map(|value_type|instructions::value_type_to_type(build_context,&value_type)).unwrap_or(Type::void(build_context.context())),
                        &param_types,
                        false
                    )
                }
            }
        }).collect()
    }



    fn build_const_initialize_global<'a>(&self, build_context:&'a BuildContext,index:u32, global_entry:&GlobalEntry)->Result<&'a Value,Error>{
        let g = self.set_declare_global(build_context,index, global_entry.global_type());
        let instruction = global_entry.init_expr().code().first().ok_or(NotExistGlobalInitializerInstruction)?;
        match instruction{
            Instruction::I32Const(v) => Some(instructions::i32_const_internal(build_context, *v)),
            Instruction::I64Const(v)=>Some(instructions::i64_const_internal(build_context, *v)),
            Instruction::F32Const(v) => Some(instructions::f32_const_internal(build_context, instructions::f32_reinterpret_i32(*v ))),
            Instruction::F64Const(v)=>Some(instructions::f64_const_internal(build_context, instructions::f64_reinterpret_i64(*v ))),
            _=>None,

        }.map(|const_initializer|{
            g.set_initializer(const_initializer);
            g.set_global_const(!global_entry.global_type().is_mutable());
        });
        Ok(g)
    }


    fn set_declare_global<'a>(& self, build_context:&'a BuildContext, index:u32, global_type:&GlobalType) ->&'a Value{
        build_context.module().set_declare_global(instructions::get_global_name(index).as_ref(), instructions::value_type_to_type(build_context,&global_type.content_type()))
    }


    fn build_init_data_sections_function(&self,build_context:&BuildContext,wasm_module:&WasmModule)->Result<(),Error>{
        if let Some(_data_section) = wasm_module.data_section() {
            let function = self.set_declare_init_data_sections_function(build_context);
            build_context.builder().build_function(build_context.context(), function, |_builder, _bb| {
                if let Some(data_section) = wasm_module.data_section() {
                    for segment in data_section.entries() {
                        self.build_data_segment(build_context, segment)?;
                    }
                }
                build_context.builder().build_ret_void();
                Ok(())
            })?;
        }
        Ok(())
    }

    fn segment_init_expr_to_value<'a>(build_context:&'a BuildContext,expr:&InitExpr)->Result<&'a Value,Error>{
        match expr.code().first().ok_or(NotExistInitExpr)? {
            Instruction::I64Const(v)=>Ok(instructions::i64_const_internal(build_context, *v)),
            Instruction::I32Const(v)=>Ok(instructions::i32_const_internal(build_context, *v )),
            Instruction::GetGlobal(v)=>Ok(instructions::get_global_internal(build_context, *v )?.get_initializer().ok_or(NotExistGlobalInitializerInstruction)?),
            invalid_instruction => Err(InvalidInstruction {instruction:invalid_instruction.clone()})?,
        }
    }

    fn build_data_segment(&self,build_context:&BuildContext,segment:&DataSegment)->Result<(),Error>{
        let offset = Self::segment_init_expr_to_value(build_context, segment.offset())?;
        let dest = self.linear_memory_compiler.build_get_real_address(build_context,segment.index(),offset,"");

        let int8 = Type::int8(build_context.context());
        let c_args = segment.value().iter().map(|v|Value::const_int(int8,*v as ::libc::c_ulonglong,false)).collect::<Vec<&Value>>();
        let src = Value::const_array( int8,&c_args);
        let dest  = build_context.builder().build_pointer_cast(dest, Type::ptr(Type::type_of(src),0),"");
        build_context.builder().build_store(src,dest);

        Ok(())
    }
}







#[cfg(test)]
mod tests{

    use super::*;
    use parity_wasm::elements::InitExpr;
    use parity_wasm::elements::ResizableLimits;
    use parity_wasm::elements::ValueType;
    use parity_wasm;
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
        let global_name = instructions::get_global_name(index);
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
            Instruction::F32Const(instructions::i32_reinterpret_f32(4.00) as u32),
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
            Instruction::F64Const(instructions::i64_reinterpret_f64(4.00) as u64),
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
        compiler.linear_memory_compiler.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(17, Some(25))])?;

        let offset = 1024;
        let expected_values:Vec<u8> =vec![221, 22, 254];
        let data_segment = DataSegment::new(0,InitExpr::new(vec![
            Instruction::I32Const(offset),
        ]),expected_values.clone());

        let function_name = "build_data_segment_works";
        build_test_function(&build_context,function_name,&[],|_builder,_bb|{
            compiler.build_data_segment(&build_context,&data_segment,)?;
            build_context.builder().build_ret_void();
            Ok(())
        })?;

        test_module_in_engine(build_context.module(),|engine|{

            let result = run_test_function_with_name(&engine, build_context.module(), &compiler.linear_memory_compiler.get_init_function_name(), &[])?;
            assert_eq!(1,result.to_int(false));
            run_test_function_with_name(engine,build_context.module(),function_name,&[])?;
            let linear_memory =  engine.get_global_value_ref_from_address::<*mut u8>(&compiler.linear_memory_compiler.get_memory_name(0));
            for (index,expected) in expected_values.iter().enumerate(){
                assert_eq!(*expected,unsafe{*linear_memory.add(offset as usize +index)});
            }
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    pub fn return_only_works()->Result<(),Error>{
        let return_only_module =  load_wasm_compiler_test_case("return_only")?;
        let wasm_compiler = WasmCompiler::<u32>::new();
        let context = Context::new();
        let module_id = "return_only";
        let build_context = BuildContext::new(module_id,&context);
        let function_name = WasmCompiler::<u32>::wasm_function_name("return_only");
        wasm_compiler.build_main_function(&build_context,module_id,&return_only_module,WasmCompiler::<u32>::set_declare_main_function(&build_context),||{
            let target_function = build_context.module().get_named_function(&function_name).ok_or(NoSuchLLVMFunction {name:function_name})?;
            build_context.builder().build_ret( build_context.builder().build_call(target_function,&[],""));
            Ok(())
        })?;
        test_module_main_in_engine(build_context.module(),32)
    }


    fn load_wasm_compiler_test_case(case_name:&str)->Result<WasmModule,Error>{
        let path_buf = get_target_dir()?;
        let wasm_path = path_buf.join("test_cases").join("wasm_compiler").join(case_name).join(case_name).with_extension("wasm");
        Ok(parity_wasm::deserialize_file(wasm_path)?)
    }
}