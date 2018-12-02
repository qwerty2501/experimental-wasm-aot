

use failure::Error;
use num::*;
use parity_wasm::elements::Instruction;

#[derive(Debug,Fail)]
pub enum RuntimeError{

    #[fail(display = "error occurred : {}",name)]
    Application{
        name:String,
    },

    #[fail(display = "no such llvm function : {}",name)]
    NoSuchLLVMFunction {
        name:String,
    },
    #[fail(display = "no such llvm function parameter : {}",name)]
    NoSuchLLVMFunctionParameter {
        name:String,
    },

    #[fail(display = "no such llvm global value : {}",name)]
    NoSuchLLVMGlobalValue{
        name:String,
    },

    #[fail(display = "no such type index:{}",index)]
    NoSuchTypeIndex{
        index:u32,
    },
    #[fail(display = "no such function index:{}",index)]
    NoSuchFunctionIndex{
        index:u32,
    },

    #[fail(display = "no such local value index:{}",index)]
    NoSuchLocalValue{
      index:u32,
    },

    #[fail(display = "no such label index:{}",index)]
    NoSuchLabel{
        index:u32,
    },



    #[fail(display = "not exist frame")]
    NotExistFrame,
    #[fail(display = "not exist value")]
    NotExistValue,

    #[fail(display = "not exist type")]
    NotExistType,
    #[fail(display = "not exist memory")]
    NotExistMemory,

    #[fail(display = "not exist label")]
    NotExistLabel,


    #[fail(display = "not exist init expr")]
    NotExistInitExpr,

    #[fail(display = "not exist global initializer instruction")]
    NotExistGlobalInitializerInstruction,

    #[fail(display = "not exist object file path")]
    NotExistObjectPath,

    #[fail(display = "not exist output file path")]
    NotExistOutputFilePath,

    #[fail(display = "not exist wasm file path")]
    NotExistWasmFilePath,

    #[fail(display = "incorrect align : {}",align)]
    InCorrectAlign{
        align:u32,
    },

    #[fail(display = "invalid instruction : {}",instruction)]
    InvalidInstruction{
        instruction:Instruction,
    },

    #[fail(display = "invalid label type")]
    InvalidLabelType,

    #[fail(display = "invalid label type")]
    InvalidType,

    #[fail(display = "size is too large maximum:{}",message)]
    SizeIsTooLarge{
        message:String,
    },

    #[fail(display = "size is too small minimum:{}",message)]
    SizeIsTooSmall{
        message:String,
    },

    #[fail(display = "failure analysis llvm: {}",message)]
    FailureLLVMAnalysis {
        message:String,
    },

    #[fail(display = "failure create execution engine:{}",message)]
    FailureLLVMCreateExecutionEngine {
        message:String,
    },

    #[fail(display = "failure initialize native target")]
    FailureLLVMInitializeNativeTarget,
    #[fail(display = "failure initialize native asm printer")]
    FailureLLVMInitializeNativeAsmPrinter,

    #[fail(display = "failure emit llvm module:{}",message)]
    FailureEmitLLVMModule{
        message:String,
    },

    #[fail(display = "failure remove module :{}",message)]
    FailureLLVMRemoveModule{
        message:String,
    },

    #[fail(display = "no such llvm  target triple:{},message:{}",triple,message)]
    FailureGetLLVMTarget{
        triple:String,
        message:String,
    },
}


pub fn check_range<T: Integer + ::std::fmt::Display>(target:T,minimum:T,maximum:T,name:&str)->Result<(),Error>{
    if target < minimum {
        Err(RuntimeError::SizeIsTooSmall{message: format!("{}:{},{}:{}",name_of!(minimum),minimum, name,target)})?
    } else if target > maximum{
        Err(RuntimeError::SizeIsTooLarge {message:format!("{}:{},{}:{}",name_of!(maximum),maximum, name,target)})?
    } else{
        Ok(())
    }

}