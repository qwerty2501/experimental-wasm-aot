use super::*;
use parity_wasm::elements::Instruction;

pub struct InstructionHistory<'a>{
    pub previous_instruction:Option<Instruction>,
    pub previous_value:Option<  WasmValue<'a>>,
}

impl<'a> InstructionHistory<'a>{
    pub fn new(previous_instruction:Option<Instruction>,previous_value:Option< WasmValue<'a>>)-> InstructionHistory<'a>{
        InstructionHistory{ previous_instruction ,previous_value}
    }
}

impl<'a> Clone for InstructionHistory<'a>{
    fn clone(&self) -> Self {
        InstructionHistory{previous_instruction:self.previous_instruction.clone(),previous_value:self.previous_value.clone()}
    }
}