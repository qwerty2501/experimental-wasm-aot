use llvm_sys::prelude::*;
use llvm_sys::core::*;
use std::ffi::CString;
use std::ops::{Deref};
use super::constants;
pub use llvm_sys::{LLVMIntPredicate as IntPredicate,LLVMRealPredicate as RealPredicate,LLVMLinkage as Linkage , LLVMOpcode as Opcode};
use failure::Error;
use error::RuntimeError::*;
use std::mem;
use engine::types::WasmIntType;
use parity_wasm::elements::ValueType;

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
    impl PartialEq for $ref_ty{
        fn eq(&self, other: & $ref_ty) -> bool {
            self as *const _ == other as *const _
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

    pub fn set_declare_global(&self, name:&str, type_ref:&Type) ->&Value{
        self.get_named_global(name).unwrap_or_else(|| self.add_global(name,type_ref))
    }
    pub fn get_named_global(&self,name:&str)->Option<&Value>{
        unsafe {
            ptr_to_optional_ref(LLVMGetNamedGlobal(self.into(), compiler_c_str!(name)))
        }
    }

    fn add_global(&self,name:&str,type_ref:&Type)->&Value{
        unsafe{
            LLVMAddGlobal(self.into(),type_ref.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn set_declare_function(&self, name:&str, type_ref:&Type) ->&Value{
        self.get_named_function(name).unwrap_or_else(||self.add_function(name,type_ref))
    }

    pub fn get_named_function(&self,name:&str)->Option<&Value>{
        unsafe{
            ptr_to_optional_ref(LLVMGetNamedFunction(self.into(), compiler_c_str!(name)))
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

    pub fn build_alloca(&self,ty:&Type,name:&str) -> &Value{
        unsafe{
            LLVMBuildAlloca(self.into(),ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fneg(&self,v:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildFNeg(self.into(),v.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_neg(&self,v:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildNeg(self.into(),v.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn position_builder_at_end(&self,bb:&BasicBlock){
        unsafe{LLVMPositionBuilderAtEnd(self.into(),bb.into())}
    }

    pub fn build_fp_to_si(&self,value:&Value,dest_ty:&Type, name:&str)-> &Value{
        unsafe{
            LLVMBuildFPToSI(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fp_to_ui(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildFPToUI(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_si_to_fp(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildSIToFP(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_ui_to_fp(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildUIToFP(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fp_ext(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildFPExt(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fp_trunc(&self,value:&Value, dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildFPTrunc(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
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

    pub fn build_cast(&self,op_code:Opcode,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildCast(self.into(),op_code,value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_zext_or_bit_cast(&self,val:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildZExtOrBitCast(self.into(),val.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_bit_cast(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{LLVMBuildBitCast(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()}
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
    pub fn build_icmp(&self,int_predicate:IntPredicate,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{LLVMBuildICmp(self.into(),int_predicate,lhs.into(),rhs.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_fcmp(&self,real_predicate:RealPredicate,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildFCmp(self.into(),real_predicate,lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_trunc(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildTrunc(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_cond_br(&self,if_value:&Value,then_block:&BasicBlock,else_block:&BasicBlock)->&Value{
        unsafe{LLVMBuildCondBr(self.into(),if_value.into(),then_block.into(),else_block.into()).into()}
    }

    pub fn build_br(&self,dest:&BasicBlock)->&Value{
        unsafe{
            LLVMBuildBr(self.into(),dest.into()).into()
        }
    }

    pub fn build_add(&self,lhs:&Value,rhs:&Value,name:&str)-> &Value{
        unsafe{LLVMBuildAdd(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_fadd(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildFAdd(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fmul(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe {
            LLVMBuildFMul(self.into(), lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_mul(&self,lhs:&Value,rhs:&Value,name:&str)-> &Value{
        unsafe{LLVMBuildMul(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_sub(&self,lhs:&Value,rhs:&Value,name:&str)-> &Value{
        unsafe{
            LLVMBuildSub(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fsub(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildFSub(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_udiv(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildUDiv(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_sdiv(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildSDiv(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_fdiv(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildFDiv(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_load(&self,pointer_value:&Value,name:&str)->&Value{
        unsafe{LLVMBuildLoad(self.into(),pointer_value.into(),compiler_c_str!(name)).into()}
    }

    pub fn build_store(&self,value:&Value,ptr:&Value)->&Value{
        unsafe{LLVMBuildStore(self.into(),value.into(),ptr.into()).into()}
    }

    pub fn build_sext(&self,value:&Value,dest_ty:&Type, name:&str)->&Value{
        unsafe{
            LLVMBuildSExt(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_zext(&self,value:&Value,dest_ty:&Type,name:&str)->&Value{
        unsafe{
            LLVMBuildZExt(self.into(),value.into(),dest_ty.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn get_last_instruction(&self,bb:&BasicBlock)-> Option<&Value>{
        unsafe{ptr_to_optional_ref(LLVMGetLastInstruction(bb.into()))}
    }



    pub fn build_srem(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildSRem(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_urem(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildURem(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_and(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildAnd(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_or(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildOr(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_xor(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildXor(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_shl(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildShl(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_ashr(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildAShr(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_lshr(&self,lhs:&Value,rhs:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildLShr(self.into(),lhs.into(),rhs.into(),compiler_c_str!(name)).into()
        }
    }

    pub fn build_not(&self,v:&Value,name:&str)->&Value{
        unsafe{
            LLVMBuildNot(self.into(),v.into(),compiler_c_str!(name)).into()
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



pub struct ConstRealGetDoubleResult{
    pub result: ::libc::c_double,
    pub loses_info:bool,
}
pub enum Value{}
impl_type_traits!(Value,LLVMValueRef);
impl  Value{

    pub fn const_array<'a>(element_type:&Type,const_values:&'a [&Value])->&'a Value{
        unsafe{
            LLVMConstArray(element_type.into(),const_values.as_ptr() as *mut _,const_values.len() as ::libc::c_uint).into()
        }
    }

    pub fn is_basic_block(&self)->bool{
        unsafe{
            LLVMValueIsBasicBlock(self.into()) == 1
        }
    }


    pub fn as_basic_block(&self)->&BasicBlock{
        unsafe{
            LLVMValueAsBasicBlock(self.into()).into()
        }
    }

    pub fn get_instruction_opcode(&self)->Opcode{
        unsafe{
            LLVMGetInstructionOpcode(self.into())
        }
    }

    pub fn get_last_basic_block(&self)->Option<&BasicBlock>{
        unsafe{
            ptr_to_optional_ref(LLVMGetLastBasicBlock(self.into()))
        }
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

    pub fn const_int_get_sign_extended_value(&self)-> ::libc::c_longlong{
        unsafe{
            LLVMConstIntGetSExtValue(self.into())
        }
    }

    pub fn count_params(&self)-> ::libc::c_uint{
        unsafe{
            LLVMCountParams(self.into())
        }
    }

    pub fn get_param(&self,index: ::libc::c_uint)->Option<&Value>{
        unsafe{
            if index < self.count_params(){
                Some(LLVMGetParam(self.into(),index).into())
            } else{
                None
            }
        }
    }

    pub fn const_real_get_double(&self)-> ConstRealGetDoubleResult{
        unsafe{
            let mut loses_info:LLVMBool  = 0;
            let ret = LLVMConstRealGetDouble(self.into(),&mut loses_info as *mut _);
            ConstRealGetDoubleResult{result:ret,loses_info:loses_info != 0}
        }
    }

    pub fn set_global_const(&self,is_constant:bool){
        unsafe{
            LLVMSetGlobalConstant(self.into(),is_constant as LLVMBool)
        }
    }

    pub fn set_alignment(&self,align:u32){
        unsafe{
            LLVMSetAlignment(self.into(),align)
        }
    }

    pub fn is_global_const(&self)->bool{
        unsafe{
            LLVMIsGlobalConstant(self.into()) != 0
        }
    }


    pub fn get_initializer(&self)->Option<&Value>{
        unsafe{
            ptr_to_optional_ref( LLVMGetInitializer(self.into()))
        }
    }

    pub fn set_initializer(&self,constant_value:&Value){
        unsafe{
            LLVMSetInitializer(self.into(),constant_value.into())
        }
    }

    pub fn const_int_to_ptr(&self,to_type:&Type)->& Value{
        unsafe{
            LLVMConstIntToPtr(self.into(),to_type.into()).into()
        }
    }

    pub fn const_pointer_cast(&self,to_type:&Type)->& Value{
        unsafe{
            LLVMConstPointerCast(self.into(),to_type.into()).into()
        }
    }

    pub fn const_null_ptr(type_ref:&Type) ->&Value {
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
        unsafe{ ptr_to_optional_ref(LLVMGetFirstParam(self.into()))}
    }
    pub fn get_next_param(&self)->Option<&Value>{
        unsafe{ ptr_to_optional_ref(LLVMGetNextParam(self.into()))}
    }
    pub fn append_basic_block<'c>(&self,context:&'c Context,name:&str)->&'c BasicBlock{
        unsafe{LLVMAppendBasicBlockInContext(context.into(),self.into(),compiler_c_str!(name)).into()}
    }

    pub fn set_linkage(&self,linkage:Linkage){
        unsafe{

            LLVMSetLinkage(self.into(),linkage)
        }
    }

    pub fn dump(&self){
        unsafe{
            LLVMDumpValue(self.into())
        }
    }
}

pub enum Type{}
impl_type_traits!(Type,LLVMTypeRef);
impl Type{


    pub fn get_return_type(&self)->&Type{
        unsafe{
            LLVMGetReturnType(self.into()).into()
        }
    }

    pub fn int1(context:&Context)->&Type{
        unsafe{LLVMInt1TypeInContext(context.into()).into()}
    }
    pub fn int8(context:&Context)->&Type{
        unsafe {LLVMInt8TypeInContext(context.into()).into()}
    }

    pub fn int16(context:&Context)->&Type{
        unsafe{
            LLVMInt16TypeInContext(context.into()).into()
        }
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

    pub fn function<'a>(return_type:&'a Type,param_types:&[&'a Type],is_var_arg:bool)->&'a Type{
        unsafe{LLVMFunctionType(return_type.into(),param_types.as_ptr() as *mut _,param_types.len() as ::libc::c_uint,is_var_arg as LLVMBool).into()}
    }

    pub fn type_of(value:&Value)->&Type{
        unsafe{LLVMTypeOf(value.into()).into()}
    }

    pub fn from_wasm_value_type(context:&Context,value_type:ValueType)->&Type{
        match value_type {
            ValueType::F32 => Type::float32(context),
            ValueType::F64 => Type::float64(context),
            ValueType::I32 => Type::int32(context),
            ValueType::I64 => Type::int64(context),
        }
    }

}

pub enum BasicBlock{}
impl_type_traits!(BasicBlock,LLVMBasicBlockRef);

impl BasicBlock{
    pub fn insert_basic_block(&self,context:& Context,name:&str)->&BasicBlock{
        unsafe{LLVMInsertBasicBlockInContext(context.into(), self.into(),compiler_c_str!(name)).into()}
    }

    pub fn get_previous_basic_block(&self)->Option<&BasicBlock>{
        unsafe{
            ptr_to_optional_ref(LLVMGetPreviousBasicBlock(self.into()))
        }
    }

    pub fn get_next_basic_block(&self)->Option<&BasicBlock>{
        unsafe{
            ptr_to_optional_ref(LLVMGetNextBasicBlock(self.into()))
        }
    }
    pub fn move_after(&self,move_pos:&BasicBlock){
        unsafe{
            LLVMMoveBasicBlockAfter(self.into(),move_pos.into())
        }
    }

    pub fn move_before(&self,move_pos:&BasicBlock){
        unsafe{
            LLVMMoveBasicBlockBefore(self.into(),move_pos.into())
        }
    }
}

pub fn build_call_and_set_raise_const<'m>(module:&'m Module,builder:&'m Builder,sig: ::libc::c_int)->&'m Value{
    build_call_and_set_raise(module,builder,Value::const_int(Type::int32(module.context()),sig as ::libc::c_ulonglong,false))
}

pub fn build_call_and_set_abort<'m>(module:&'m Module,builder:&'m Builder){
    let context = module.context();
    let void_type = Type::void(&context);
    let abort_type = Type::function(void_type,&[],false);
    let abort = module.set_declare_function("abort",abort_type);
    builder.build_call(abort,&[],"");
}

pub fn build_call_and_set_raise<'m>(module:&'m Module,builder:&'m Builder,sig: &Value)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(&context);
    let raise_type = Type::function(int32_type,&[int32_type],false);
    let raise = module.set_declare_function("raise",raise_type);
    builder.build_call(raise,&[sig],"")
}

pub fn build_call_and_set_mmap<'m>(module:&'m  Module,builder:&'m Builder,addr:&Value,length:&Value,plot:&Value,flags:&Value,fd:&Value,offset:&Value,name:&str)->&'m Value{
    let context = module.context();
    let void_ptr_type = Type::ptr(Type::void(context),0);
    let int32_type = Type::int32(context);
    let param_types = [void_ptr_type,Type::int_ptr(context),int32_type,int32_type,int32_type,int32_type];
    let mmap_type = Type::function(void_ptr_type,&param_types,false);
    let mmap = module.set_declare_function("mmap", mmap_type);
    let args = [addr,length,plot,flags,fd,offset];
    builder.build_call(mmap,&args,name)
}

pub fn build_call_and_set_munmap<'m>(module:&'m Module,builder:&'m Builder,addr:&Value,length:&Value,name:&str)->&'m Value{
    let context =module.context();
    let void_ptr_type = Type::ptr(Type::void(context),0);
    let param_types = [void_ptr_type,Type::int_ptr(context)];
    let munmap_type = Type::function(Type::int32(context),&param_types,false);
    let munmap = module.set_declare_function("munmap", munmap_type);
    let args =[addr,length];
    builder.build_call(munmap,&args,name)
}

pub fn build_call_and_set_fminf<'m>(module:&'m Module,builder:&'m Builder,x:&Value,y:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let fminf_type = Type::function(float32_type,&[float32_type,float32_type],false);
    let fminf = module.set_declare_function("fminf",fminf_type);
    builder.build_call(fminf,&[x,y],name)
}

pub fn build_call_and_set_fmin<'m>(module:&'m Module, builder:&'m Builder, x:&Value, y:&Value, name:&str) ->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let fmin_type = Type::function(float64_type,&[float64_type,float64_type],false);
    let fmin = module.set_declare_function("fmin",fmin_type);
    builder.build_call(fmin,&[x,y],name)
}

pub fn build_call_and_set_fmaxf<'m>(module:&'m Module,builder:&'m Builder,x:&Value,y:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let fmaxf_type = Type::function(float32_type,&[float32_type,float32_type],false);
    let fminf = module.set_declare_function("fmaxf",fmaxf_type);
    builder.build_call(fminf,&[x,y],name)
}

pub fn build_call_and_set_fmax<'m>(module:&'m Module, builder:&'m Builder, x:&Value, y:&Value, name:&str) ->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let fmax_type = Type::function(float64_type,&[float64_type,float64_type],false);
    let fmax = module.set_declare_function("fmax",fmax_type);
    builder.build_call(fmax,&[x,y],name)
}


pub fn build_call_and_set_copysignf<'m>(module:&'m Module,builder:&'m Builder,x:&Value,y:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let copysignf_type = Type::function(float32_type,&[float32_type,float32_type],false);
    let copysignf = module.set_declare_function("copysignf",copysignf_type);
    builder.build_call(copysignf,&[x,y],name)
}

pub fn build_call_and_set_copysign<'m>(module:&'m Module, builder:&'m Builder, x:&Value, y:&Value, name:&str) ->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let copysign_type = Type::function(float64_type,&[float64_type,float64_type],false);
    let copysign = module.set_declare_function("copysign",copysign_type);
    builder.build_call(copysign,&[x,y],name)
}

pub fn build_call_and_set_ctlz_i32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let int1_type = Type::int1(context);
    let ctlz_i32_type = Type::function(int32_type,&[int32_type,int1_type],false);
    let ctlz_i32 = module.set_declare_function("llvm.ctlz.i32",ctlz_i32_type);
    builder.build_call(ctlz_i32,&[x,Value::const_int(int1_type,0,false)],name)
}

pub fn build_call_and_set_cttz_i32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let int1_type = Type::int1(context);
    let cttz_i32_type = Type::function(int32_type,&[int32_type,int1_type],false);
    let cttz_i32 = module.set_declare_function("llvm.cttz.i32",cttz_i32_type);
    builder.build_call(cttz_i32,&[x,Value::const_int(int1_type,0,false)],name)
}

pub fn build_call_and_set_ctlz_i64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int64_type = Type::int64(context);
    let int1_type = Type::int1(context);
    let ctlz_i64_type = Type::function(int64_type,&[int64_type,int1_type],false);
    let ctlz_i64 = module.set_declare_function("llvm.ctlz.i64",ctlz_i64_type);
    builder.build_call(ctlz_i64,&[x,Value::const_int(int1_type,0,false)],name)
}

pub fn build_call_and_set_cttz_i64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int64_type = Type::int64(context);
    let int1_type = Type::int1(context);
    let cttz_i64_type = Type::function(int64_type,&[int64_type,int1_type],false);
    let cttz_i64 = module.set_declare_function("llvm.cttz.i64",cttz_i64_type);
    builder.build_call(cttz_i64,&[x,Value::const_int(int1_type,0,false)],name)
}

pub fn build_call_and_set_ctpop_i32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let ctpop_i32_type = Type::function(int32_type,&[int32_type],false);
    let ctpop_i32 = module.set_declare_function("llvm.ctpop.i32",ctpop_i32_type);
    builder.build_call(ctpop_i32,&[x],name)
}

pub fn build_call_and_set_ctpop_i64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int64_type = Type::int64(context);
    let ctpop_i64_type = Type::function(int64_type,&[int64_type],false);
    let ctpop_i64 = module.set_declare_function("llvm.ctpop.i64",ctpop_i64_type);
    builder.build_call(ctpop_i64,&[x],name)
}

pub fn build_call_and_set_fabs_f32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let fabs_f32_type = Type::function(float32_type,&[float32_type],false);
    let fabs_f32 = module.set_declare_function("llvm.fabs.f32",fabs_f32_type);
    builder.build_call(fabs_f32,&[x],name)
}

pub fn build_call_and_set_fabs_f64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let fabs_f64_type = Type::function(float64_type,&[float64_type],false);
    let fabs_f64 = module.set_declare_function("llvm.fabs.f64",fabs_f64_type);
    builder.build_call(fabs_f64,&[x],name)
}

pub fn build_call_and_set_sqrt_f32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let sqrt_f32_type = Type::function(float32_type,&[float32_type],false);
    let sqrt_f32 = module.set_declare_function("llvm.sqrt.f32",sqrt_f32_type);
    builder.build_call(sqrt_f32,&[x],name)
}


pub fn build_call_and_set_sqrt_f64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let sqrt_f64_type = Type::function(float64_type,&[float64_type],false);
    let sqrt_f64 = module.set_declare_function("llvm.sqrt.f64",sqrt_f64_type);
    builder.build_call(sqrt_f64,&[x],name)
}

pub fn build_call_and_set_ceil_f32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let ceil_f32_type = Type::function(float32_type,&[float32_type],false);
    let ceil_f32 = module.set_declare_function("llvm.ceil.f32",ceil_f32_type);
    builder.build_call(ceil_f32,&[x],name)
}


pub fn build_call_and_set_ceil_f64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let ceil_f64_type = Type::function(float64_type,&[float64_type],false);
    let ceil_f64 = module.set_declare_function("llvm.ceil.f64",ceil_f64_type);
    builder.build_call(ceil_f64,&[x],name)
}

pub fn build_call_and_set_floor_f32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let floor_f32_type = Type::function(float32_type,&[float32_type],false);
    let floor_f32 = module.set_declare_function("llvm.floor.f32",floor_f32_type);
    builder.build_call(floor_f32,&[x],name)
}


pub fn build_call_and_set_floor_f64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let floor_f64_type = Type::function(float64_type,&[float64_type],false);
    let floor_f64 = module.set_declare_function("llvm.floor.f64",floor_f64_type);
    builder.build_call(floor_f64,&[x],name)
}

pub fn build_call_and_set_trunc_f32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let trunc_f32_type = Type::function(float32_type,&[float32_type],false);
    let trunc_f32 = module.set_declare_function("llvm.trunc.f32",trunc_f32_type);
    builder.build_call(trunc_f32,&[x],name)
}


pub fn build_call_and_set_trunc_f64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let trunc_f64_type = Type::function(float64_type,&[float64_type],false);
    let trunc_f64 = module.set_declare_function("llvm.trunc.f64",trunc_f64_type);
    builder.build_call(trunc_f64,&[x],name)
}

pub fn build_call_and_set_nearbyint_f32<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float32_type = Type::float32(context);
    let nearbyint_f32_type = Type::function(float32_type,&[float32_type],false);
    let nearbyint_f32 = module.set_declare_function("llvm.nearbyint.f32",nearbyint_f32_type);
    builder.build_call(nearbyint_f32,&[x],name)
}


pub fn build_call_and_set_nearbyint_f64<'m>(module:&'m Module,builder:&'m Builder,x:&Value,name:&str)->&'m Value{
    let context = module.context();
    let float64_type = Type::float64(context);
    let nearbyint_f64_type = Type::function(float64_type,&[float64_type],false);
    let nearbyint_f64 = module.set_declare_function("llvm.nearbyint.f64",nearbyint_f64_type);
    builder.build_call(nearbyint_f64,&[x],name)
}

pub fn build_call_and_set_donothing<'m>(module:&'m Module,builder:&'m Builder,name:&str)->&'m Value{
    let context = module.context();
    let donothing_type = Type::function(Type::void(context),&[],false);
    let donothing = module.set_declare_function("llvm.donothing",donothing_type);
    builder.build_call(donothing,&[],name)
}

pub fn build_call_and_set_trap<'m>(module:&'m Module,builder:&'m Builder,name:&str)->&'m Value{
    let context = module.context();
    let donothing_type = Type::function(Type::void(context),&[],false);
    let donothing = module.set_declare_function("llvm.trap",donothing_type);
    builder.build_call(donothing,&[],name)
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


unsafe fn ptr_to_optional_ref<P,T >(ptr: *mut P) -> Option<&'static T>  where &'static T:From<*mut P>  {
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
    pub use llvm_sys::analysis::LLVMVerifierFailureAction as VerifierFailureAction;
    pub fn verify_module(module:&Module,verifier_failure_action:VerifierFailureAction)-> Result<(),Error>{

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

pub mod target_machine{
    use super::*;
    use llvm_sys::target_machine::*;
    pub use llvm_sys::target_machine::{LLVMCodeGenOptLevel as CodeGenOptLevel,LLVMRelocMode as RelocMode,LLVMCodeModel as CodeModel ,LLVMCodeGenFileType as CodeGenFileType};



    pub fn get_default_target_triple()->Result<String,Error>{
        unsafe{
           Ok(CString::from_raw(LLVMGetDefaultTargetTriple()).to_str()?.to_string())
        }
    }

    pub enum Target{}
    impl_type_traits!(Target,LLVMTargetRef);
    impl Target{
        pub fn get_target_from_triple(triple:&str)->Result<&'static Target,Error>{
            unsafe{
                let mut out_message:*mut ::libc::c_char = mem::uninitialized();
                let mut target_ref:LLVMTargetRef = mem::uninitialized();
                if LLVMGetTargetFromTriple(compiler_c_str!(triple),&mut target_ref as *mut _,&mut  out_message as *mut _) != 0{
                    Err(FailureGetLLVMTarget{triple:triple.to_string(),message:convert_message_to_string(out_message)?})?
                } else{
                    Ok(target_ref.into())
                }
            }
        }

    }



    pub enum TargetMachine{}
    impl_type_traits!(TargetMachine,LLVMTargetMachineRef);
    impl TargetMachine{
        pub fn create_target_machine(target:&Target,triple:&str,cpu:&str,features:&str,level:CodeGenOptLevel,reloc:RelocMode,code_model:CodeModel)->&'static TargetMachine{
            unsafe{
                LLVMCreateTargetMachine(target.into(),compiler_c_str!(triple),compiler_c_str!(cpu),compiler_c_str!(features),level,reloc,code_model).into()
            }
        }

        pub fn emit_to_file(&self,module:&Module,file_name:&str,codegen:CodeGenFileType)->Result<(),Error>{
            unsafe{
                let mut out_message:*mut ::libc::c_char = mem::uninitialized();
                if LLVMTargetMachineEmitToFile(self.into(),module.into(),compiler_c_str!(file_name) as *mut _ ,codegen,&mut out_message as *mut _) != 0{
                    Err(FailureEmitLLVMModule {message:convert_message_to_string(out_message)?})?
                } else{
                    Ok(())
                }
            }
        }
    }

}

pub mod target{
    use super::*;
    use llvm_sys::target::*;

    pub fn initialize_all_asm_printers(){
        unsafe{
            LLVM_InitializeAllAsmPrinters()
        }
    }

    pub fn initialize_all_asm_parsers(){
        unsafe{
            LLVM_InitializeAllAsmParsers()
        }
    }

    pub fn initialize_all_target_mcs(){
        unsafe{
            LLVM_InitializeAllTargetMCs()
        }
    }

    pub fn initialize_all_target_infos(){
        unsafe{

            LLVM_InitializeAllTargetInfos()
        }

    }
    pub fn initialize_all_targets(){
        unsafe{
            LLVM_InitializeAllTargets()
        }
    }


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


        pub fn get_global_value_ref_from_address<T>(&self,name:&str)->&T{
            unsafe{
                ::std::mem::transmute(self.get_global_value_address(name) as usize)
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

        pub fn to_int(&self,is_signed:bool)->::libc::c_ulonglong{
            unsafe{
                LLVMGenericValueToInt(self.into(),if is_signed {1} else {0})
            }
        }

        pub fn to_float(&self,ty_ref:&Type)-> ::libc::c_double{
            unsafe{
                LLVMGenericValueToFloat(ty_ref.into(),self.into())
            }
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