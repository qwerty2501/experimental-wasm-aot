use super::*;
use failure::Error;
use error::RuntimeError::*;
pub struct Stack<'a,T:WasmIntType + 'a>{
    pub current_function:&'a Value,
    pub values:Vec<&'a Value>,
    pub activations:Vec<Frame<'a,T>>,
}

impl<'a,T:WasmIntType> Stack<'a,T>{

    pub fn new<'b>(current_function:&'b Value, values:Vec<&'b Value>,activations:Vec<Frame<'b,T>>)-> Stack<'b,T>{
        Stack{current_function, values,activations}
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