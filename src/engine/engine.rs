extern crate parity_wasm;
extern crate failure;
use self::parity_wasm::elements::Module as ParityWasmModule;
use failure::Error;
use super::linear_memory as memory;
pub struct Engine;
use super::llvm::*;

impl  Engine{
    pub fn build( &self ,wasm_module:&ParityWasmModule)->Result<(),Error>{
        let context = Context::new();
        let liner_memory_compiler = memory::LinearMemoryCompiler::<i32>::new();
        liner_memory_compiler.compile(&context,17,Some(25))?;
        Ok(())
    }
}