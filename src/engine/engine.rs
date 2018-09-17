use super ::*;
use parity_wasm::elements::Module as WasmModule;
use failure::Error;
use error::RuntimeError::*;
use engine::llvm::target::*;
use engine::llvm::target_machine::*;
use std::path::Path;
use std::ffi::OsStr;
use std::process::Command;

pub struct Engine<T:  WasmIntType>{
    wasm_compiler: WasmCompiler<T>,
    linear_memory_compiler: LinearMemoryCompiler<T>,
    function_table_compiler: FunctionTableCompiler<T>,
}

pub struct BuildWasmOption<'a>{
    output_file_path:&'a Path,
}

impl<'a> BuildWasmOption<'a>{
    pub fn new(output_file_path:&'a Path)->BuildWasmOption<'a>{
        BuildWasmOption{ output_file_path }
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
    pub fn build( &self ,wasm_module:&WasmModule,option:&BuildWasmOption)->Result<(),Error>{
        let context = Context::new();
        let default_module_name = "main_module";
        let module_id = option.output_file_path.file_stem().unwrap_or(OsStr::new(default_module_name)).to_str().unwrap_or(default_module_name);
        let module = self.wasm_compiler.compile(module_id,wasm_module,&context)?;

        initialize_native_target()?;
        initialize_native_asm_printer()?;

        let triple = get_default_target_triple()?;
        let target = Target::get_target_from_name(&triple ).ok_or(NoSuchLLVMTarget{triple:triple.clone()})?;
        let target_machine = TargetMachine::create_target_machine(target,&triple,"generic","",CodeGenOptLevel::LLVMCodeGenLevelDefault,RelocMode::LLVMRelocDefault,CodeModel::LLVMCodeModelDefault);

        let object_file_path = option.output_file_path.parent().unwrap_or(Path::new("")).join(module_id).join(".o");
        target_machine.emit_to_file(&module, object_file_path.to_str().ok_or(NotExistObjectPath)?, CodeGenFileType::LLVMObjectFile)?;
        Command::new("gcc").arg("-o").arg(option.output_file_path.to_str().ok_or(NotExistOutputFilePath)?).arg(object_file_path.to_str().ok_or(NotExistObjectPath)?);
        Ok(())
    }




}