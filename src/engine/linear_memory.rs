use failure::Error;
use super::llvm::*;
use super::constants::*;
use super::wasm::*;
use std::ptr;
use error::RuntimeError::*;

const MODULE_ID:&str = "__wasm_linear_memory_module";
const LINEAR_MEMORY_NAME:&str = "__wasm_linear_memory";
const LINEAR_MEMORY_PAGE_COUNT_NAME:&str = "__wasm_linear_memory_size";


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

pub fn set_linear_memory_size(module:&Module,wasm_align:u32)->&Value{
    let i64_type = Type::int64(module.context());
    module.set_global(LINEAR_MEMORY_PAGE_COUNT_NAME, i64_type)
}

pub fn set_grow_linear_memory_function(module:&Module,wasm_align:u32)->&Value{
    let context = module.context();
    let wasm32_int_type = Type::int_wasm32_ptr(context,wasm_align);
    let parms =[wasm32_int_type];
    let grow_linear_memory_type = Type::function(wasm32_int_type,&parms,true);
    module.set_wasm_function(wasm_function_name!("grow_linear_memory"), grow_linear_memory_type)
}

pub fn build_grow_linear_memory_function<'a>(module:&'a Module, b:&'a Builder,wasm_align:u32,maximum:Option<usize>)->Result<(),Error>{
   let function = set_grow_linear_memory_function(module,wasm_align);
    b.build_function(module.context(),function,|builder,bb|{
        let context = module.context();
        let delta = function.get_first_parm().ok_or(FunctionParameterNotEnough{message:"the parameter \"delta\" is missing.".to_string()})?;
        let wasm32_int_type = Type::int_wasm32_ptr(context,wasm_align);
        let i64_type = Type::int64(context);
        let max_linear_memory_size = Value::const_int(i64_type,(maximum.unwrap_or_else(|| DEFAULT_MAXIMUM ) * PAGE_SIZE) as ::libc::c_ulonglong,false);
        let page_size_value = Value::const_int(wasm32_int_type, PAGE_SIZE as u64, false);
        let grow_size = builder.build_mul(page_size_value,delta,"grow_size");
        let linear_memory = set_linear_memory(module);
        let linear_memory_size = set_linear_memory_size(module,wasm_align);
        let linear_memory_size_cache = builder.build_load(linear_memory_size,"linear_memory_size_cache");
        let new_linear_memory_size = builder.build_add(linear_memory_size_cache,builder.build_int_cast(grow_size,i64_type,"") ,"new_linear_memory_size");

        let grow_bb = BasicBlock::append_basic_block(context,function,"grow_bb");
        let cant_grow_bb = BasicBlock::append_basic_block(context,function,"cant_grow_bb");
        builder.build_cond_br(builder.build_icmp(LLVMIntPredicate::LLVMIntUGE,max_linear_memory_size,new_linear_memory_size,"cmp_result"),grow_bb,cant_grow_bb);

        builder.position_builder_at_end(grow_bb);
        let linear_memory_cache = builder.build_load(linear_memory,"linear_memory_cache");
        let i32_type = Type::int32(context);
        let int_type = Type::int_ptr(context);
        let void_type = Type::void(context);
        let void_ptr_type = Type::ptr(void_type, 0);
        let param_types = [void_ptr_type,wasm32_int_type,i32_type,i32_type,i32_type,i32_type];
        let mmap_type = Type::function(void_ptr_type,&param_types,true);
        let mmap_function = module.set_function("mmap",mmap_type);
        let prot_value = Value::const_int(i32_type,(::libc::PROT_READ | ::libc::PROT_WRITE) as ::libc::c_ulonglong,true);
        let flags_value = Value::const_int(i32_type, (::libc::MAP_PRIVATE | ::libc::MAP_ANONYMOUS) as ::libc::c_ulonglong,true );
        let fd_value = Value::const_int(i32_type,-1_isize as ::libc::c_ulonglong,true);
        let offset_value = Value::const_int(i32_type,0,true);
        let addr_value = builder.build_int_to_ptr(
            builder.build_add(builder.build_ptr_to_int(linear_memory_cache,int_type,""),builder.build_int_cast( linear_memory_size_cache,int_type,""),""),
            void_ptr_type,
            "addr_value"
        );
        let args = [addr_value,grow_size,prot_value,flags_value,fd_value,offset_value];
        let mapped_ptr = builder.build_call(mmap_function,&args,"mapped_ptr");
        builder.build_store(mapped_ptr,linear_memory);
        builder.build_store(new_linear_memory_size,linear_memory_size);
        builder.build_ret( builder.build_int_cast( linear_memory_size_cache,wasm32_int_type,""));

        builder.position_builder_at_end(cant_grow_bb);
        builder.build_ret(Value::const_int(wasm32_int_type,-1_isize as ::libc::c_ulonglong,true));
        Ok(())
    })
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
        let result = build_grow_linear_memory_function(&module,&builder,32,Some(25));
        assert!(result.is_ok());
        let analysis_result = analysis::verify_module(&module,analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction);
        assert!(analysis_result.is_ok());


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