#[macro_use]
use failure::Error;
use super::llvm::*;
use super::constants::*;
use super::wasm::*;
use std::ptr;
const MODULE_ID:&str = "__wasm_linear_memory_module";
const LINEAR_MEMORY_NAME:&str = "__wasm_linear_memory";
const LINEAR_MEMORY_SIZE_NAME:&str = "__wasm_linear_memory_size";


pub fn compile(context:&Context) -> Result<Guard<Module>,Error>{
    let builder = Builder::new( context);
    let module = Module::new(MODULE_ID, context);
    set_linear_memory(&module);
    Ok(module)
}

pub fn set_linear_memory(module:&Module) ->& Value {
    let memory_pointer_type =  Type::ptr(Type::void( module.context()), 0);
    module.set_global(LINEAR_MEMORY_NAME,memory_pointer_type)
}

pub fn set_linear_memory_size(module:&Module)->&Value{
    let int_ptr_type = Type::int_ptr(module.context());
    module.set_global(LINEAR_MEMORY_SIZE_NAME,int_ptr_type)
}

pub fn set_grow_linear_memory_function(module:&Module)->&Value{
    let context = module.context();
    let int_ptr_type = Type::int_ptr(context);
    let parms =[int_ptr_type];
    let grow_linear_memory_type = Type::function(int_ptr_type,&parms,true);
    module.set_wasm_function(wasm_call_name!("grow_linear_memory"),grow_linear_memory_type)
}

pub fn build_grow_linear_memory_function<'a>(module:&'a Module, builder:&'a Builder)->&'a Value{
   let function = set_grow_linear_memory_function(module);
    builder.build_function(module.context(),function,|b|{
        build_grow_linear_memory(module,b,function.get_first_parm().unwrap())
    })
}

pub fn build_grow_linear_memory<'a>(module:&'a Module, builder:&'a Builder, delta :&Value)->&'a Value{

    let linear_memory = set_linear_memory(module);
    let linear_memory_size = set_linear_memory_size(module);
    let linear_memory_size_cache = builder.build_load(linear_memory_size,"");
    let linear_memory_cache = builder.build_load(linear_memory,"");
    let context = module.context();

    let i32_type = Type::int32(context);
    let int_type = Type::int_ptr(context);
    let void_type = Type::void(context);
    let void_ptr_type = Type::ptr(void_type, 0);
    let param_types = [void_ptr_type,int_type,i32_type,i32_type,i32_type,i32_type];
    let mmap_type = Type::function(void_ptr_type,&param_types,true);
    let mmap_function = module.set_function("mmap",mmap_type);
    let page_size_value = Value::const_int(int_type,page_size as u64,false);
    let grow_size = builder.build_mul(page_size_value,delta,"");
    let prot_value = Value::const_int(i32_type,(::libc::PROT_READ | ::libc::PROT_WRITE) as ::libc::c_ulonglong,true);
    let flags_value = Value::const_int(i32_type, (::libc::MAP_PRIVATE | ::libc::MAP_ANONYMOUS) as ::libc::c_ulonglong,true );
    let fd_value = Value::const_int(i32_type,-1_isize as ::libc::c_ulonglong,true);
    let offset_value = Value::const_int(i32_type,0,true);
    let args = [linear_memory_cache,grow_size,prot_value,flags_value,fd_value,offset_value];
    let mapped_ptr = builder.build_call(mmap_function,&args,"");
    builder.build_store(mapped_ptr,linear_memory);
    builder.build_store(builder.build_add(linear_memory_size_cache,grow_size,""),linear_memory_size);
    linear_memory_size_cache
}



#[cfg(test)]
mod tests{



    use super::*;

    #[test]
    pub fn compile_works(){
        let  context = Context::new();

        let result = compile(&context);

        assert!(result.is_ok());


        let _ =result.map(| module|{
            assert!(module.get_named_global(LINEAR_MEMORY_NAME).is_some());
        });
    }

    #[test]
    pub fn grow_linear_memory_works(){
        let context = Context::new();
        let module_id = "build_linear_memory_works";
        let module = Module::new(module_id,&context);
        let builder = Builder::new(&context);
        build_grow_linear_memory_function(&module,&builder);
        module.dump();
        assert!(!analysis::verify_module(&module,analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction));

    }

    #[test]
    pub fn set_linear_memory_works(){
        let context = Context::new();

        let module_id = "set_linear_memory_works";
        let module = Module::new(module_id, &context);
        assert_ne!(ptr::null(), set_linear_memory(&module).as_ptr());
        assert!( module.get_named_global(LINEAR_MEMORY_NAME).is_some());
    }

}