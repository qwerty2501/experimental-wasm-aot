use failure::Error;
use super::compiler::*;
use std::ptr;
const MODULE_ID:&str = "__wasm_linear_memory_module";
const LINEAR_MEMORY_NAME:&str = "__wasm_linear_memory";
pub fn compile(context:&Context) -> Result<Guard<Module>,Error>{
    let builder = Builder::new( context);
    let module = Module::new(MODULE_ID, context);
    set_linear_memory(&module);
    Ok(module)
}

pub fn set_linear_memory(module:&Module) ->& Value {
    let memory_pointer_type =  Type::pointer(Type::int8( module.context()), 0);
    module.set_global(LINEAR_MEMORY_NAME,memory_pointer_type)
}

pub fn build_linear_memory(module:& Module, builder:& Builder,memory_size: ::libc::size_t){

    let linear_memory = set_linear_memory(module);
    let context = module.context();

    let i32_type = Type::int32(context);

    let prot = Value::const_int(i32_type,(::libc::PROT_READ | ::libc::PROT_WRITE) as ::libc::c_ulonglong,true);
    let flags = Value::const_int(i32_type, (::libc::MAP_PRIVATE | ::libc::MAP_ANONYMOUS) as ::libc::c_ulonglong,true );
}



#[cfg(test)]
mod tests{



    use super::*;

    #[test]
    pub fn compile_works(){
        let  context = Context::new();

        let result = compile(&context);

        assert_eq!(true,result.is_ok());


        let _ =result.map(| module|{
            assert_eq!(true, module.get_named_global(LINEAR_MEMORY_NAME).is_some());
        });


    }

    #[test]
    pub fn set_linear_memory_works(){
        let context = Context::new();

        let module_id = "get_or_insert_linear_memory_works";
        let module = Module::new(module_id, &context);
        assert_ne!(ptr::null(), set_linear_memory(&module).as_ptr());
        assert_eq!(true, module.get_named_global(LINEAR_MEMORY_NAME).is_some());
    }

}