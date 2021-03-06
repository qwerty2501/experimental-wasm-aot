use super::*;
use failure::Error;
use error::RuntimeError::*;
use parity_wasm::elements::Instruction;

pub struct Stack<'a,T:WasmIntType + 'a>{
    pub current_function:&'a Value,
    pub values:Vec<WasmValue<'a>>,
    pub labels:Vec<Label<'a>>,
    pub activations:Vec<Frame<'a,T>>,
}

impl<'a,T:WasmIntType> Stack<'a,T>{

    pub fn new<'b>(current_function:&'b Value, labels:Vec<Label<'b>>, values:Vec<WasmValue<'b>>,activations:Vec<Frame<'b,T>>)-> Stack<'b,T>{
        Stack{current_function, labels,values,activations}
    }

    pub fn previous_instruction(&self)->Result<Option<Instruction>,Error>{
        if let Some(last_label) = self.labels.last(){
            Ok(last_label.previous_instruction())
        } else{
            let current_frame = self.activations.current()?;
            Ok(current_frame.history.clone().previous_instruction)
        }
    }

    pub fn with_previous_instruction(mut self,instruction:Instruction,previous_value:Option<WasmValue<'a>>)->Result<Stack<'a,T>,Error>{
        {
            if let Some(last_label) = self.labels.pop(){
                self.labels.push(last_label.with_previous_instruction(instruction,previous_value));

            } else {
                let current = self.activations.current_mut()?;
                current.history = InstructionHistory::new(Some(instruction),previous_value)
            }
        }
        Ok(self)
    }
}

pub trait Activations<'a,T:WasmIntType>{
    fn current(&self)->Result<&Frame<'a,T>,Error>;
    fn current_mut(&mut self) -> Result<&mut Frame<'a, T>, Error>;
}

impl<'a,T:WasmIntType> Activations<'a,T> for Vec<Frame<'a,T>>{
    fn current(&self) -> Result<&Frame<'a, T>, Error> {
        Ok(self.first().ok_or(NotExistFrame)?)
    }

    fn current_mut(&mut self) -> Result<&mut Frame<'a, T>, Error> {
        Ok(self.first_mut().ok_or(NotExistFrame)?)
    }
}