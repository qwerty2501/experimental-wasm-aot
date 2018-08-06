
use failure::Error;
use super::*;
use std::ptr;
use error::RuntimeError::*;
use error::*;
use parity_wasm::elements::Module as WasmModule;
use parity_wasm::elements::External;

const MODULE_ID:&str = "__wasm_linear_memory_module";
const LINEAR_MEMORY_NAME_BASE:&str = "__wasm_linear_memory";
const LINEAR_MEMORY_PAGE_SIZE_NAME_BASE:&str = "__wasm_linear_memory_size";

pub struct LinearMemoryCompiler<T:WasmIntType>(::std::marker::PhantomData<T>);

impl<T:WasmIntType> LinearMemoryCompiler<T> {

    pub fn new()-> LinearMemoryCompiler<T>{
        LinearMemoryCompiler(::std::marker::PhantomData::<T>{})
    }
    pub fn compile<'a>(& self,context:&'a Context,wasm_module:&WasmModule) -> Result<ModuleGuard<'a>,Error>{
        let import_memory_count = wasm_module.import_section().map_or(0,|section|{
            section.entries().iter().filter(|p| is_match_case!( p.external(),External::Memory(_))).count() as u32
        });
        let build_context = BuildContext::new(MODULE_ID,context);
        for (index,segment) in wasm_module.memory_section().ok_or(NotExistMemorySection)?.entries().iter().enumerate(){
            let memory_limits = segment.limits();
            self.build_init_linear_memory_function(&build_context,import_memory_count + index as u32,  memory_limits.initial() ,memory_limits.maximum())?;
        }
        Ok(build_context.move_module())
    }

    pub fn get_linear_memory_name(&self,index:u32)->String{
        [LINEAR_MEMORY_NAME_BASE,&index.to_string()].concat()
    }

    pub fn set_declare_linear_memory<'a>(&self, build_context:&'a BuildContext, index:u32) ->&'a Value {
        let memory_pointer_type =  Type::ptr(Type::int8(  build_context.context()), 0);
        build_context.module().set_declare_global(&self.get_linear_memory_name(index), memory_pointer_type)
    }

    pub fn get_linear_memory_size_name(&self,index:u32)->String{
        [LINEAR_MEMORY_PAGE_SIZE_NAME_BASE,&index.to_string()].concat()
    }

    pub fn set_declare_linear_memory_size<'a>(&self, build_context:&'a BuildContext, index:u32) ->&'a Value{
        let wasm_int_type = Type::int_wasm_ptr::<T>(build_context.context());
        build_context.module().set_declare_global(&self.get_linear_memory_size_name(index), wasm_int_type)
    }

    pub fn build_get_real_address<'a>(&self,build_context:&'a BuildContext,index:u32,address:&Value, name:&str )->&'a Value{
        let linear_memory = self.set_declare_linear_memory(build_context, index);
        let linear_memory = build_context.builder().build_load(linear_memory,"");
        let zero = Value::const_int(Type::int_ptr(build_context.context()),0,false);
        build_context.builder().build_gep(linear_memory,&[address],name)
    }

    pub fn set_init_linear_memory_function<'a>(&self,build_context:&'a BuildContext,index:u32)->&'a Value{
        let int1_type = Type::int1(build_context.context());
        let grow_linear_memory_type = Type::function(int1_type,&[],true);
        build_context.module().set_declare_function(&self.get_init_linear_memory_function_name(index), grow_linear_memory_type)
    }

    pub fn get_init_linear_memory_function_name(&self,index:u32)->String{
        let bit_width = bit_width::<T>();
        ["init_linear_memory",&index.to_string(),"_", &bit_width.to_string()].concat()
    }

    pub fn build_init_linear_memory_function(&self,build_context:&BuildContext,index:u32, minimum:u32, maximum:Option<u32>)->Result<(),Error>{
        let linear_memory = self.set_declare_linear_memory(build_context, index);
        let linear_memory_size = self.set_declare_linear_memory_size(build_context, index);
        linear_memory_size.set_initializer(Value::const_int(Type::int_wasm_ptr::<T>(build_context.context()),0,false));
        linear_memory.set_initializer(Value::const_null(Type::ptr(Type::int8(build_context.context()),0)));
        let function = self.set_init_linear_memory_function(build_context,index);
        build_context.builder().build_function(build_context.context(),function,|builder,_|{
            let maximum = maximum.unwrap_or_else(|| DEFAULT_MAXIMUM ) ;
            check_range(maximum,1,DEFAULT_MAXIMUM,name_of!(maximum))?;
            check_range(minimum,1,maximum - 1,name_of!(minimum))?;
            let context = build_context.context();
            let wasm_int_type = Type::int_wasm_ptr::<T>(context);
            let int_type = Type::int_ptr(context);



            let linear_memory_cache = builder.build_load(linear_memory,"linear_memory_cache");
            let i32_type = Type::int32(context);

            let void_type = Type::void(context);
            let void_ptr_type = Type::ptr(void_type, 0);
            let fail_bb = function.append_basic_block(context,"fail_bb");
            let mmap_closure = MMapClosure {
                build_context,
                int_type,
                void_ptr_type,
                prot_value:Value::const_int(i32_type,(::libc::PROT_READ | ::libc::PROT_WRITE) as ::libc::c_ulonglong,true),
                flags_value:Value::const_int(i32_type, (::libc::MAP_PRIVATE | ::libc::MAP_ANONYMOUS) as ::libc::c_ulonglong,true ),
                fd_value:Value::const_int(i32_type,-1_isize as ::libc::c_ulonglong,true),
                offset_value:Value::const_int(i32_type,0,true),
                fail_bb,
                function,
                context,
                phantom : ::std::marker::PhantomData::<T>
            };

            let mapped_ptr = mmap_closure.extend_linear_memory(linear_memory_cache,maximum);
            builder.build_store(builder.build_pointer_cast(mapped_ptr,Type::type_of(linear_memory_cache),""),linear_memory);

            builder.build_store(Value::const_int(wasm_int_type,minimum as ::libc::c_ulonglong,false),linear_memory_size);
            let int1_type = Type::int1(context);
            builder.build_ret(Value::const_int(int1_type,1 ,false));

            builder.position_builder_at_end(fail_bb);
            builder.build_ret(Value::const_int(int1_type,0 ,false));
            Ok(())
        })
    }

}

struct MMapClosure<'a,T:WasmIntType>{
    build_context:&'a BuildContext<'a>,
    int_type:&'a Type,
    void_ptr_type:&'a Type,
    prot_value : &'a Value,
    flags_value:&'a Value,
    fd_value:&'a Value,
    offset_value :&'a Value,
    fail_bb:&'a BasicBlock,
    function:&'a Value,
    context:&'a Context,
    phantom: ::std::marker::PhantomData<T>
}

#[derive(Clone,Copy)]
struct ExtendSize(usize);
fn to_extend(size:usize)->ExtendSize{
    ExtendSize(size*PAGE_SIZE as usize)
}
impl<'a,T:WasmIntType> MMapClosure<'a,T>{
    const LIMIT_PAGE_SIZE:usize = ::std::usize::MAX / PAGE_SIZE as usize;
    fn extend_linear_memory(&self,linear_memory_ptr:&'a Value,maximum:u32)->&'a Value{


        let (addr,extended_size) = self.partial_extend_linear_memory(linear_memory_ptr, to_extend(Self::LIMIT_PAGE_SIZE), 0,maximum as usize/ Self::LIMIT_PAGE_SIZE );
        let reminder_extend_size = to_extend(maximum  as usize % Self::LIMIT_PAGE_SIZE);
        if reminder_extend_size.0 > 0{
            let (addr,_) = self.partial_extend_linear_memory(addr,reminder_extend_size,extended_size,1);
            addr
        }  else{
            addr
        }

    }
    fn partial_extend_linear_memory(&self,mapped_ptr:&'a Value,extend_size:ExtendSize, extended_size:usize,count:usize)->(&'a Value,usize){
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
            self.partial_extend_linear_memory(addr,extend_size,extended_size + extend_size.0,count -1)
        } else{
            (mapped_ptr,extended_size)
        }

    }
}



#[cfg(test)]
mod tests{

    use super::*;
    use super::super::llvm::execution_engine::*;
    use super::super::test_utils::*;
    type Compiler = LinearMemoryCompiler<u32>;

    #[test]
    pub fn init_linear_memory_works() ->Result<(),Error>{
        test_init_linear_memory_in(17,Some(25))
    }

    #[test]
    pub fn init_none_maximum_linear_memory_works()->Result<(),Error>{
        test_init_linear_memory_in(17,None)
    }


    #[test]
    pub fn init_maximum_65536_linear_memory_works()->Result<(),Error>{

        test_init_linear_memory_in(17,Some(65536))
    }

    #[test]
    pub fn init_minimum_greater_maximum_linear_memory_works() ->Result<(),Error>{
        error_should_be!(test_init_linear_memory_in(26,Some(25)),SizeIsTooLarge{message})
    }

    #[test]
    pub fn init_maximum_65537_linear_memory_not_works()->Result<(),Error>{
        error_should_be!(test_init_linear_memory_in(17,Some(65537)),SizeIsTooLarge {message})
    }

    fn test_init_linear_memory_in(minimum:u32,maximum:Option<u32>)->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("grow_linear_memory_works",&context);
        let compiler = Compiler::new();
        compiler.build_init_linear_memory_function(&build_context, 0,minimum,maximum)?;
        analysis::verify_module(build_context.module(),analysis::VerifierFailureAction::LLVMPrintMessageAction)?;
        init_test_jit()?;
        test_module_in_engine(build_context.module(),|engine|{
            let result = run_test_function_with_name(&engine, build_context.module(), &compiler.get_init_linear_memory_function_name(0), &[])?;
            assert_eq!(1,result.int_width());

            let mapped_linear_memory_size= *engine.get_global_value_ref_from_address::<u32>(compiler.get_linear_memory_size_name(0).as_ref());
            assert_eq!( minimum,mapped_linear_memory_size);

            let mapped_linear_memory= *engine.get_global_value_ref_from_address::<*mut ::libc::c_void>(&compiler.get_linear_memory_name(0));
            assert_ne!(::std::ptr::null_mut(),mapped_linear_memory);
            assert_ne!(-1_isize , unsafe{::std::mem::transmute::<*mut ::libc::c_void ,isize>(mapped_linear_memory)});
            unsafe{
                let  p:*mut i8 =mapped_linear_memory.add(((maximum.unwrap_or(DEFAULT_MAXIMUM)-1) *PAGE_SIZE) as usize) as *mut _;
                *p = 32;
                assert_eq!(*p,32);
            }

            Ok(())
        })?;
        Ok(())
    }


    #[test]
    pub fn set_linear_memory_works(){
        let  context = Context::new();
        let build_context = BuildContext::new("set_linear_memory_works",&context);
        let compiler = Compiler::new();
        assert_ne!(ptr::null(), compiler.set_declare_linear_memory(&build_context, 0).as_ptr());
        assert!( build_context.module().get_named_global(&compiler.get_linear_memory_name(0)).is_some());
    }

    #[test]
    pub fn build_get_real_address_works()->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("build_get_real_address_works",&context);
        let compiler = Compiler::new();
        let test_function_name = "build_get_real_address_test";
        compiler.build_init_linear_memory_function(&build_context,0,17,Some(25))?;
        build_test_function(&build_context,test_function_name,&[], |builder,bb|{
            let addr = compiler.build_get_real_address(&build_context,0,Value::const_int(Type::int_wasm_ptr::<u32>(&context),32,false),"addr_value");
            builder.build_store(Value::const_int(Type::int8(build_context.context()),55,false),addr);
            builder.build_ret_void();
            Ok(())
        })?;

        analysis::verify_module(build_context.module(),analysis::VerifierFailureAction::LLVMPrintMessageAction)?;

        init_test_jit()?;
        test_module_in_engine(build_context.module(),|engine|{
            let result = run_test_function_with_name(&engine, build_context.module(), &compiler.get_init_linear_memory_function_name(0), &[])?;
            assert_eq!(1,result.int_width());

            run_test_function_with_name(&engine,build_context.module(),test_function_name,&[])?;
            let mapped_linear_memory= *engine.get_global_value_ref_from_address::<*mut i8>(&compiler.get_linear_memory_name(0));
            assert_eq!(55,unsafe{*mapped_linear_memory.add(32)});
            Ok(())
        })?;
        Ok(())
    }

}