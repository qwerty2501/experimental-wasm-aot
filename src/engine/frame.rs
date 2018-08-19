use super::*;



pub struct Frame<'a,T:WasmIntType + 'a>{
    pub locals:Vec<LocalValue<'a>>,
    pub module_instance:ModuleInstance<'a,T>,
}

pub struct ModuleInstance<'a,T:WasmIntType + 'a>{
    pub types:Vec<&'a Type>,
    pub functions:Vec<&'a Value>,
    pub labels:Vec<&'a BasicBlock>,
    pub table_compiler:FunctionTableCompiler<T>,
    pub linear_memory_compiler:LinearMemoryCompiler<T>,

}

pub struct LocalValue<'a>{
    pub value:Option<&'a Value>,
    pub value_type:&'a Type,
}

impl<'a,T:WasmIntType + 'a> Frame<'a,T>{
    pub fn new(locals:Vec<LocalValue<'a>>,module_instance:ModuleInstance<'a,T> )->Frame<'a,T>{
        Frame{locals,module_instance}
    }
}

impl<'a,T:WasmIntType + 'a> ModuleInstance<'a,T>{
    pub fn new(types:Vec<&'a Type>,functions:Vec<&'a Value>,labels:Vec<&'a BasicBlock>,table_compiler:FunctionTableCompiler<T>,linear_memory_compiler:LinearMemoryCompiler<T>)->ModuleInstance<'a,T>{
        ModuleInstance{types,functions,labels,table_compiler,linear_memory_compiler}
    }
}

impl<'a> LocalValue<'a>{
    pub fn new (value:&Value)->LocalValue{
        LocalValue{value:Some(value),value_type:Type::type_of(value)}
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    pub fn new_test_frame<'a,T:WasmIntType>(locals:Vec<LocalValue<'a>>,types:Vec<&'a Type>,labels:Vec<&'a BasicBlock>, functions:Vec<&'a Value>)->Frame<'a,T>{
        Frame::new(locals,ModuleInstance::new(types, functions,labels, FunctionTableCompiler::<T>::new(),LinearMemoryCompiler::<T>::new()))
    }
}