
extern crate parity_wasm;
use self::parity_wasm::elements::Module as ParityWasmModule;
use super::compiler::*;
use failure::Error;

pub fn compile(module_id:&str,wasm_module:&ParityWasmModule,context:&mut  Context)->Result<Guard<Module>,Error> {

    let builder = Builder::new(context);
    let module = Module::new(module_id, context);

    Ok(module)
}