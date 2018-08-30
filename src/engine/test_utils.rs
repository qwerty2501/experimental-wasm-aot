use super::*;
use super::llvm::execution_engine::*;
use super::llvm::target::*;
use failure::*;
use error::RuntimeError::*;
use std::path::Path;
use std::env;
use std::path::PathBuf;
use parity_wasm::elements::Module as WasmModule;
#[cfg(test)]
pub fn get_target_dir()->Result<PathBuf,Error>{
   env::var("CARGO_TARGET_DIR").map(|v|Ok(Path::new(&v).to_path_buf())).unwrap_or_else(|_| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
        let manifest_dir = Path::new(&manifest_dir);
        Ok(manifest_dir.join("target"))
    })
}

#[cfg(test)]
pub fn build_test_function<F:FnOnce(& Builder,& BasicBlock) -> Result<(),Error>>(build_context:&BuildContext,function_name:&str,args:&[&Value],on_build:F)->Result<(),Error>{
    build_test_function_with_return(build_context,function_name,Type::void(build_context.context()),args,on_build)
}

#[cfg(test)]
pub fn build_test_function_with_return<F:FnOnce(& Builder,& BasicBlock) -> Result<(),Error>>(build_context:&BuildContext,function_name:&str, return_type:&Type, args:&[&Value],on_build:F)->Result<(),Error>{
    let function = build_context.module().set_declare_function(function_name,Type::function(return_type,&args.iter().map(|v|Type::type_of(v)).collect::<Vec<_>>(),false));
    build_context.builder().build_function(build_context.context(),function,on_build)
}

#[cfg(test)]
fn init_test_jit() ->Result<(),Error>{
    link_in_mc_jit();
    initialize_native_target()?;
    initialize_native_asm_printer()?;
    Ok(())
}

#[cfg(test)]
pub fn test_module_in_engine<F:FnOnce(&ExecutionEngine)->Result<(),Error>>(module:&Module,f:F)->Result<(),Error>{
    analysis::verify_module(module,analysis::VerifierFailureAction::LLVMReturnStatusAction)?;
    init_test_jit()?;
    let engine = ExecutionEngine::new_for_module(module)?;
    f(&engine)?;
    engine.remove_module(module)?;
    Ok(())
}

#[cfg(test)]
pub fn test_module_main_in_engine(module:&Module,expected:i32)->Result<(),Error>{
    test_module_in_engine(module,|engine|{
        let result =  run_test_function_with_name(engine, module, "main", &[])?;
        assert_eq!(expected,result.to_int(false) as i32);
        Ok(())
    })
}

#[cfg(test)]
pub fn run_test_function_with_name<'a>(engine:&ExecutionEngine, module:&'a Module, function_name:&str, args:&[&GenericValue]) ->Result<GenericValueGuard<'a>,Error>{
    let function = module.get_named_function(function_name).ok_or_else(||NoSuchLLVMFunction{ name:function_name.to_string()})?;
    Ok(engine.run_function(function,args))
}

