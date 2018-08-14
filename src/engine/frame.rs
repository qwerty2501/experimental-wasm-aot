use super::*;
pub struct Frame<'a,T:WasmIntType>{
    pub locals:Vec<&'a Value>,
    pub module_instance:ModuleInstance<'a,T>,
}

pub struct ModuleInstance<'a,T:WasmIntType>{
    pub types:&'a [&'a Type],
    pub functions:&'a[&'a Value],
    pub table_compiler:FunctionTableCompiler<T>,
    pub linear_memory_compiler:LinearMemoryCompiler<T>,

}