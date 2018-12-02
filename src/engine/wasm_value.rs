use super::*;


pub enum WasmValue<'a>{
    Value{value:&'a Value},
    BlockReturnValue{return_value:BlockReturnValue<'a>},
}


impl<'a> WasmValue<'a>{
    pub fn new_value(value:&'a Value)-> WasmValue<'a>{
        WasmValue::Value {value}
    }

    pub fn new_block_return_value(return_value:BlockReturnValue<'a>)-> WasmValue<'a>{
        WasmValue::BlockReturnValue {return_value}
    }

    pub fn to_value(&self, build_context:&'a BuildContext)->&'a Value{
        match self {
            WasmValue::Value {value} => value,
            WasmValue::BlockReturnValue {return_value} => return_value.to_value(build_context),
        }
    }

    pub fn value_type(&self) -> &'a Type{
        match self{
            WasmValue::Value {value} => Type::type_of(value),
            WasmValue::BlockReturnValue {return_value} => return_value.value_type(),
        }
    }
}

impl<'a> Clone for WasmValue<'a>{
    fn clone(&self) -> Self {
        match self {
            WasmValue::Value {value} => WasmValue::Value {value},
            WasmValue::BlockReturnValue {return_value} => WasmValue::BlockReturnValue {return_value:return_value.clone()}
        }
    }
}