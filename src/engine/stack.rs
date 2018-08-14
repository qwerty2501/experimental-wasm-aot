use super::*;

pub struct Stack<'a,T:WasmIntType>{
    values:Vec<&'a Value>,
    activations:Vec<Frame<'a,T>>,
}