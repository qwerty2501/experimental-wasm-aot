extern crate parity_wasm;
extern crate failure;
use self::parity_wasm::elements::Module as ParityWasmModule;
use failure::Error;
use super::linear_memory as memory;
pub struct Engine;
use super::compiler::*;

impl  Engine{
    pub fn build( &self ,wasm_module:&ParityWasmModule,context:&mut Context)->Result<Guard<Module>,Error>{
        memory::compile(context)
    }
}