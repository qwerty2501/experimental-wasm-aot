
use super::llvm::*;

pub struct BuildContext<'a>{
    context:&'a Context,
    module:ModuleGuard<'a>,
    builder:BuilderGuard<'a>,
}

impl<'a> BuildContext<'a>{
    pub fn new<'c>(module_id:&str, context:&'c Context)->BuildContext<'c>{
        BuildContext{context,module:Module::new(module_id,context),builder:Builder::new(context)}
    }

    pub fn move_module(self)->ModuleGuard<'a>{
        self.module
    }

    #[inline]
    pub fn context(&self)->&Context{
        self.context
    }

    #[inline]
    pub fn module(&self)->&Module{
        &self.module
    }

    #[inline]
    pub fn builder(&self)->&Builder{
        &self.builder
    }
}