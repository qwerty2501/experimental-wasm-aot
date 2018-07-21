

use failure::Error;
use num::*;
#[derive(Debug,Fail)]
pub enum RuntimeError{

    #[fail(display = "error occurred : {}",name)]
    Application{
        name:String,
    },

    #[fail(display = "no such llvm function parameter : {}",message)]
    NoSuchLLVMFunctionParameter {
        message:String,
    },

    #[fail(display = "size is too large. maximum:{}",message)]
    SizeIsTooLarge{
        message:String,
    },

    #[fail(display = "size is too small. minimum:{}",message)]
    SizeIsTooSmall{
        message:String,
    },

    #[fail(display = "fatal analysis llvm: {}",message)]
    FatalLLVMAnalysis {
        message:String,
    }
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