



#[derive(Debug,Fail)]
pub enum RuntimeError{

    #[fail(display = "error occurred : {}",name)]
    Application{
        name:String,
    },

    #[fail(display = "the function parameter is not enough : {}",message)]
    FunctionParameterNotEnough{
        message:String,
    },

    #[fail(display = "fatal analysis llvm: {}",message)]
    FatalAnalysisLLVM{
        message:String,
    }
}
