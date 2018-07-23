use failure::Error;
use num::*;
use super::llvm::*;
use super::constants::*;
use super::wasm::*;
use std::ptr;
use error::RuntimeError::*;
use error::*;
const MODULE_ID:&str = "__wasm_linear_memory_module";
const LINEAR_MEMORY_NAME:&str = "__wasm_linear_memory";
const LINEAR_MEMORY_PAGE_SIZE_NAME:&str = "__wasm_linear_memory_size";

pub struct LinearMemoryCompiler<T:Integer>(::std::marker::PhantomData<T>);

impl<T:Integer> LinearMemoryCompiler<T> {
    pub fn compile(context:&Context,minimum:usize,maximum:Option<usize>) -> Result<Guard<Module>,Error>{
        let module = Module::new(MODULE_ID,context);
        let builder = Builder::new(context);
        Self::build_init_linear_memory_function(&module,&builder,minimum,maximum)?;
        Ok(module)
    }

    pub fn set_linear_memory(module:&Module) ->& Value {
        let memory_pointer_type =  Type::ptr(Type::void( module.context()), 0);
        module.set_global(LINEAR_MEMORY_NAME,memory_pointer_type)
    }

    pub fn set_linear_memory_size(module:&Module)->&Value{
        let wasm_int_type = Type::int_wasm_ptr::<T>(module.context());
        module.set_global(LINEAR_MEMORY_PAGE_SIZE_NAME, wasm_int_type)
    }

    pub fn set_init_linear_memory_function(module:&Module)->&Value{
        let context = module.context();
        let void_type = Type::void(context);
        let parms:[&Type;0] =[];
        let grow_linear_memory_type = Type::function(void_type,&parms,true);
        module.set_function(Self::get_init_linear_memory_function_name().as_ref(),grow_linear_memory_type)
    }

    pub fn get_init_linear_memory_function_name()->String{
        let bit_width = bit_width::<T>();
        to_wasm_call_name(["__init_linear_memory", bit_width.to_string().as_ref()].concat().as_ref())
    }

    pub fn build_init_linear_memory_function<'a>(module:&'a Module,b:&'a Builder,minimum:usize, maximum:Option<usize>)->Result<(),Error>{
        let function = Self::set_init_linear_memory_function(module);
        b.build_function(module.context(),function,|builder,bb|{
            let maximum = maximum.unwrap_or_else(|| DEFAULT_MAXIMUM ) ;
            let _:() = check_range(maximum,1,DEFAULT_MAXIMUM,name_of!(maximum))?;
            let _ :() = check_range(minimum,1,maximum - 1,name_of!(minimum))?;
            let context = module.context();
            let wasm_int_type = Type::int_wasm_ptr::<T>(context);
            let i64_type = Type::int64(context);
            let int_type = Type::int_ptr(context);

            let linear_memory = Self::set_linear_memory(module);

            let linear_memory_cache = builder.build_load(linear_memory,"linear_memory_cache");
            let i32_type = Type::int32(context);

            let void_type = Type::void(context);
            let void_ptr_type = Type::ptr(void_type, 0);
            let param_types = [void_ptr_type,int_type,i32_type,i32_type,i32_type,i32_type];
            let mmap_type = Type::function(void_ptr_type,&param_types,true);
            let mmap_caller = MMapCaller{
                int_type,
                void_ptr_type,
                mmap_function:module.set_function("mmap",mmap_type),
                prot_value:Value::const_int(i32_type,(::libc::PROT_READ | ::libc::PROT_WRITE) as ::libc::c_ulonglong,true),
                flags_value:Value::const_int(i32_type, (::libc::MAP_PRIVATE | ::libc::MAP_ANONYMOUS) as ::libc::c_ulonglong,true ),
                fd_value:Value::const_int(i32_type,-1_isize as ::libc::c_ulonglong,true),
                offset_value:Value::const_int(i32_type,0,true),
                builder,
            };
            let mapped_ptr = mmap_caller.extend_linear_memory(linear_memory_cache,maximum);
            builder.build_store(mapped_ptr,linear_memory);
            let linear_memory_size = Self::set_linear_memory_size(module);
            builder.build_store(Value::const_int(wasm_int_type,minimum as ::libc::c_ulonglong,false),linear_memory_size);
            builder.build_ret_void();
            Ok(())
        })
    }

}

struct MMapCaller<'a>{
    int_type:&'a Type,
    void_ptr_type:&'a Type,
    mmap_function :&'a Value,
    prot_value : &'a Value,
    flags_value:&'a Value,
    fd_value:&'a Value,
    offset_value :&'a Value,
    builder:&'a Builder,
}

#[derive(Clone,Copy)]
struct ExtendSize(usize);
fn to_extend(size:usize)->ExtendSize{
    ExtendSize(size*PAGE_SIZE)
}
impl<'a> MMapCaller<'a>{
    const LIMIT_PAGE_SIZE:usize = ::std::usize::MAX / PAGE_SIZE;
    fn extend_linear_memory(&self,liner_memory_ptr:&'a Value,maximum:usize)->&'a Value{


        let (addr,extended_size) = self.partial_extend_linear_memory(liner_memory_ptr, to_extend(Self::LIMIT_PAGE_SIZE), 0,maximum/ Self::LIMIT_PAGE_SIZE );
        let reminder_extend_size = to_extend(maximum % Self::LIMIT_PAGE_SIZE);
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
            let tail_addr = self.builder.build_int_to_ptr(
                self.builder.build_add(self.builder.build_ptr_to_int(mapped_ptr,self.int_type,""),self.builder.build_int_cast( extended_size_value,self.int_type,""),""),
                self.void_ptr_type,
                "tail_addr"
            );
            let args = [tail_addr,extend_size_value,self.prot_value,self.flags_value,self.fd_value,self.offset_value];
            let addr = self.builder.build_call(self.mmap_function,&args,"mapped_ptr");
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
    type Compiler = LinearMemoryCompiler<i32>;

    #[test]
    pub fn compile_works(){
        let  context = Context::new();

        let result = Compiler::compile(&context, 17,Some(25));
        assert!(result.is_ok());
        let _ =result.map(| module|{
            assert!(module.get_named_global(LINEAR_MEMORY_NAME).is_some());
        });
    }

    #[test]
    pub fn init_linear_memory_works() ->Result<(),Error>{
        test_init_linear_memory_in(17,Some(25))
    }

    #[test]
    pub fn init_maximum_65536_linear_memory_works()->Result<(),Error>{
        test_init_linear_memory_in(17,Some(65536))
    }

    #[test]
    pub fn init_maximum_65537_linear_memory_not_works()->Result<(),Error>{
        match test_init_linear_memory_in(17,Some(65537)){
            Ok(_)=>panic!("should be error."),
            Err(_) =>Ok(()),
        }
    }

    fn test_init_linear_memory_in(minimum:usize,maximum:Option<usize>)->Result<(),Error>{
        let context = Context::new();
        let module = Module::new("grow_linear_memory_works",&context);
        let builder = Builder::new(&context);
        Compiler::build_init_linear_memory_function(&module, &builder, minimum,maximum)?;
        analysis::verify_module(&module,analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction)?;
        test_jit_init()?;
        test_module_in_engine(&module,|engine|{
            let mut mapped_liner_memory_size: usize = 0;
            let mut mapped_liner_memory: *mut ::libc::c_void = ::std::ptr::null_mut();
            let liner_memory_size = module.get_named_global(LINEAR_MEMORY_PAGE_SIZE_NAME).ok_or_else(|| NoSuchLLVMGlobalValue { name: LINEAR_MEMORY_PAGE_SIZE_NAME.to_string() })?;
            let liner_memory = module.get_named_global(LINEAR_MEMORY_NAME).ok_or_else(|| NoSuchLLVMGlobalValue { name: LINEAR_MEMORY_NAME.to_string() })?;
            engine.add_global_mapping(liner_memory, &mut mapped_liner_memory);
            engine.add_global_mapping(liner_memory_size, &mut mapped_liner_memory_size);
            let args: [&GenericValue; 0] = [];
            test_run_function_with_name(&engine, &module, Compiler::get_init_linear_memory_function_name().as_ref(), &args)?;
            assert_eq!(mapped_liner_memory_size, minimum);
            assert_ne!(::std::ptr::null_mut(),mapped_liner_memory);
            assert_ne!(-1_isize , unsafe{::std::mem::transmute::<*mut ::libc::c_void ,isize>(mapped_liner_memory)});
            Ok(())
        })?;
        Ok(())
    }


    #[test]
    pub fn set_linear_memory_works(){
        let  context = Context::new();
        let module = Module::new("grow_linear_memory_works",&context);
        let builder = Builder::new(&context);
        assert_ne!(ptr::null(), Compiler::set_linear_memory(&module).as_ptr());
        assert!( module.get_named_global(LINEAR_MEMORY_NAME).is_some());
    }

}