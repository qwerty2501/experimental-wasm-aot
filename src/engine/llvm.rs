use llvm_sys::prelude::*;
use llvm_sys::core::*;
use std::ffi::CString;
use std::ops::{Deref};
use super::constants;
pub use llvm_sys::LLVMIntPredicate;
use failure::Error;
use error::RuntimeError::*;
use std::mem;
use engine::types::WasmIntType;
macro_rules! compiler_c_str{
    ($s:expr) => (CString::new($s).unwrap().as_ptr())
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
        ModuleGuard::new( unsafe{ LLVMModuleCreateWithNameInContext(compiler_c_str!(module_id), context.into()).into()} )
    }
    pub fn context(&self)-> &Context {
        unsafe{LLVMGetModuleContext(self.into()).into()}
    }

    pub fn set_global(&self,name:&str,type_ref:&Type)->&Value{
        self.get_named_global(name).unwrap_or_else(|| self.add_global(name,type_ref))
    }
    pub fn get_named_global(&self,name:&str)->Option<&Value>{
        unsafe {
            to_optional_ref(LLVMGetNamedGlobal(self.into(),compiler_c_str!(name)))
        }
    }

    fn add_global(&self,name:&str,type_ref:&Type)->&Value{
        unsafe{
            LLVMAddGlobal(self.into(),type_ref.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn set_function(&self,name:&str,type_ref:&Type)->&Value{
        self.get_named_function(name).unwrap_or_else(||self.add_function(name,type_ref))
    }

    pub fn get_named_function(&self,name:&str)->Option<&Value>{
        unsafe{
            to_optional_ref(LLVMGetNamedFunction(self.into(),compiler_c_str!(name)))
        }
    }

    pub fn dump(&self){
        unsafe{LLVMDumpModule(self.into())}
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
            ContextGuard::new( LLVMContextCreate().into())
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

        BuilderGuard::new(unsafe{LLVMCreateBuilderInContext(context.into()).into()})
    }

    pub fn position_builder_at_end(&self,bb:&BasicBlock){
        unsafe{LLVMPositionBuilderAtEnd(self.into(),bb.into())}
    }

    pub fn build_call(&self,func:&Value,args:&[&Value],name:&str)-> &Value{
        unsafe{

            LLVMBuildCall(self.into(),func.into(),args.as_ptr()  as *mut _,args.len() as u32,compiler_c_str!(name)).into( )
        }
    }

    pub fn build_ptr_to_int(&self,ptr_ref:&Value,int_type:&Type,name:&str)->&Value{
        unsafe{LLVMBuildPtrToInt(self.into(),ptr_ref.into(),int_type.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_int_to_ptr(&self,int_ref:&Value,ptr_type:&Type,name:&str)->&Value{
        unsafe{LLVMBuildIntToPtr(self.into(),int_ref.into(),ptr_type.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_int_cast(&self,value_ref:&Value,dst_ty:&Type,name:&str)->&Value{
        unsafe{LLVMBuildIntCast(self.into(),value_ref.into(),dst_ty.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_function<'a,F:FnOnce(&'a Builder,&'a BasicBlock) -> Result<(),Error>>(&'a self,context:&'a Context, func:& Value,on_build:F)->Result<(),Error>{
        let bb = func.append_basic_block(context,"entry");
        self.position_builder_at_end(bb);
        on_build(self,bb)
    }

    pub fn build_pointer_cast(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{LLVMBuildPointerCast(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_ret_void(&self)->&Value{
        unsafe{LLVMBuildRetVoid(self.into()).into()}
    }
    pub fn build_ret(&self,v:&Value)->&Value {
        unsafe{LLVMBuildRet(self.into(),v.into()).into()}
    }
    pub fn build_gep(&self,pointer:&Value,indices:&[&Value],name:&str)->&Value{
        unsafe{LLVMBuildGEP(self.into(),pointer.into(),indices.as_ptr() as *mut _,indices.len() as u32 ,compiler_c_str!(name)).into()}
    }
    pub fn build_icmp(&self,int_predicate:LLVMIntPredicate,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{LLVMBuildICmp(self.into(),int_predicate,lhs.into(),rhs.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_cond_br(&self,if_value:&Value,then_block:&BasicBlock,else_block:&BasicBlock)->&Value{
        unsafe{LLVMBuildCondBr(self.into(),if_value.into(),then_block.into(),else_block.into()).into()}
    }

    pub fn build_add(&self,lhs:&Value,rhs:&Value,name:&str)-> &Value{
        unsafe{LLVMBuildAdd(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_mul(&self,lhs:&Value,rhs:&Value,name:&str)-> &Value{
        unsafe{LLVMBuildMul(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_load(&self,pointer_value:&Value,name:&str)->&Value{
        unsafe{LLVMBuildLoad(self.into(),pointer_value.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_store(&self,value:&Value,ptr:&Value)->&Value{
        unsafe{LLVMBuildStore(self.into(),value.into(),ptr.into()).into()}
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

    pub fn const_vector<'a>(scalar_const_values:&'a [&Value])->&'a Value{
        unsafe{LLVMConstVector(scalar_const_values.as_ptr() as *mut _,scalar_const_values.len() as  ::libc::c_uint).into()}
    }

    pub fn const_int(type_ref:&Type, value: ::libc::c_ulonglong, sign_extend: bool) ->&Value{
        unsafe{
            LLVMConstInt(type_ref.into(),value,sign_extend as LLVMBool).into( )
        }
    }

    pub fn const_real(type_ref:&Type,n: ::libc::c_double)->&Value{
        unsafe{
            LLVMConstReal(type_ref.into(),n.into()).into()
        }
    }

    pub fn const_null(type_ref:&Type)->&Value{
        unsafe{
            LLVMConstNull(type_ref.into()).into()
        }
    }

    pub fn set_global_const(&self,is_constant:bool){
        unsafe{
            LLVMSetGlobalConstant(self.into(),is_constant as LLVMBool)
        }
    }



    pub fn set_initializer(&self,constant_value:&Value){
        unsafe{
            LLVMSetInitializer(self.into(),constant_value.into())
        }
    }

    pub fn const_int_to_ptr<'a>(value:&'a Value,to_type:&Type)->&'a Value{
        unsafe{
            LLVMConstIntToPtr(value.into(),to_type.into()).into()
        }
    }



    pub fn null_ptr(type_ref:&Type)->&Value {
        unsafe{LLVMConstPointerNull(type_ref.into()).into()}
    }

    pub fn get_params(&self)->Vec<&Value>{
        unsafe{
            let count = LLVMCountParams(self.into());
            let vec =Vec::<&Value>::with_capacity(count as usize);
            if vec.capacity() > 0{
                LLVMGetParams(self.into(),vec.as_ptr() as *mut _);
            }
            vec
        }
    }

    pub fn get_first_param(&self) ->Option<&Value>{
        unsafe{to_optional_ref(LLVMGetFirstParam(self.into()))}
    }
    pub fn get_next_param(&self)->Option<&Value>{
        unsafe{to_optional_ref(LLVMGetNextParam(self.into()))}
    }
    pub fn append_basic_block<'c>(&self,context:&'c Context,name:&str)->&'c BasicBlock{
        unsafe{LLVMAppendBasicBlockInContext(context.into(),self.into(),compiler_c_str!(name)).into()}
    }
}

pub enum Type{}
impl_type_traits!(Type,LLVMTypeRef);
impl Type{
    pub fn int1(context:&Context)->&Type{
        unsafe{LLVMInt1TypeInContext(context.into()).into()}
    }
    pub fn int8(context:&Context)->&Type{
        unsafe {LLVMInt8TypeInContext(context.into()).into()}
    }
    pub fn int32(context:&Context)->&Type {
        unsafe{LLVMInt32TypeInContext(context.into()).into()}
    }

    pub fn int64(context:&Context)->&Type{
        unsafe{LLVMInt64TypeInContext(context.into()).into()}
    }

    pub fn float32(context:&Context) ->&Type{
        unsafe{LLVMFloatTypeInContext(context.into()).into()}
    }

    pub fn float64(context:&Context) ->&Type{
        unsafe{LLVMDoubleTypeInContext(context.into()).into()}
    }

    pub fn int(context:&Context,num_bits: ::libc::c_uint) -> &Type{
        unsafe {LLVMIntTypeInContext(context.into(),num_bits).into( )}
    }

    pub fn int_ptr(context:&Context)->&Type{
        Type::int(context, constants::CPU_BIT_WIDTH as ::libc::c_uint)
    }

    pub fn int_wasm_ptr<T:WasmIntType>(context:&Context) ->&Type{
        Type::int(context,constants::bit_width::<T>() as ::libc::c_uint)
    }
    pub fn void(context:&Context)->&Type{
        unsafe{LLVMVoidTypeInContext(context.into()).into()}
    }

    pub fn ptr(type_ref:&Type, address_space: ::libc::c_uint) ->& Type {
        unsafe {LLVMPointerType(type_ref.into(),address_space).into( )}
    }

    pub fn function<'a>(return_type:&'a Type,param_types:&'a[&'a Type],is_var_arg:bool)->&'a Type{
        unsafe{LLVMFunctionType(return_type.into(),param_types.as_ptr() as *mut _,param_types.len() as ::libc::c_uint,is_var_arg as LLVMBool).into()}
    }

    pub fn type_of(value:&Value)->&Type{
        unsafe{LLVMTypeOf(value.into()).into()}
    }
}

pub enum BasicBlock{}
impl_type_traits!(BasicBlock,LLVMBasicBlockRef);

impl BasicBlock{
    pub fn insert_basic_block(&self,context:& Context,name:&str)->&BasicBlock{
        unsafe{LLVMInsertBasicBlockInContext(context.into(), self.into(),compiler_c_str!(name)).into()}
    }
}


pub struct BuildCallAndSetResult<'a>{function:&'a  Value,return_value:&'a  Value}


pub fn build_call_and_set_mmap<'m>(module:&'m  Module,builder:&'m Builder,addr:&Value,length:&Value,plot:&Value,flags:&Value,fd:&Value,offset:&Value,name:&str)->&'m Value{
    let context = module.context();
    let void_ptr_type = Type::ptr(Type::void(context),0);
    let int32_type = Type::int32(context);
    let param_types = [void_ptr_type,Type::int_ptr(context),int32_type,int32_type,int32_type,int32_type];
    let mmap_type = Type::function(void_ptr_type,&param_types,false);
    let mmap = module.set_function("mmap",&mmap_type);
    let args = [addr,length,plot,flags,fd,offset];
    builder.build_call(mmap,&args,name)
}

pub fn build_call_and_set_munmap<'m>(module:&'m Module,builder:&'m Builder,addr:&Value,length:&Value,name:&str)->&'m Value{
    let context =module.context();
    let void_ptr_type = Type::ptr(Type::void(context),0);
    let param_types = [void_ptr_type,Type::int_ptr(context)];
    let munmap_type = Type::function(Type::int32(context),&param_types,false);
    let munmap = module.set_function("munmap",munmap_type);
    let args =[addr,length];
    builder.build_call(munmap,&args,name)
}


pub fn build_call_and_set_memcpy<'m>(module:&'m Module,builder:&'m Builder,dest:&Value,src:&Value,n:&Value,name:&str)->&'m Value{
    let context = module.context();
    let void_ptr_type = Type::ptr(Type::void(context),0);
    let param_types = [void_ptr_type,void_ptr_type,Type::int_ptr(context)];
    let memcpy_type = Type::function(void_ptr_type,&param_types,false);
    let memcpy = module.set_function("memcpy",memcpy_type);
    let args = [dest,src,n];
    builder.build_call(memcpy,&args,name)
}

pub fn build_call_and_set<'m>(module:&'m  Module, builder:&'m Builder, args:&[&Value], function_name:&str, type_ref:& Type,return_name:&str) -> BuildCallAndSetResult<'m>{
    let function = module.set_function(function_name,type_ref);
    BuildCallAndSetResult {function, return_value: builder.build_call(&function, args, return_name)}
}









pub trait Disposable{
    fn dispose(&mut self);
}

pub struct Guard<'a,T:'a + Disposable>{
    source:&'a mut T
}

impl <'a,T:Disposable>  Guard<'a,T>{
    fn new(source:&'a mut T)->Guard<'a,T>{
        Guard{source}
    }
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


pub trait AsPtr<T> {
    fn as_ptr(&self) ->T;
}


unsafe fn to_optional_ref<P,T >(ptr: *mut P)-> Option<&'static T>  where  &'static T:From<*mut P>  {
    if ptr.is_null() {
        None
    } else {
        Some(ptr.into())
    }
}
fn convert_message_to_string(message: *mut ::libc::c_char)->Result<String,Error>{
    unsafe{
        let ret_message = CString::from_raw(message).to_str()?.to_string();
        LLVMDisposeMessage(message);
        Ok(ret_message)
    }
}
pub mod analysis{
    use super::*;
    use llvm_sys::analysis::*;
    pub use llvm_sys::analysis::LLVMVerifierFailureAction;
    pub fn verify_module(module:&Module,verifier_failure_action:LLVMVerifierFailureAction)-> Result<(),Error>{

        unsafe{
            let mut out_message:*mut ::libc::c_char = mem::uninitialized();
            if LLVMVerifyModule(module.into(),verifier_failure_action,  &mut out_message as *mut _) != 0 {
                Err(FailureLLVMAnalysis {message:convert_message_to_string(out_message)?})?
            } else{
                Ok(())
            }

        }
    }
}

pub mod target{
    use super::*;
    use llvm_sys::target::*;
    pub fn initialize_native_target()->Result<(),Error>{
        unsafe{
            if LLVM_InitializeNativeTarget() != 0{
                Err(FailureLLVMInitializeNativeTarget)?
            } else{
                Ok(())
            }
        }
    }

    pub fn initialize_native_asm_printer()->Result<(),Error>{
        unsafe{
            if LLVM_InitializeNativeAsmPrinter() != 0{
                Err(FailureLLVMInitializeNativeAsmPrinter)?
            } else{
                Ok(())
            }
        }
    }
}

pub mod execution_engine {
    use super::*;
    use llvm_sys::execution_engine::*;
    use llvm_sys::target_machine::*;

    pub fn link_in_mc_jit(){
        unsafe{LLVMLinkInMCJIT()}
    }



    pub enum ExecutionEngine{}
    impl_type_traits!(ExecutionEngine,LLVMExecutionEngineRef);
    pub type ExecutionEngineGuard<'a> = Guard<'a,ExecutionEngine>;

    impl ExecutionEngine{

        pub fn new_for_module(module:&Module) ->Result<ExecutionEngineGuard,Error>{
            unsafe{
                let mut execution_engine_ptr:LLVMExecutionEngineRef = mem::uninitialized();
                let mut out_message:*mut ::libc::c_char = mem::uninitialized();
                if LLVMCreateExecutionEngineForModule(&mut execution_engine_ptr as *mut _,module.into(),&mut out_message as *mut _) != 0{
                    Err(FailureLLVMCreateExecutionEngine{message: convert_message_to_string(out_message)?})?
                } else{
                    Ok(ExecutionEngineGuard::new(execution_engine_ptr.into()))
                }
            }
        }


        pub fn get_global_value_ref_from_address<'a,T>(&'a self,name:&str)->&'a T{
            unsafe{
                ::std::mem::transmute(self.get_global_value_address(name))
            }
        }

        fn get_global_value_address(&self,name:&str)-> u64{
            unsafe{
                LLVMGetGlobalValueAddress(self.into(),compiler_c_str!(name))
            }
        }


        pub fn add_global_mapping<T>(&self,global:&Value,global_ref:&mut T){
            unsafe{
                LLVMAddGlobalMapping(self.into(),global.into(),global_ref as *mut T as *mut _)
            }
        }

        pub fn remove_module(&self,module:&Module)->Result<&Module,Error>{

            unsafe{
                let mut out_mod:LLVMModuleRef = mem::uninitialized();
                let mut out_message:*mut ::libc::c_char = mem::uninitialized();
                if LLVMRemoveModule(self.into(),module.into(),&mut out_mod as *mut _,&mut out_message as *mut _) != 0{
                    Err(FailureLLVMRemoveModule{message:convert_message_to_string(out_message)?})?
                } else{
                    Ok(out_mod.into())
                }
            }
        }


        pub fn run_function<'a>(&self,function:&Value,args:&[&GenericValue])->GenericValueGuard<'a>{
            GenericValueGuard::new( unsafe{ LLVMRunFunction(self.into(),function.into(),args.len() as ::libc::c_uint,args.as_ptr() as *mut _).into() })
        }
    }

    impl Disposable for ExecutionEngine{
        fn dispose(&mut self) {
           unsafe{LLVMDisposeExecutionEngine(self.into())}
        }
    }

    pub enum GenericValue {}
    impl_type_traits!(GenericValue,LLVMGenericValueRef);
    pub type GenericValueGuard<'a> = Guard<'a,GenericValue>;

    impl GenericValue{
        pub fn int_width(&self)->::libc::c_uint{
            unsafe{LLVMGenericValueIntWidth(self.into())}
        }
        pub fn value_of_int(ty:&Type,n: ::libc::c_ulonglong,is_signed:bool)->GenericValueGuard{
            GenericValueGuard::new( unsafe{LLVMCreateGenericValueOfInt(ty.into(),n,is_signed as LLVMBool).into()})
        }
    }

    impl Disposable for GenericValue{
        fn dispose(&mut self) {
            unsafe{LLVMDisposeGenericValue(self.into())}
        }
    }


}