
use super::*;
use failure::Error;
use parity_wasm::elements;
use parity_wasm::elements::ImportCountType;
use parity_wasm::elements::Module as WasmModule;

pub struct TableTypeContext<TType:TableType>(::std::marker::PhantomData<TType>);

impl<TType:TableType> MemoryTypeContext for TableTypeContext<TType>{
    const MEMORY_NAME_PREFIX: &'static str = "__wasm_table";
    const UNIT_SIZE: u32 = TType::ELEMENT_SIZE;
    const DEFAULT_MAXIMUM_UNIT_SIZE: u32 = ::std::u32::MAX / TType::ELEMENT_SIZE;
}

pub type TableMemoryCompiler<TType,T> = MemoryCompiler<TableTypeContext<TType>,T>;

pub trait TableType{
    const ELEMENT_SIZE:u32;
}

pub enum AnyFunctionTableType{}
impl TableType for AnyFunctionTableType{
    const ELEMENT_SIZE: u32 = ::std::mem::size_of::<fn()>() as u32;
}

pub type FunctionTableCompiler<T> = TableCompiler<AnyFunctionTableType,T>;

pub struct TableCompiler<TType:TableType,T:WasmIntType>{
    table_memory_compiler:TableMemoryCompiler<TType,T>,
    table_type: ::std::marker::PhantomData<TType>
}

impl<TType:TableType,T:WasmIntType> TableCompiler<TType,T>{

    pub fn new()->TableCompiler<TType,T>{
        TableCompiler::<TType,T>{table_memory_compiler:TableMemoryCompiler::<TType,T>::new(),table_type: ::std::marker::PhantomData::<TType>}
    }

    pub fn get_init_function_name(&self)->String{
        self.table_memory_compiler.get_init_function_name()
    }

    pub fn compile(&self, build_context:&BuildContext,wasm_module:&WasmModule,initializers:&[TableInitializer])->Result<(),Error>{

        if let Some(table_section) = wasm_module.table_section(){
            self.build_init_function(build_context,table_section.entries(),initializers,wasm_module.import_count(ImportCountType::Table ) as u32)?;
        }
        Ok(())
    }

    pub fn build_init_function(&self, build_context:&BuildContext,table_types:&[elements::TableType],initializers:&[TableInitializer], import_count:u32) -> Result<(),Error>{

        self.table_memory_compiler.build_init_functions(build_context, import_count, &table_types.iter().map(|t|t.limits()).collect::<Vec<_>>(), ||{
            let function_pointer_type = Type::ptr(Type::void(build_context.context()),0);
            for initializer in initializers{
                let address = self.table_memory_compiler.build_get_real_address(build_context,initializer.index,Self::build_size_to_element_size(build_context,initializer.offset),"");

                let values = initializer.members.iter().map(|e|e.const_pointer_cast(function_pointer_type)).collect::<Vec<_>>();
                let array = Value::const_array(function_pointer_type,&values);
                let address = build_context.builder().build_pointer_cast(address,Type::ptr(Type::type_of(array),0),"address");
                build_context.builder().build_store(array,address);
            }

            Ok(())
        })?;
        Ok(())
    }

    pub fn build_get_function_address<'a>(&self,build_context:&'a BuildContext,index_value:&'a Value,function_type:&'a Type, table_index:u8)-> &'a Value{
        let address = self.table_memory_compiler.build_get_real_address(build_context,table_index as u32,Self::build_size_to_element_size(build_context,index_value),"");
        build_context.builder().build_load( build_context.builder().build_pointer_cast(address,Type::ptr(Type::ptr(function_type,0),0),""),"")
    }


    fn build_size_to_element_size<'a>( build_context:&'a BuildContext,size:&Value)->&'a Value{
        build_context.builder().build_mul(size,Value::const_int(Type::int32(build_context.context()),TType::ELEMENT_SIZE as u64,false),"")
    }


}

pub struct TableInitializer<'a>{
    index:u32,
    offset:&'a Value,
    members:Vec<&'a Value>,
}

impl<'a> TableInitializer<'a>{

    pub fn new(index:u32,offset:&'a Value,members:Vec<&'a Value>)->TableInitializer<'a>{
        TableInitializer{index,offset, members }
    }
}

#[cfg(test)]
mod tests{

    use super::*;
    #[test]
    pub fn build_init_function_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_init_function_works",&context);
        let compiler = FunctionTableCompiler::<u32>::new();
        let table_types = [
            elements::TableType::new(2,Some(2)),
        ];


        let function_name = "test_function_name";
        let int32  = Type::int32(build_context.context());
        let target_function_type = Type::function(int32,&[],false);
        let target_function = build_context.module().set_declare_function(function_name,target_function_type);
        let expected:u32 = 32;
        build_context.builder().build_function(build_context.context(),target_function,|_,_|{
            build_context.builder().build_ret(Value::const_int(int32,expected as u64,false));
            Ok(())
        })?;


        let initializers = [
            TableInitializer::new(0,Value::const_int(Type::int32(build_context.context()),1 ,false),vec![target_function]),
        ];


        compiler.build_init_function(&build_context,&table_types,&initializers,0)?;
        let call_function_name = "call_function_name";
        let call_function = build_context.module().set_declare_function(call_function_name,target_function_type);
        build_context.builder().build_function(build_context.context(),call_function,|_,_|{
            let f = compiler.build_get_function_address(&build_context,Value::const_int(Type::int32(build_context.context()),1,false),target_function_type,0);
            build_context.builder().build_ret( build_context.builder().build_call(f,&[],""));
            Ok(())
        })?;
        analysis::verify_module(build_context.module(),analysis::VerifierFailureAction::LLVMPrintMessageAction)?;
        test_module_in_engine(build_context.module(),|engine|{
            let _ = run_test_function_with_name(engine,build_context.module(),&compiler.get_init_function_name(),&[])?;
            let memory:*mut fn()->u32 = *engine.get_global_value_ref_from_address(&compiler.table_memory_compiler.get_memory_name(0));
            unsafe{
                let test_func_ptr = memory.add(1);
                assert_ne!(::std::ptr::null_mut(),test_func_ptr);
                assert_eq!(expected,(*test_func_ptr)());
            }

            let r = run_test_function_with_name(engine,build_context.module(),call_function_name,&[])?;

            assert_eq!(expected,r.to_int(false) as u32);
            Ok(())
        })?;

        Ok(())
    }
}