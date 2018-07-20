

use parity_wasm::elements::Module as ParityWasmModule;
use super::llvm::*;
use failure::Error;
use std::str;
pub struct WasmCallFunctionNameStr(str);


pub const WASM_CALL_PREFIX:&str = "__experimental_wasm_call_";



pub fn compile<'c>(module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {

    let builder = Builder::new(context);
    let module = Module::new(module_id, context);

    Ok(module)
}