
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
            section.entries().iter().filter(|p|match p.external() {
                External::Memory(_) =>true,
                _=>false,
            }).count()
        });
        let build_context = BuildContext::new(MODULE_ID,context);
        for (index,segment) in wasm_module.memory_section().ok_or(NotExistMemorySection)?.entries().iter().enumerate(){
            let memory_limits = segment.limits();
            self.build_init_linear_memory_function(&build_context,import_memory_count + index,  memory_limits.initial() ,memory_limits.maximum())?;
        }
        Ok(build_context.move_module())
    }

    pub fn get_linear_memory_name(&self,index:usize)->String{
        [LINEAR_MEMORY_NAME_BASE,index.to_string().as_ref()].concat()
    }

    pub fn set_linear_memory<'a>(&self,build_context:&'a BuildContext,index:usize) ->&'a Value {
        let memory_pointer_type =  Type::ptr(Type::int8(  build_context.context()), 0);
        build_context.module().set_global(self.get_linear_memory_name(index).as_ref(), memory_pointer_type)
    }

    pub fn get_linear_memory_size_name(&self,index:usize)->String{
        [LINEAR_MEMORY_PAGE_SIZE_NAME_BASE,index.to_string().as_ref()].concat()
    }

    pub fn set_linear_memory_size<'a>(&self,build_context:&'a BuildContext,index:usize)->&'a Value{
        let wasm_int_type = Type::int_wasm_ptr::<T>(build_context.context());
        build_context.module().set_global(self.get_linear_memory_size_name(index).as_ref(), wasm_int_type)
    }

    pub fn build_get_real_address<'a>(&self,build_context:&'a BuildContext,address:&Value, name:&str, index:usize)->&'a Value{
        let linear_memory = self.set_linear_memory(build_context,index);
        let int_ptr = Type::int_ptr(build_context.context());
        let indices = [Value::const_int(int_ptr,0,false), address];
        build_context.builder().build_gep(linear_memory,&indices,name)
    }

    pub fn set_init_linear_memory_function<'a>(&self,build_context:&'a BuildContext,index:usize)->&'a Value{
        let int1_type = Type::int1(build_context.context());
        let params:[&Type;0] =[];
        let grow_linear_memory_type = Type::function(int1_type,&params,true);
        build_context.module().set_function(self.get_init_linear_memory_function_name(index).as_ref(),grow_linear_memory_type)
    }

    pub fn get_init_linear_memory_function_name(&self,index:usize)->String{
        let bit_width = bit_width::<T>();
        ["init_linear_memory",index.to_string().as_ref(),"_", bit_width.to_string().as_ref()].concat()
    }

    pub fn build_init_linear_memory_function(&self,build_context:&BuildContext,index:usize, minimum:u32, maximum:Option<u32>)->Result<(),Error>{
        let function = self.set_init_linear_memory_function(build_context,index);
        build_context.builder().build_function(build_context.context(),function,|builder,_|{
            let maximum = maximum.unwrap_or_else(|| DEFAULT_MAXIMUM ) ;
            check_range(maximum,1,DEFAULT_MAXIMUM,name_of!(maximum))?;
            check_range(minimum,1,maximum - 1,name_of!(minimum))?;
            let context = build_context.context();
            let wasm_int_type = Type::int_wasm_ptr::<T>(context);
            let int_type = Type::int_ptr(context);

            let linear_memory = self.set_linear_memory(build_context,index);

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
            let linear_memory_size = self.set_linear_memory_size(build_context,index);
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
    fn extend_linear_memory(&self,liner_memory_ptr:&'a Value,maximum:u32)->&'a Value{


        let (addr,extended_size) = self.partial_extend_linear_memory(liner_memory_ptr, to_extend(Self::LIMIT_PAGE_SIZE), 0,maximum as usize/ Self::LIMIT_PAGE_SIZE );
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
            let indices = [extended_size_value];
            let tail_addr = self.build_context.builder().build_pointer_cast( self.build_context.builder().build_gep(mapped_ptr,&indices,""),Type::ptr(Type::void(self.context),0),"tail_addr");
            let args = [tail_addr,extend_size_value,self.prot_value,self.flags_value,self.fd_value,self.offset_value];
            let addr = build_call_and_set_mmap(self.build_context.module(),self.build_context.builder(),tail_addr,extend_size_value,self.prot_value,self.flags_value,self.fd_value,self.offset_value,"mapped_ptr");
            let stay_bb = self.function.append_basic_block(self.context,["count_",count.to_string().as_ref()].concat().as_ref());
            self.build_context.builder().build_cond_br(self.build_context.builder().build_icmp(LLVMIntPredicate::LLVMIntEQ,addr, Value::const_int_to_ptr(  Value::const_int(self.int_type,::libc::MAP_FAILED as u64,true),Type::type_of(addr)),""),self.fail_bb,stay_bb);
            self.build_context.builder().position_builder_at_end(stay_bb);
            self.build_context.builder().position_builder_at_end(stay_bb);
            if extended_size > 0{

                let munmap_result = build_call_and_set_munmap(self.build_context.module(),self.build_context.builder(),mapped_ptr,extended_size_value,"munmap_result");
                let munmap_result_cond = self.build_context.builder().build_icmp(LLVMIntPredicate::LLVMIntEQ,munmap_result,Value::const_int(self.int_type,-1_isize as u64,true),"");
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
        error_should_be!(test_init_linear_memory_in(26,Some(25)),RuntimeError,SizeIsTooLarge{message})
    }

    #[test]
    pub fn init_maximum_65537_linear_memory_not_works()->Result<(),Error>{
        error_should_be!(test_init_linear_memory_in(17,Some(65537)),RuntimeError,SizeIsTooLarge {message})
    }

    fn test_init_linear_memory_in(minimum:u32,maximum:Option<u32>)->Result<(),Error>{
        let context = Context::new();
        let build_context = BuildContext::new("grow_linear_memory_works",&context);
        let compiler = Compiler::new();
        compiler.build_init_linear_memory_function(&build_context, 0,minimum,maximum)?;
        analysis::verify_module(build_context.module(),analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction)?;
        test_jit_init()?;
        test_module_in_engine(build_context.module(),|engine|{
            let mut mapped_liner_memory_size: u32 = 0;
            let mut mapped_liner_memory: *mut ::libc::c_void = ::std::ptr::null_mut();
            let liner_memory_size = build_context.module().get_named_global(compiler.get_linear_memory_size_name(0).as_ref()).ok_or_else(|| NoSuchLLVMGlobalValue { name: LINEAR_MEMORY_PAGE_SIZE_NAME_BASE.to_string() })?;
            let liner_memory = build_context.module().get_named_global(compiler.get_linear_memory_name(0).as_ref()).ok_or_else(|| NoSuchLLVMGlobalValue { name: LINEAR_MEMORY_NAME_BASE.to_string() })?;
            engine.add_global_mapping(liner_memory, &mut mapped_liner_memory);
            engine.add_global_mapping(liner_memory_size, &mut mapped_liner_memory_size);
            let args: [&GenericValue; 0] = [];
            let result = test_run_function_with_name(&engine, build_context.module(), compiler.get_init_linear_memory_function_name(0).as_ref(), &args)?;
            assert_eq!(1,result.int_width());
            assert_eq!( minimum,mapped_liner_memory_size);
            assert_ne!(::std::ptr::null_mut(),mapped_liner_memory);
            assert_ne!(-1_isize , unsafe{::std::mem::transmute::<*mut ::libc::c_void ,isize>(mapped_liner_memory)});
            unsafe{
                let  p:*mut i8 =mapped_liner_memory.add(((maximum.unwrap_or(DEFAULT_MAXIMUM)-1) *PAGE_SIZE) as usize) as *mut _;
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
        assert_ne!(ptr::null(), compiler.set_linear_memory(&build_context,0).as_ptr());
        assert!( build_context.module().get_named_global(compiler.get_linear_memory_name(0).as_ref()).is_some());
    }

}