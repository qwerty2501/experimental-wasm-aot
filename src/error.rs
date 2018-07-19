



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

    #[fail(display = "fatal analysis llvm: {}",message)]
    FatalLLVMAnalysis {
        message:String,
    }
}
