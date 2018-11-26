use super::*;
use parity_wasm::elements::ValueType as WasmValueType;
use parity_wasm::elements::Instruction;



pub struct Frame<'a,T:WasmIntType + 'a>{
    pub locals:Vec<LocalValue<'a>>,
    pub module_instance:ModuleInstance<'a,T>,
    pub before_instruction:Option<Instruction>,
}

pub struct ModuleInstance<'a,T:WasmIntType + 'a>{
    pub types:&'a[ &'a Type],
    pub functions:&'a[&'a Value],
    pub table_compiler:&'a FunctionTableCompiler<T>,
    pub linear_memory_compiler:&'a LinearMemoryCompiler<T>,

}

pub struct LocalValue<'a>{
    pub value:Option<WasmValue<'a>>,
    pub value_type:&'a Type,
}

pub struct Label<'a>{
    pub label_type: LabelType<'a>,
    pub return_value:Option<BlockReturnValue<'a>>
}

pub enum LabelType<'a>{
    Loop{start:&'a BasicBlock,next:&'a BasicBlock},
    If{start:&'a BasicBlock,else_block:&'a BasicBlock,next:&'a BasicBlock},
    Block{start:&'a BasicBlock,next:&'a BasicBlock},
}



impl<'a,T:WasmIntType + 'a> ModuleInstance<'a,T>{
    pub fn new(types:&'a[&'a Type],functions:&'a[&'a Value],table_compiler:&'a FunctionTableCompiler<T>,linear_memory_compiler:&'a LinearMemoryCompiler<T>)->ModuleInstance<'a,T>{
        ModuleInstance{types,functions,table_compiler,linear_memory_compiler}
    }
}

impl<'a,T:WasmIntType + 'a> Clone for ModuleInstance<'a,T>{
    fn clone(&self) -> Self {
        Self{
            functions:self.functions,
            linear_memory_compiler:self.linear_memory_compiler,
            table_compiler:self.table_compiler,
            types:self.types,
        }
    }
}

impl<'a,T:WasmIntType + 'a> Frame<'a,T>{
    pub fn new(locals:Vec<LocalValue<'a>>, module_instance:ModuleInstance<'a,T> )->Frame<'a,T>{
        Frame{locals,module_instance,before_instruction:None}
    }
}

impl<'a> Clone for LocalValue<'a>{
    fn clone(&self) -> Self {
       LocalValue{value:self.value.clone(),value_type: self.value_type}
    }
}

impl<'a> LocalValue<'a>{
    pub fn from_value(value:&Value) ->LocalValue{
        LocalValue{value:Some(WasmValue::new_value(value)),value_type:Type::type_of(value)}
    }

    pub fn from_value_type(value_type:&Type)->LocalValue{
        LocalValue{value:None,value_type}
    }
}

impl<'a> Label<'a>{
    pub fn new_block(start:&'a BasicBlock,next:&'a BasicBlock,return_value:Option<BlockReturnValue<'a>>)->Label<'a>{
        Label{
            label_type: LabelType::Block{start,next},
            return_value,
        }
    }

    pub fn new_loop(start:&'a BasicBlock, next:&'a BasicBlock,return_value:Option<BlockReturnValue<'a>>)-> Label<'a>{
        Label{
            label_type: LabelType::Loop {start,next},
            return_value,
        }
    }

    pub fn new_if(start:&'a BasicBlock,else_block:&'a BasicBlock,next:&'a BasicBlock, return_value:Option<BlockReturnValue<'a>>) -> Label<'a>{
        Label{
            label_type: LabelType::If {start,else_block, next},
            return_value,
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn new_test_frame<'a,T:WasmIntType>(locals:Vec<LocalValue<'a>>,types:&'a[&'a Type],functions:&'a[&'a Value], table_compiler:&'a FunctionTableCompiler<T>,linear_memory_compiler:&'a LinearMemoryCompiler<T>)->Frame<'a,T>{
        Frame::new(locals,ModuleInstance::new(types, functions, table_compiler,linear_memory_compiler))
    }
}