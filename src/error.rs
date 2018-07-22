

use failure::Error;
use num::*;
#[derive(Debug,Fail)]
pub enum RuntimeError{

    #[fail(display = "error occurred : {}",name)]
    Application{
        name:String,
    },

    #[fail(display = "no such llvm function : {}",function_name)]
    NoSuchLLVMFunction {
        function_name:String,
    },
    #[fail(display = "no such llvm function parameter : {}",parameter_name)]
    NoSuchLLVMFunctionParameter {
        parameter_name:String,
    },

    #[fail(display = "size is too large. maximum:{}",message)]
    SizeIsTooLarge{
        message:String,
    },

    #[fail(display = "size is too small. minimum:{}",message)]
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

    #[fail(display = "failure initialize native target.")]
    FailureLLVMInitializeNativeTarget,
    #[fail(display = "failure initialize native asm printer.")]
    FailureLLVMInitializeNativeAsmPrinter,
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