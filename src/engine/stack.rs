use super::*;

pub struct Stack<'a,T:WasmIntType + 'a>{
    values:Vec<&'a Value>,
    activations:Vec<Frame<'a,T>>,
}

impl<'a,T:WasmIntType> Stack<'a,T>{

    pub fn new<'b>(values:Vec<&'b Value>,activations:Vec<Frame<'b,T>>)->Stack<'b,T>{
        Stack{values,activations}
    }

    pub fn pop_value(&mut self)->Option<&Value>{
        self.values.pop()
    }

    pub fn push_value(&'a mut self,value:&'a Value){
        self.values.push(value)
    }
}