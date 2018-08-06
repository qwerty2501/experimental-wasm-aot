use super::*;
use super::llvm::execution_engine::*;
use super::llvm::target::*;
use failure::*;
use error::RuntimeError::*;

pub fn build_test_function<F:FnOnce(& Builder,& BasicBlock) -> Result<(),Error>>(build_context:&BuildContext,function_name:&str,args:&[&Value],on_build:F)->Result<(),Error>{
    build_test_function_with_return(build_context,function_name,Type::void(build_context.context()),args,on_build)
}

pub fn build_test_function_with_return<F:FnOnce(& Builder,& BasicBlock) -> Result<(),Error>>(build_context:&BuildContext,function_name:&str, return_type:&Type, args:&[&Value],on_build:F)->Result<(),Error>{
    let function = build_context.module().set_declare_function(function_name,Type::function(return_type,&args.iter().map(|v|Type::type_of(v)).collect::<Vec<_>>(),false));
    build_context.builder().build_function(build_context.context(),function,on_build)
}

pub fn init_test_jit() ->Result<(),Error>{
    link_in_mc_jit();
    initialize_native_target()?;
    initialize_native_asm_printer()?;
    Ok(())
}

pub fn test_module_in_engine<F:FnOnce(&ExecutionEngine)->Result<(),Error>>(module:&Module,f:F)->Result<(),Error>{
    let engine = ExecutionEngine::new_for_module(module)?;
    f(&engine)?;
    engine.remove_module(module)?;
    Ok(())
}
pub fn run_test_function_with_name<'a>(engine:&ExecutionEngine, module:&'a Module, function_name:&str, args:&[&GenericValue]) ->Result<GenericValueGuard<'a>,Error>{
    let function = module.get_named_function(function_name).ok_or_else(||NoSuchLLVMFunction{ name:function_name.to_string()})?;
    Ok(engine.run_function(function,args))
}