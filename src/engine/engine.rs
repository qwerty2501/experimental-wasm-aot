use super ::*;
use parity_wasm::elements::Module as WasmModule;
use failure::Error;
use error::RuntimeError::*;

pub struct Engine<T:  WasmIntType>{
    wasm_compiler: WasmCompiler<T>,
    linear_memory_compiler: LinearMemoryCompiler<T>,
    function_table_compiler: FunctionTableCompiler<T>,
}
impl<'a,T:WasmIntType>  Engine<T>{

    pub fn new()->Engine<T>{
        let  linear_memory_compiler =  memory::LinearMemoryCompiler::<T>::new();
        Engine{
            linear_memory_compiler,
            wasm_compiler:WasmCompiler::<T>::new(),
            function_table_compiler:FunctionTableCompiler::<T>::new()
        }
    }
    pub fn build( &self ,wasm_module:&WasmModule)->Result<(),Error>{
        let context = Context::new();
        self.wasm_compiler.compile("main_module",wasm_module,&context)?;
        Ok(())
    }




}