
use super::*;

pub struct TableTypeContext;

impl MemoryTypeContext for TableTypeContext{
    const MEMORY_NAME_PREFIX: &'static str = "__wasm_table";
}

pub type TableMemoryCompiler<T> = MemoryCompiler<TableTypeContext,T>;