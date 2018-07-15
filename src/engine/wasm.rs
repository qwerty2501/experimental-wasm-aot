
extern crate parity_wasm;
use self::parity_wasm::elements::Module as ParityWasmModule;
use super::compiler::*;
use failure::Error;

pub fn compile<'c>(module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {

    let builder = Builder::new(context);
    let module = Module::new(module_id, context);

    Ok(module)
}