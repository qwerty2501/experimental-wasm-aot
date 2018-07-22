

use parity_wasm::elements::Module as ParityWasmModule;
use super::llvm::*;
use failure::Error;
use std::str;
pub const WASM_FUNCTION_PREFIX:&str = "WASM_FUNCTION_";

pub fn to_wasm_call_name(name:&str)->String{
    [WASM_FUNCTION_PREFIX,name].concat()
}


pub fn compile<'c>(module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {

    let builder = Builder::new(context);
    let module = Module::new(module_id, context);

    Ok(module)
}