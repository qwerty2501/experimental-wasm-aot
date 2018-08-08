
use super::*;
use failure::Error;
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

    pub fn build_init_table_function(&self,build_context:&BuildContext,index:u32, minimum:u32,maximum:Option<u32>) -> Result<(),Error>{
        let minimum = Self::size_to_element_size(minimum);
        let maximum = maximum.map(Self::size_to_element_size);
        //self.table_memory_compiler.build_init_linear_memory_function(build_context,index,minimum,maximum)?;
        Ok(())
    }


    fn size_to_element_size(size:u32)->u32{
        size  * TType::ELEMENT_SIZE as u32
    }

}