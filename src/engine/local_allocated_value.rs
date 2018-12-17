use super::*;
use parity_wasm::elements::ValueType;
use std::slice::Iter;

pub struct LocalAllocatedValue<'a>{
    value_ptr:&'a Value,
    value_type:&'a Type
}


impl<'a> LocalAllocatedValue<'a>{
    pub fn from_block_types(build_context:&'a BuildContext,value_types: &[ValueType])->Vec<Self>{
        value_types.iter().map(|vt| Self::new_from_wasm_type(build_context, *vt)).collect()
    }
    pub fn new_from_wasm_type(build_context:&'a BuildContext, value_type:ValueType) -> Self{
        Self::new(build_context,Type::from_wasm_value_type(build_context.context(), value_type))
    }

    pub fn new(build_context:&'a BuildContext,ty:&'a Type)->Self{
        LocalAllocatedValue {value_ptr:build_context.builder().build_alloca(ty, ""),  value_type:ty}
    }

    pub fn store(&self, build_context:&'a BuildContext, value:&'a Value)-> &'a Value{
        build_context.builder().build_store(value,self.value_ptr)
    }

    pub fn to_value(&self, build_context:&'a BuildContext)->&'a Value{
        build_context.builder().build_load(self.value_ptr,"")
    }

    pub fn value_type(&self)->&'a Type{
        self.value_type
    }
}

impl<'a> Clone for LocalAllocatedValue<'a> {
    fn clone(&self) -> Self {
        LocalAllocatedValue {value_ptr:self.value_ptr,value_type:self.value_type}
    }
}

