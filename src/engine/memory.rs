
use failure::Error;
use super::*;
use error::RuntimeError::*;
use error::*;
use parity_wasm::elements::Module as WasmModule;
use parity_wasm::elements::ResizableLimits;
use parity_wasm::elements::ImportCountType;

const MODULE_ID:&str = "__wasm_memory_module";
const MEMORY_NAME_BASE:&str = "_memory";
const MEMORY_PAGE_SIZE_NAME_BASE:&str = "_memory_size";
const MEMORY_REAL_CHECK_BASE:&str = "_memory_real_check";


pub trait MemoryTypeContext {
    const MEMORY_NAME_PREFIX:&'static str;
    const UNIT_SIZE:u32;
    const DEFAULT_MAXIMUM_UNIT_SIZE:u32;
}

pub enum LinearMemoryTypeContext{}
impl MemoryTypeContext for LinearMemoryTypeContext {
    const MEMORY_NAME_PREFIX: &'static str = "__wasm_linear";
    const UNIT_SIZE: u32 = 65536;
    const DEFAULT_MAXIMUM_UNIT_SIZE: u32 = 65536;
}

pub type LinearMemoryCompiler<T> =MemoryCompiler<LinearMemoryTypeContext,T>;

pub struct MemoryCompiler<M: MemoryTypeContext,T:WasmIntType>(::std::marker::PhantomData<T>, ::std::marker::PhantomData<M>);



impl<M: MemoryTypeContext,T:WasmIntType> MemoryCompiler<M,T> {

    pub fn new()-> MemoryCompiler<M,T>{
        MemoryCompiler(::std::marker::PhantomData::<T>{},::std::marker::PhantomData::<M>{})
    }
    pub fn compile<'a>(& self,build_context:&'a BuildContext,wasm_module:&WasmModule) -> Result<(),Error>{

        let import_memory_count = wasm_module.import_count(ImportCountType::Memory) as u32;
        if let Some(memory_section) = wasm_module.memory_section(){
            self.build_memory_functions(build_context, import_memory_count, &memory_section.entries().iter().map(|m|m.limits()).collect::<Vec<_>>())?;
        }

        Ok(())
    }

    pub fn get_memory_name(&self, index:u32) ->String{
        [M::MEMORY_NAME_PREFIX,MEMORY_NAME_BASE,&index.to_string()].concat()
    }

    pub fn set_declare_memory<'a>(&self, build_context:&'a BuildContext, index:u32) ->&'a Value {
        let memory_pointer_type =  Type::ptr(Type::int8(  build_context.context()), 0);
        build_context.module().set_declare_global(&self.get_memory_name(index), memory_pointer_type)
    }

    pub fn get_memory_size_name(&self, index:u32) ->String{
        [M::MEMORY_NAME_PREFIX, MEMORY_PAGE_SIZE_NAME_BASE,&index.to_string()].concat()
    }

    pub fn get_memory_real_check_name(&self,index:u32) -> String{
        [M::MEMORY_NAME_PREFIX,MEMORY_REAL_CHECK_BASE,&index.to_string()].concat()
    }

    pub fn set_declare_memory_size<'a>(&self, build_context:&'a BuildContext, index:u32) ->&'a Value{
        let wasm_int_type = Type::int_wasm_ptr::<T>(build_context.context());
        build_context.module().set_declare_global(&self.get_memory_size_name(index), wasm_int_type)
    }

    pub fn build_get_real_address<'a>(&self,build_context:&'a BuildContext,index:u32,address:&Value, name:&str )->&'a Value{
        let memory = self.set_declare_memory(build_context, index);
        let memory = build_context.builder().build_load(memory,"");
        build_context.builder().build_gep(memory,&[address],name)
    }

    pub fn build_get_real_memory_size<'a>(&self,build_context:&'a BuildContext,index:u32)->&'a Value {
        build_context.builder().build_mul(self.build_get_memory_size(build_context,index),Value::const_int(Type::int32(build_context.context()),M::UNIT_SIZE as u64,false),"")
    }
    pub fn build_get_memory_size<'a>(&self,build_context:&'a BuildContext,index:u32)->&'a Value{
        let memory_size = self.set_declare_memory_size(build_context,index);
        build_context.builder().build_load(memory_size,"")
    }

    pub fn build_set_memory_size<'a>(&self,build_context:&'a BuildContext,index:u32, new_memory_size:&'a Value){
        let memory_size = self.set_declare_memory_size(build_context,index);
        build_context.builder().build_store(new_memory_size,memory_size);
    }

    pub fn set_init_function<'a>(&self, build_context:&'a BuildContext) ->&'a Value{
        let int1_type = Type::int1(build_context.context());
        let grow_memory_type = Type::function(int1_type,&[],true);
        build_context.module().set_declare_function(&self.get_init_function_name(), grow_memory_type)
    }

    pub fn get_init_function_name(&self) ->String{
        let bit_width = bit_width::<T>();
        [M::MEMORY_NAME_PREFIX,"_init",MEMORY_NAME_BASE, &bit_width.to_string()].concat()
    }

    pub fn get_grow_function_name(&self,index:u32)->String{
        let bit_width = bit_width::<T>();
        [M::MEMORY_NAME_PREFIX,"_grow",MEMORY_NAME_BASE,"_",&bit_width.to_string(),"_",&index.to_string()].concat()
    }

    pub fn build_grow<'m>(&self,build_context:&'m BuildContext,  size:&Value,index:u32)->Result<&'m Value,Error>{
        let grow_memory_function_name = self.get_grow_function_name(index as u32);
        let grow_memory_function =  build_context.module().get_named_function(&grow_memory_function_name).ok_or(NoSuchLLVMFunction {name:grow_memory_function_name})?;
        Ok(build_context.builder().build_call(grow_memory_function,&[size],""))
    }

    pub fn build_memory_functions(&self, build_context:&BuildContext, import_count:u32, limits:&[&ResizableLimits]) ->Result<(),Error> {
        self.build_init_functions(build_context, import_count, limits, ||Ok(()))?;
        self.build_grow_memory_functions(build_context,import_count ,limits)?;
        self.build_check_memory_size_functions(build_context,import_count,limits)
    }

    pub fn build_check_memory_size_functions(&self,build_context:&BuildContext,import_count:u32, limits:&[&ResizableLimits]) -> Result<(),Error>{
        for (index,limit ) in limits.iter().enumerate(){
            let index = index as u32 + import_count ;
            let check_memory_size_function =  build_context.module().set_declare_function(&self.get_memory_real_check_name(index),Type::function(Type::void(build_context.context()),&[Type::int32(build_context.context())],false));
            build_context.builder().build_function(build_context.context(),check_memory_size_function,|_,_|{
                let target = check_memory_size_function.get_first_param().ok_or(NotExistValue)?;
                let memory_size = self.build_get_real_memory_size(build_context, index);
                let cmp_ret = build_context.builder().build_icmp(IntPredicate::LLVMIntULT,target,memory_size,"");
                let else_bb = check_memory_size_function.append_basic_block(build_context.context(),"");
                let then_bb = check_memory_size_function.append_basic_block(build_context.context(),"");
                build_context.builder().build_cond_br(cmp_ret,then_bb,else_bb);
                build_context.builder().position_builder_at_end(else_bb);
                build_call_and_set_raise_const(build_context.module(),build_context.builder(),::libc::SIGSEGV);
                build_call_and_set_raise_const(build_context.module(),build_context.builder(),::libc::SIGSEGV); // for test on JIT. It need to send it twice why.
                build_context.builder().build_br(then_bb);
                build_context.builder().position_builder_at_end(then_bb);
                build_context.builder().build_ret_void();
                Ok(())
            } )?;


        }
        Ok(())
    }

    pub fn build_grow_memory_functions(&self, build_context:&BuildContext,import_count:u32, limits:&[&ResizableLimits])->Result<(),Error>{
        let int32_type = Type::int32(build_context.context());
        for (index,limit) in limits.iter().enumerate(){
            let index = index as u32 + import_count ;
            let grow_memory_function = build_context.module().set_declare_function(&self.get_grow_function_name(index),Type::function(int32_type,&[int32_type],false));
            build_context.builder().build_function(build_context.context(),grow_memory_function,|_,_|{
                let max_memory_size_value = Value::const_int(Type::int32(build_context.context()),limit.maximum().unwrap_or(M::DEFAULT_MAXIMUM_UNIT_SIZE) as u64,false);
                let grow_memory_size = grow_memory_function.get_first_param().ok_or(NotExistValue)?;
                let current_memory_size = self.build_get_memory_size(build_context,index as u32);
                let target_memory_size = build_context.builder().build_add(current_memory_size,grow_memory_size,"");
                let grow_bb = grow_memory_function.append_basic_block(build_context.context(),"");
                let fail_bb = grow_memory_function.append_basic_block(build_context.context(),"");
                build_context.builder().build_cond_br(build_context.builder().build_icmp(IntPredicate::LLVMIntULE,target_memory_size,max_memory_size_value,""), grow_bb,fail_bb);

                build_context.builder().position_builder_at_end(grow_bb);
                self.build_set_memory_size(build_context,index as u32,target_memory_size);
                build_context.builder().build_ret(current_memory_size);

                build_context.builder().position_builder_at_end(fail_bb);
                build_context.builder().build_ret(Value::const_int(Type::int32(build_context.context()),-1_i64 as u64,false));
                Ok(())
            })?;
        }

        Ok(())
    }

    pub fn build_init_functions<F:FnOnce()->Result<(),Error>>(&self, build_context:&BuildContext, import_count:u32, limits:&[&ResizableLimits], and_then:F) ->Result<(),Error>{

        let function = self.set_init_function(build_context);
        build_context.builder().build_function(build_context.context(),function,|builder,_|{
            let fail_bb = function.append_basic_block(build_context.context(),"fail_bb");
            for (index,limit) in limits.iter().enumerate(){
                let index = index as u32 + import_count;
                let minimum = limit.initial();
                let maximum = limit.maximum();
                let memory = self.set_declare_memory(build_context, index);
                let memory_size = self.set_declare_memory_size(build_context, index);
                memory_size.set_initializer(Value::const_int(Type::int_wasm_ptr::<T>(build_context.context()),0,false));
                memory.set_initializer(Value::const_null(Type::ptr(Type::int8(build_context.context()),0)));
                let maximum = maximum.unwrap_or(M::DEFAULT_MAXIMUM_UNIT_SIZE ) ;
                check_range(maximum,1,M::DEFAULT_MAXIMUM_UNIT_SIZE,name_of!(maximum))?;
                check_range(minimum,1,maximum,name_of!(minimum))?;
                let context = build_context.context();
                let wasm_int_type = Type::int_wasm_ptr::<T>(context);
                let int_type = Type::int_ptr(context);
                let memory_cache = builder.build_load(memory,"memory_cache");
                let i32_type = Type::int32(context);

                let mmap_closure = MMapClosure {
                    build_context,
                    int_type,
                    prot_value:Value::const_int(i32_type,(::libc::PROT_READ | ::libc::PROT_WRITE) as ::libc::c_ulonglong,true),
                    flags_value:Value::const_int(i32_type, (::libc::MAP_PRIVATE | ::libc::MAP_ANONYMOUS) as ::libc::c_ulonglong,true ),
                    fd_value:Value::const_int(i32_type,-1_isize as ::libc::c_ulonglong,true),
                    offset_value:Value::const_int(i32_type,0,true),
                    fail_bb,
                    function,
                    context,
                    phantom_target : ::std::marker::PhantomData::<T>,
                    phantom_memory_context: ::std::marker::PhantomData::<M>,
                };

                let mapped_ptr = mmap_closure.extend_memory(memory_cache,maximum);
                builder.build_store(builder.build_pointer_cast(mapped_ptr,Type::type_of(memory_cache),""),memory);

                builder.build_store(Value::const_int(wasm_int_type,minimum as ::libc::c_ulonglong,false),memory_size);
            }
            and_then()?;
            let int1_type = Type::int1(build_context.context());
            builder.build_ret(Value::const_int(int1_type,1 ,false));

            builder.position_builder_at_end(fail_bb);
            builder.build_ret(Value::const_int(int1_type,0 ,false));
            Ok(())
        })
    }

}

struct MMapClosure<'a,T:WasmIntType,M:MemoryTypeContext>{
    build_context:&'a BuildContext<'a>,
    int_type:&'a Type,
    prot_value : &'a Value,
    flags_value:&'a Value,
    fd_value:&'a Value,
    offset_value :&'a Value,
    fail_bb:&'a BasicBlock,
    function:&'a Value,
    context:&'a Context,
    phantom_target: ::std::marker::PhantomData<T>,
    phantom_memory_context: ::std::marker::PhantomData<M>,
}

#[derive(Clone,Copy)]
struct ExtendSize(usize);
fn size_to_extend<M:MemoryTypeContext>(size:usize) ->ExtendSize{
    ExtendSize(size* M::UNIT_SIZE as usize)
}
impl<'a,T:WasmIntType,M:MemoryTypeContext> MMapClosure<'a,T,M>{
    const LIMIT_PAGE_SIZE:usize = ::std::usize::MAX / M::UNIT_SIZE as usize;
    fn extend_memory(&self,memory_ptr:&'a Value,maximum:u32)->&'a Value{


        let (addr,extended_size) = self.partial_extend_memory(memory_ptr, size_to_extend::<M>(Self::LIMIT_PAGE_SIZE), 0, maximum as usize/ Self::LIMIT_PAGE_SIZE );
        let reminder_extend_size = size_to_extend::<M>(maximum  as usize % Self::LIMIT_PAGE_SIZE);
        if reminder_extend_size.0 > 0{
            let (addr,_) = self.partial_extend_memory(addr,reminder_extend_size,extended_size,1);
            addr
        }  else{
            addr
        }

    }
    fn partial_extend_memory(&self,mapped_ptr:&'a Value,extend_size:ExtendSize, extended_size:usize,count:usize)->(&'a Value,usize){
        if count > 0{
            let extend_size_value = Value::const_int(self.int_type,extend_size.0 as ::libc::c_ulonglong,false);
            let extended_size_value = Value::const_int(self.int_type, extended_size as ::libc::c_ulonglong,false);
            let tail_addr = self.build_context.builder().build_pointer_cast( self.build_context.builder().build_gep(mapped_ptr, &[extended_size_value],""),Type::ptr(Type::void(self.context),0),"tail_addr");
            let addr = build_call_and_set_mmap(self.build_context.module(),self.build_context.builder(),tail_addr,extend_size_value,self.prot_value,self.flags_value,self.fd_value,self.offset_value,"mapped_ptr");
            let stay_bb = self.function.append_basic_block(self.context,&["count_",&count.to_string()].concat());
            self.build_context.builder().build_cond_br(self.build_context.builder().build_icmp(IntPredicate::LLVMIntEQ,addr, Value::const_int_to_ptr(  Value::const_int(self.int_type,::libc::MAP_FAILED as u64,true),Type::type_of(addr)),""),self.fail_bb,stay_bb);
            self.build_context.builder().position_builder_at_end(stay_bb);
            self.build_context.builder().position_builder_at_end(stay_bb);
            if extended_size > 0{

                let munmap_result = build_call_and_set_munmap(self.build_context.module(),self.build_context.builder(),mapped_ptr,extended_size_value,"munmap_result");
                let munmap_result_cond = self.build_context.builder().build_icmp(IntPredicate::LLVMIntEQ,munmap_result,Value::const_int(self.int_type,-1_isize as u64,true),"");
                self.build_context.builder().build_cond_br(munmap_result_cond,self.fail_bb,stay_bb);
            }
            self.partial_extend_memory(addr,extend_size,extended_size + extend_size.0,count -1)
        } else{
            (mapped_ptr,extended_size)
        }

    }
}

#[cfg(test)]
pub mod test_utils{
    use super::*;
    type Compiler = LinearMemoryCompiler<u32>;

    pub fn init_test_memory(build_context:&BuildContext)->Result<String,Error>{
        let compiler = Compiler::new();
        compiler.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(17, Some(25))])?;
        Ok(compiler.get_init_function_name())
    }
}

#[cfg(test)]
mod tests{

    use super::*;
    use std::ptr;

    type Compiler = LinearMemoryCompiler<u32>;

    #[test]
    pub fn init_memory_works() ->Result<(),Error>{
        test_init_memory_in(17,Some(25))
    }

    #[test]
    pub fn init_none_maximum_memory_works()->Result<(),Error>{
        test_init_memory_in(17,None)
    }


    #[test]
    pub fn init_maximum_65536_memory_works()->Result<(),Error>{

        test_init_memory_in(17,Some(65536))
    }

    #[test]
    pub fn init_minimum_greater_maximum_memory_works() ->Result<(),Error>{
        error_should_be!(test_init_memory_in(26,Some(25)),SizeIsTooLarge{message:_})
    }

    #[test]
    pub fn init_maximum_65537_memory_not_works()->Result<(),Error>{
        error_should_be!(test_init_memory_in(17,Some(65537)),SizeIsTooLarge {message:_})
    }

    fn test_init_memory_in(minimum:u32,maximum:Option<u32>)->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("grow_memory_works",&context);
        let compiler = Compiler::new();
        compiler.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(minimum, maximum)])?;
        test_module_in_engine(build_context.module(),|engine|{
            let result = run_test_function_with_name(&engine, build_context.module(), &compiler.get_init_function_name(), &[])?;
            assert_eq!(1,result.to_int(false));

            let mapped_memory_size= *engine.get_global_value_ref_from_address::<u32>(compiler.get_memory_size_name(0).as_ref());
            assert_eq!( minimum,mapped_memory_size);

            let mapped_memory= *engine.get_global_value_ref_from_address::<*mut ::libc::c_void>(&compiler.get_memory_name(0));
            assert_ne!(::std::ptr::null_mut(),mapped_memory);
            assert_ne!(-1_isize , unsafe{::std::mem::transmute::<*mut ::libc::c_void ,isize>(mapped_memory)});
            unsafe{
                let  p:*mut i8 =mapped_memory.add(((maximum.unwrap_or(LinearMemoryTypeContext::DEFAULT_MAXIMUM_UNIT_SIZE)-1) *LinearMemoryTypeContext::UNIT_SIZE) as usize) as *mut _;
                *p = 32;
                assert_eq!(*p,32);
            }

            Ok(())
        })?;
        Ok(())
    }


    #[test]
    pub fn set_memory_works(){
        let  context = Context::new();
        let build_context = BuildContext::new("set_memory_works",&context);
        let compiler = Compiler::new();
        assert_ne!(ptr::null(), compiler.set_declare_memory(&build_context, 0).as_ptr());
        assert!( build_context.module().get_named_global(&compiler.get_memory_name(0)).is_some());
    }

    #[test]
    pub fn build_get_real_address_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_get_real_address_works",&context);
        let compiler = Compiler::new();
        let test_function_name = "build_get_real_address_test";
        compiler.build_memory_functions(&build_context, 0, &[&ResizableLimits::new(17, Some(25))])?;
        build_test_function(&build_context,test_function_name,&[], |builder,_bb|{
            let addr = compiler.build_get_real_address(&build_context,0,Value::const_int(Type::int_wasm_ptr::<u32>(&context),32,false),"addr_value");
            builder.build_store(Value::const_int(Type::int8(build_context.context()),55,false),addr);
            builder.build_ret_void();
            Ok(())
        })?;

        analysis::verify_module(build_context.module(),analysis::VerifierFailureAction::LLVMPrintMessageAction)?;

        test_module_in_engine(build_context.module(),|engine|{
            let result = run_test_function_with_name(&engine, build_context.module(), &compiler.get_init_function_name(), &[])?;
            assert_eq!(1,result.to_int(false));

            run_test_function_with_name(&engine,build_context.module(),test_function_name,&[])?;
            let mapped_memory= *engine.get_global_value_ref_from_address::<*mut i8>(&compiler.get_memory_name(0));
            assert_eq!(55,unsafe{*mapped_memory.add(32)});
            Ok(())
        })?;
        Ok(())
    }

}