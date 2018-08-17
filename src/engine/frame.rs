use super::*;
pub struct Frame<'a,T:WasmIntType + 'a>{
    pub locals:Vec<&'a Value>,
    pub module_instance:ModuleInstance<'a,T>,
}

pub struct ModuleInstance<'a,T:WasmIntType + 'a>{
    pub types:Vec<&'a Type>,
    pub functions:Vec<&'a Value>,
    pub labels:Vec<&'a BasicBlock>,
    pub table_compiler:FunctionTableCompiler<T>,
    pub linear_memory_compiler:LinearMemoryCompiler<T>,

}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    pub fn new_test_frame<'a,T:WasmIntType>(locals:Vec<&'a Value>,types:Vec<&'a Type>,labels:Vec<&'a BasicBlock>, functions:Vec<&'a Value>)->Frame<'a,T>{
        Frame{locals,module_instance:ModuleInstance::<T>{
            types,
            functions,
            labels,
            table_compiler: FunctionTableCompiler::<T>::new(),
            linear_memory_compiler: LinearMemoryCompiler::<T>::new(),
        }}
    }
}