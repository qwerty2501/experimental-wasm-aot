

use parity_wasm::elements::Module as ParityWasmModule;
use super::llvm::*;
use super::types::*;
use super::linear_memory::*;
use failure::Error;
use std::str;
pub const WASM_FUNCTION_PREFIX:&str = "__WASM_FUNCTION_";

pub struct WasmCompiler<T: WasmIntType>{
    linear_memory_compiler: LinearMemoryCompiler<T>
}

impl<T:WasmIntType> WasmCompiler<T>{

    pub fn new()->WasmCompiler<T>{
        WasmCompiler{ linear_memory_compiler: LinearMemoryCompiler::<T>::new()}
    }
    fn wasm_call_name(name:&str) ->String{
        [WASM_FUNCTION_PREFIX,name].concat()
    }


    pub fn compile<'c>(&self, module_id:&str,wasm_module:&ParityWasmModule,context:&'c Context)->Result<ModuleGuard<'c>,Error> {
        let builder = Builder::new(context);
        let module = Module::new(module_id, context);
        self.build_init_data_sections_function(wasm_module,&module,&builder)?;
        Ok(module)
    }

    pub fn set_init_data_sections_function<'c>(&self,module:&'c Module)->&'c Value{
        let context = module.context();
        let param_types:[&Type;0] = [];
        module.set_function("init_data_sections",Type::function(Type::int8(context),&param_types,true))
    }

    fn build_init_data_sections_function(&self,wasm_module:&ParityWasmModule,module:&Module,builder:&Builder)->Result<(),Error>{
        let function = self.set_init_data_sections_function(module);
        let context = module.context();
        builder.build_function(context,function,|builder,bb|{
            wasm_module.data_section().map_or(Ok(()),|data_section|{
               for segment in data_section.entries(){

               }
                Ok(())
            })
        })
    }
}
