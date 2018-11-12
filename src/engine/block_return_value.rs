use super::*;
use parity_wasm::elements::ValueType;

pub struct BlockReturnValue<'a>{
    value_ptr:&'a Value
}


impl<'a> BlockReturnValue<'a>{
    pub fn new_block_return_value(build_context:&'a BuildContext,value_type:ValueType)-> BlockReturnValue<'a>{
        BlockReturnValue {value_ptr:build_context.builder().build_alloca(Type::from_wasm_value_type(build_context.context(), value_type),"")}
    }

    pub fn store(&self, build_context:&'a BuildContext, value:&'a Value)-> &'a Value{
        build_context.builder().build_store(value,self.value_ptr)
    }

    pub fn to_value(&self, build_context:&'a BuildContext)->&'a Value{
        build_context.builder().build_load(self.value_ptr,"")
    }
}