
use super::*;
use failure::Error;
use parity_wasm::elements::ResizableLimits;
use parity_wasm::elements::ElementSegment;
use parity_wasm::elements;
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

    pub fn build_init_function(&self, build_context:&BuildContext,table_types:&[elements::TableType],initializers:&[TableInitializer], import_count:u32) -> Result<(),Error>{

        self.table_memory_compiler.build_init_function_internal(build_context,import_count,&table_types.iter().map(|t|t.limits()).collect::<Vec<_>>(),||{

            let address_type = Type::int_wasm_ptr::<T>(build_context.context());
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



    fn build_size_to_element_size<'a>( build_context:&'a BuildContext,size:&Value)->&'a Value{
        build_context.builder().build_mul(size,Value::const_int(Type::int32(build_context.context()),TType::ELEMENT_SIZE as u64,false),"")
    }

    fn size_to_element_size(size:u32)->u32{
        size  * TType::ELEMENT_SIZE as u32
    }

}

pub struct TableInitializer<'a>{
    index:u32,
    offset:&'a Value,
    members:Vec<&'a Value>,
}

impl<'a> TableInitializer<'a>{

    pub fn new<'e>(index:u32,offset:&'e Value,members:Vec<&'e Value>)->TableInitializer<'e>{
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
        analysis::verify_module(build_context.module(),analysis::VerifierFailureAction::LLVMPrintMessageAction)?;
        test_module_in_engine(build_context.module(),|engine|{
            let _ = run_test_function_with_name(engine,build_context.module(),&compiler.table_memory_compiler.get_init_function_name(),&[])?;
            let memory:*mut fn()->u32 = *engine.get_global_value_ref_from_address(&compiler.table_memory_compiler.get_memory_name(0));
            unsafe{
                let test_func_ptr = memory.add(1);
                assert_ne!(::std::ptr::null_mut(),test_func_ptr);
                assert_eq!(expected,(*test_func_ptr)());
            }
            Ok(())
        })?;

        Ok(())
    }
}