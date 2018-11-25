use super::*;
use parity_wasm::elements::ValueType;
use std::slice::Iter;

pub struct BlockReturnValue<'a>{
    value_ptr:&'a Value,
    value_type:&'a Type
}


impl<'a> BlockReturnValue<'a>{
    pub fn from_block_types(build_context:&'a BuildContext,value_types: &[ValueType])->Vec<Self>{
        value_types.iter().map(|vt| Self::new(build_context,*vt)).collect()
    }
    pub fn new(build_context:&'a BuildContext,value_type:ValueType)-> Self{

        BlockReturnValue {value_ptr:build_context.builder().build_alloca(Type::from_wasm_value_type(build_context.context(), value_type),""),  value_type:Type::from_wasm_value_type(build_context.context(), value_type)}
    }

    pub fn store(&self, build_context:&'a BuildContext, value:&'a Value)-> &'a Value{
        build_context.builder().build_store(value,self.value_ptr)
    }

    pub fn to_value(&self, build_context:&'a BuildContext)->&'a Value{
        build_context.builder().build_load(self.value_ptr,"")
    }
}

impl<'a> Clone for BlockReturnValue<'a> {
    fn clone(&self) -> Self {
        BlockReturnValue{value_ptr:self.value_ptr,value_type:self.value_type}
    }
}

