extern crate parity_wasm;
extern crate failure;
use self::parity_wasm::elements::Module as ParityWasmModule;
use failure::Error;
use super::linear_memory as memory;
pub struct Engine;
use super::compiler::*;

impl  Engine{
    pub fn build( &self ,wasm_module:&ParityWasmModule)->Result<(),Error>{
        let context = Context::new();
        memory::compile(&context)?;
        Ok(())
    }
}