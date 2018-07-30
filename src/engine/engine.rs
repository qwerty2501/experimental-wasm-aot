extern crate parity_wasm;
extern crate failure;
use self::parity_wasm::elements::Module as WasmModule;
use failure::Error;
use super::linear_memory as memory;
use super::wasm;
use error::RuntimeError::*;
use super::llvm::*;
use engine::types::WasmIntType;
use self::parity_wasm::elements::External;


pub struct Engine<T:  WasmIntType>{
    wasm_compiler:wasm::WasmCompiler<T>,
    linear_memory_compiler:memory::LinearMemoryCompiler<T>,
}
impl<'a,T:WasmIntType>  Engine<T>{

    pub fn new()->Engine<T>{
        let  linear_memory_compiler =  memory::LinearMemoryCompiler::<T>::new();
        Engine{
            linear_memory_compiler,
            wasm_compiler:wasm::WasmCompiler::<T>::new(),
        }
    }
    pub fn build( &self ,wasm_module:&WasmModule)->Result<(),Error>{
        let context = Context::new();
        self.linear_memory_compiler.compile(&context,wasm_module)?;
        self.wasm_compiler.compile("main_module",wasm_module,&context)?;
        Ok(())
    }
}