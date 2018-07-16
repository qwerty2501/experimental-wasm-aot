use llvm_sys::prelude::*;
use llvm_sys::core::*;
use std::ffi::CString;
use std::ptr;
use std::ops::{Deref};



pub trait Disposable{
    fn dispose(&mut self);
}

pub struct Guard<'a,T:'a + Disposable>{
    source:&'a mut T
}



impl <'a,T:Disposable> Deref for Guard<'a,T>{
    type Target = T;

    fn deref(&self) -> &<Self as Deref>::Target {
        self.source
    }


}



impl<'a,T:Disposable> Drop for  Guard<'a,T> {
    fn drop(&mut self) {
        self.source.dispose();
    }
}

macro_rules! compiler_c_str{
    ($s:expr) => (CString::new($s).unwrap().as_ptr())
}
pub trait AsPtr<T> {
    fn as_ptr(&self) ->T;
}

macro_rules! impl_type_traits{
    ($ref_ty:ty,$pointer_ty:ty) =>(
    impl From<$pointer_ty> for  &'static  $ref_ty{
        fn from(p: $pointer_ty) -> Self {
            unsafe{::std::mem::transmute(p)}
        }
    }
    impl From<$pointer_ty> for  &'static mut  $ref_ty{
        fn from(p: $pointer_ty) -> Self {
            unsafe{::std::mem::transmute(p)}
        }
    }
    impl<'a> Into<$pointer_ty> for &'a  $ref_ty{
        fn into(self)->$pointer_ty{
            unsafe{::std::mem::transmute(self)}
        }
    }

     impl<'a> Into<$pointer_ty> for &'a mut $ref_ty{
        fn into(self)->$pointer_ty{
            self.as_ptr()
        }
    }


    impl AsPtr<$pointer_ty> for $ref_ty{
        fn as_ptr(&self)->$pointer_ty{
            unsafe{::std::mem::transmute(self)}
        }
    }
    )
}



pub enum Module{}

impl_type_traits!(Module,LLVMModuleRef);

pub type ModuleGuard<'c> = Guard<'c,Module>;

impl Module {
    pub fn new<'c>(module_id:&str, context:&'c Context) ->  ModuleGuard<'c> {
        ModuleGuard{source:unsafe{ LLVMModuleCreateWithNameInContext(compiler_c_str!(module_id), context.into()).into()} }
    }
    pub fn context(&self)-> &Context {
        unsafe{LLVMGetModuleContext(self.into()).into()}
    }

    pub fn set_global(&self,name:&str,type_ref:&Type)->&Value{
        let global_value = self.get_named_global(name);
        if global_value.as_ptr() != ptr::null_mut(){
            global_value
        } else {
            self.add_global(name,type_ref)
        }
    }
    pub fn get_named_global(&self,name:&str)->&Value{
        unsafe {
            LLVMGetNamedGlobal(self.into(),compiler_c_str!(name)).into()
        }
    }

    fn add_global(&self,name:&str,type_ref:&Type)->&Value{
        unsafe{
            LLVMAddGlobal(self.into(),type_ref.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn set_function(&self,name:&str,type_ref:&Type)->&Value{
        let function = self.get_named_function(name);
        if function.as_ptr() != ptr::null_mut(){
            function
        } else{
            self.add_function(name,type_ref)
        }
    }

    pub fn get_named_function(&self,name:&str)->&Value{
        unsafe{
            LLVMGetNamedFunction(self.into(),compiler_c_str!(name)).into( )
        }
    }

    fn add_function(&self,name:&str,type_ref:&Type)->&Value{
        unsafe{
            LLVMAddFunction(self.into(),compiler_c_str!(name),type_ref.into()).into( )
        }
    }
}

impl Disposable for Module {
    fn dispose(&mut self) {
        unsafe {
            LLVMDisposeModule(self.into());
        }
    }
}




pub enum Context{}
impl_type_traits!(Context,LLVMContextRef);
pub type ContextGuard<'c> = Guard<'c,Context>;
impl Context {
    pub fn new<'c>() -> ContextGuard<'c>{
        unsafe{
            ContextGuard{source: LLVMContextCreate().into()}
        }
    }
}

impl Disposable for Context{
    fn dispose(&mut self) {
        unsafe{
            LLVMContextDispose(self.into());
        }
    }
}




pub enum Builder {}
impl_type_traits!(Builder,LLVMBuilderRef);
pub type BuilderGuard<'c> = Guard<'c,Builder>;
impl Builder {
    pub fn new(context:& Context) -> BuilderGuard{
        BuilderGuard{source:unsafe{LLVMCreateBuilderInContext(context.into()).into()}}
    }

    pub fn build_call(&self,func:&Value,args:&[&Value],name:&str)-> &Value{
        unsafe{
            LLVMBuildCall(self.into(),func.into(),args.as_ptr()  as *mut _,args.len() as u32,compiler_c_str!(name)).into( )
        }
    }
}
impl Disposable for Builder{
    fn dispose(&mut self) {
        unsafe{
            LLVMDisposeBuilder(self.into());
        }
    }
}





pub enum Value{}
impl_type_traits!(Value,LLVMValueRef);
impl  Value{
    pub fn const_int(type_ref:&Type, value: ::libc::c_ulonglong, sign_extend: bool) ->&Value{
        unsafe{
            LLVMConstInt(type_ref.into(),value,sign_extend as LLVMBool).into( )
        }
    }
}

pub enum Type{}
impl_type_traits!(Type,LLVMTypeRef);
impl Type{
    pub fn int8(context:&Context)->&Type{
        unsafe {LLVMInt8TypeInContext(context.into() ).into( )}
    }

    pub fn int32(context:&Context)->&Type{
        unsafe{LLVMInt32TypeInContext(context.into()).into( )}
    }

    pub fn int(context:&Context,num_bits: ::libc::c_uint) -> &Type{
        unsafe {LLVMIntTypeInContext(context.into(),num_bits).into( )}
    }

    pub fn pointer(type_ref:&Type,address_space: ::libc::c_uint)->& Type {
        unsafe {LLVMPointerType(type_ref.into(),address_space).into( )}
    }
}


pub struct BuildAndSetCallResult<'a>{function:&'a  Value,return_value:&'a  Value}

pub fn build_and_set_call<'m>(module:&'m mut Module, builder:&'m mut Builder, args:&& [&Value], name:&str, type_ref:& Type) ->BuildAndSetCallResult<'m>{
    let function = module.set_function(name,type_ref);
    BuildAndSetCallResult{function, return_value: builder.build_call(&function,args,name)}
}