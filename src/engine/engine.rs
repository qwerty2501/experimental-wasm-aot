extern crate parity_wasm;
extern crate failure;
use self::parity_wasm::elements::Module as ParityWasmModule;
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
    pub fn build( &self ,wasm_module:&ParityWasmModule)->Result<(),Error>{
        let context = Context::new();
        let import_memory_count = wasm_module.import_section().map_or(0,|section|{
            section.entries().iter().filter(|p|match p.external() {
                External::Memory(_) =>true,
                _=>false,
            }).count()
        });
        for (index,segment) in wasm_module.memory_section().ok_or(NotExistMemorySection)?.entries().iter().enumerate(){
            let memory_limits = segment.limits();
            self.linear_memory_compiler.compile(&context,import_memory_count + index,  memory_limits.initial() ,memory_limits.maximum())?;
        }
        self.wasm_compiler.compile("main_module",wasm_module,&context)?;
        Ok(())
    }
}