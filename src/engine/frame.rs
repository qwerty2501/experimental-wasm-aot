use super::*;
use parity_wasm::elements::ValueType as WasmValueType;
use parity_wasm::elements::Instruction;



pub struct Frame<'a,T:WasmIntType + 'a>{
    pub locals:Vec<LocalValue<'a>>,
    pub module_instance:ModuleInstance<'a,T>,
    pub history:InstructionHistory<'a>,
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
    Loop{start:&'a BasicBlock,next:&'a BasicBlock,history:InstructionHistory<'a>},
    If{start:&'a BasicBlock,else_block:&'a BasicBlock,next:&'a BasicBlock,if_history:InstructionHistory<'a>,else_history:Option<InstructionHistory<'a>>},
    Block{start:&'a BasicBlock,next:&'a BasicBlock,history:InstructionHistory<'a>},
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
        Frame{locals,module_instance, history:InstructionHistory::new(None,None)}
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
            label_type: LabelType::Block{start,next,history:InstructionHistory::new(None,None)},
            return_value,
        }
    }

    pub fn new_loop(start:&'a BasicBlock, next:&'a BasicBlock,return_value:Option<BlockReturnValue<'a>>)-> Label<'a>{
        Label{
            label_type: LabelType::Loop {start,next,history:InstructionHistory::new(None,None)},
            return_value,
        }
    }

    pub fn new_if(start:&'a BasicBlock,else_block:&'a BasicBlock,next:&'a BasicBlock, return_value:Option<BlockReturnValue<'a>>) -> Label<'a>{
        Label{
            label_type: LabelType::If {start,else_block, next,if_history:InstructionHistory::new(None,None),else_history:None},
            return_value,
        }
    }

    pub fn previous_instruction(&self) -> Option<Instruction>{
        match self.label_type{
            LabelType::If {start:_,else_block:_,next:_,ref if_history,ref else_history} =>{
                if let Some(history) = else_history{
                    history.clone().previous_instruction
                } else{
                    if_history.clone().previous_instruction
                }
            }

            LabelType::Block {start:_,next:_,ref history} =>{
                history.clone().previous_instruction
            }
            LabelType::Loop {start:_,next:_,ref history} =>{
                history.clone().previous_instruction
            }
        }
    }
    pub fn with_previous_instruction(&self,instruction:Instruction,previous_value:Option< WasmValue<'a>>)->Label<'a>{
        let label_type = match self.label_type{
            LabelType::If {start,else_block,next,ref if_history, ref else_history} =>{
                if let Some(history) = else_history{
                    LabelType::If {start,else_block,next,if_history:if_history.clone(),else_history:Some(InstructionHistory::new(Some(instruction),previous_value))}
                } else{
                    LabelType::If {start,else_block,next,if_history:InstructionHistory::new(Some(instruction),previous_value),else_history:else_history.clone()}
                }
            }

            LabelType::Block {start,next,history:_} =>{
                LabelType::Block {start,next,history:InstructionHistory::new(Some(instruction),previous_value)}
            }
            LabelType::Loop {start,next,history:_} =>{
                LabelType::Loop {start,next,history:InstructionHistory::new(Some(instruction),previous_value)}
            }
        };
        Label{
            label_type,
            return_value:self.return_value.clone()}

    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn new_test_frame<'a,T:WasmIntType>(locals:Vec<LocalValue<'a>>,types:&'a[&'a Type],functions:&'a[&'a Value], table_compiler:&'a FunctionTableCompiler<T>,linear_memory_compiler:&'a LinearMemoryCompiler<T>)->Frame<'a,T>{
        Frame::new(locals,ModuleInstance::new(types, functions, table_compiler,linear_memory_compiler))
    }
}