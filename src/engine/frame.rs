use super::*;



pub struct Frame<'a,T:WasmIntType + 'a>{
    pub locals:Vec<LocalValue<'a>>,
    pub module_instance:ModuleInstance<'a,T>,
}

pub struct ModuleInstance<'a,T:WasmIntType + 'a>{
    pub types:&'a[ &'a Type],
    pub functions:&'a[&'a Value],
    pub labels:Vec<Label<'a>>,
    pub table_compiler:&'a FunctionTableCompiler<T>,
    pub linear_memory_compiler:&'a LinearMemoryCompiler<T>,

}

pub struct LocalValue<'a>{
    pub value:Option<&'a Value>,
    pub value_type:&'a Type,
}

pub struct Label<'a>{
    pub branch:Branch<'a>
}

pub struct Branch <'a>{
    pub start:&'a BasicBlock
}

impl<'a,T:WasmIntType + 'a> Frame<'a,T>{
    pub fn new(locals:Vec<LocalValue<'a>>,module_instance:ModuleInstance<'a,T> )->Frame<'a,T>{
        Frame{locals,module_instance}
    }
}

impl<'a,T:WasmIntType + 'a> ModuleInstance<'a,T>{
    pub fn new(types:&'a[&'a Type],functions:&'a[&'a Value],labels:Vec<Label<'a>>,table_compiler:&'a FunctionTableCompiler<T>,linear_memory_compiler:&'a LinearMemoryCompiler<T>)->ModuleInstance<'a,T>{
        ModuleInstance{types,functions,labels,table_compiler,linear_memory_compiler}
    }
}

impl<'a> LocalValue<'a>{
    pub fn from_value(value:&Value) ->LocalValue{
        LocalValue{value:Some(value),value_type:Type::type_of(value)}
    }

    pub fn from_value_type(value_type:&Type)->LocalValue{
        LocalValue{value:None,value_type}
    }
}

impl<'a> Label<'a>{
    pub fn new(branch:Branch<'a>)->Label{
        Label{branch}
    }
}

impl<'a> Branch<'a>{
    pub fn new(start:&'a BasicBlock)->Branch<'a>{
        Branch{start}
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    pub fn new_test_frame<'a,T:WasmIntType>(locals:Vec<LocalValue<'a>>,types:&'a[&'a Type],functions:&'a[&'a Value], labels:Vec<Label<'a>>,table_compiler:&'a FunctionTableCompiler<T>,linear_memory_compiler:&'a LinearMemoryCompiler<T>)->Frame<'a,T>{
        Frame::new(locals,ModuleInstance::new(types, functions,labels, table_compiler,linear_memory_compiler))
    }
}