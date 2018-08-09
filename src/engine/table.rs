
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

    pub fn build_init_function(&self, build_context:&BuildContext,table_types:&[&elements::TableType],initializers:&[TableInitializer], import_count:u32) -> Result<(),Error>{

        self.table_memory_compiler.build_init_function_internal(build_context,import_count,&table_types.iter().map(|t|t.limits()).collect::<Vec<_>>(),||{
            let address_type = Type::int_wasm_ptr::<T>(build_context.context());
            let function_pointer_type = Type::ptr(Type::void(build_context.context()),0);
            for initializer in initializers{
                let address = self.table_memory_compiler.build_get_real_address(build_context,initializer.index,Value::const_int(address_type,Self::size_to_element_size(initializer.offset) as u64,false),"");

                let values = initializer.elements.iter().map(|e|e.const_pointer_cast(function_pointer_type)).collect::<Vec<_>>();
                let array = Value::const_array(function_pointer_type,&values);
                build_context.builder().build_store(array,address);
            }
            Ok(())
        })?;
        Ok(())
    }


    fn size_to_element_size(size:u32)->u32{
        size  * TType::ELEMENT_SIZE as u32
    }

}

pub struct TableInitializer<'a>{
    index:u32,
    offset:u32,
    elements:&'a [&'a Value]
}