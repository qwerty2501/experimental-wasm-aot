use super ::*;
use parity_wasm::elements::Module as WasmModule;
use parity_wasm::elements::{Error as WasmError};
use failure::Error;
use error::RuntimeError::*;
use engine::llvm::target::*;
use engine::llvm::target_machine::*;
use std::path::Path;
use std::ffi::OsStr;
use std::process::Command;
use parity_wasm;

pub struct Engine<T:  WasmIntType>{
    wasm_compiler: WasmCompiler<T>,
    linear_memory_compiler: LinearMemoryCompiler<T>,
    function_table_compiler: FunctionTableCompiler<T>,
}

pub struct BuildWasmOptions<'a>{
    wasm_file_paths:&'a[&'a Path],
    output_file_path:&'a Path,
}

impl<'a> BuildWasmOptions<'a>{
    pub fn new(wasm_file_paths:&'a[&'a Path],output_file_path:&'a Path)-> BuildWasmOptions<'a>{
        BuildWasmOptions { wasm_file_paths,output_file_path }
    }
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
    pub fn build(&self, option:&BuildWasmOptions) ->Result<(),Error>{
        initialize_native_target()?;
        initialize_native_asm_printer()?;

        let wasm_modules = option.wasm_file_paths.iter().map(parity_wasm::deserialize_file).collect::<Result<Vec<WasmModule>,WasmError>>()?;
        let wasm_module = wasm_modules.first().ok_or(NotExistWasmFilePath)?;
        let context = Context::new();
        let default_module_name = "main_module";
        let module_id = option.output_file_path.file_stem().unwrap_or(OsStr::new(default_module_name)).to_str().unwrap_or(default_module_name);
        let module = self.wasm_compiler.compile(module_id,wasm_module,&context)?;



        let triple = get_default_target_triple()?;
        let target = Target::get_target_from_triple(&triple )?;
        let target_machine = TargetMachine::create_target_machine(target,&triple,"generic","",CodeGenOptLevel::LLVMCodeGenLevelDefault,RelocMode::LLVMRelocPIC,CodeModel::LLVMCodeModelDefault);

        let object_file_path = option.output_file_path.parent().unwrap_or(Path::new("")).join([module_id,".o"].concat());
        target_machine.emit_to_file(&module, object_file_path.to_str().ok_or(NotExistObjectPath)?, CodeGenFileType::LLVMObjectFile)?;
        Command::new("gcc")
            .args(["-o",option.output_file_path.to_str().ok_or(NotExistOutputFilePath)?,object_file_path.to_str().ok_or(NotExistObjectPath)?].iter())
            .status()?;
        Ok(())
    }

}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    pub fn build_add_2_3_works()->Result<(),Error>{
        let engine = Engine::<u32>::new();
        engine.build(&BuildWasmOptions::new(&[Path::new("target/test_cases/engine/add_2_3/add_2_3.wasm")],Path::new("target/test_cases/engine/add_2_3/add_2_3")))
    }
}