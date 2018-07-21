

use parity_wasm::elements::Module as ParityWasmModule;
use super::llvm::*;
use failure::Error;
use std::str;
pub const WASM_CALL_PREFIX:&str = "WASM_CALL_";

pub fn to_wasm_call_name(name:&str)->String{
    [WASM_CALL_PREFIX,name].concat()
}


pub fn compile<'c>(module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {

    let builder = Builder::new(context);
    let module = Module::new(module_id, context);

    Ok(module)
}