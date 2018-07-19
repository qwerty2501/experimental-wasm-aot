

use parity_wasm::elements::Module as ParityWasmModule;
use super::llvm::*;
use failure::Error;
use std::str;
pub struct WasmCallFunctionNameStr(str);

macro_rules! wasm_function_prefix {
    ()=>("__experimental_wasm_call_")
}
pub const WASM_CALL_PREFIX:&str = wasm_function_prefix!();
macro_rules! wasm_function_name {
    ($name:expr)=>(unsafe{ ::std::mem::transmute::<&str,&WasmCallFunctionNameStr>(concat!(wasm_function_prefix!(),$name))})
}



pub fn compile<'c>(module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {

    let builder = Builder::new(context);
    let module = Module::new(module_id, context);

    Ok(module)
}