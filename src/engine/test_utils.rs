use super::llvm::*;
use super::llvm::execution_engine::*;
use super::llvm::target::*;
use failure::*;
use error::RuntimeError::*;

pub fn test_jit_init()->Result<(),Error>{
    link_in_mc_jit();
    initialize_native_target()?;
    initialize_native_asm_printer()?;
    Ok(())
}


pub fn test_run_function_with_name<'a>(engine:&ExecutionEngine, module:&'a Module,function_name:&str,args:&[&GenericValue])->Result<GenericValueGuard<'a>,Error>{
    let function = module.get_named_function(function_name).ok_or_else(||NoSuchLLVMFunction{ function_name:function_name.to_string()})?;
    Ok(engine.run_function(function,args))
}