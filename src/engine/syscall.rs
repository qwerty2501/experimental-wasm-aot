use super::*;
use failure::Error;
use error::RuntimeError::*;
const SYS_BRK:u64=45;
const SYS_WRITEV:u64=146;

pub fn build_syscalls<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let int_type = Type::int_wasm_ptr::<T>(build_context.context());
    build_syscall1::<T>(build_context,int_type,linear_memory_compiler)?;
    build_syscall3::<T>(build_context,int_type,linear_memory_compiler)?;
    Ok(())
}


struct SysCallCase <F:Fn()->Result<(),Error>>{
    code:u64,
    on_case:F
}

trait SysCallCaseTrait{
    fn syscall_code(&self)->u64;
    fn case(&self)->Result<(),Error>;
}

impl<F:Fn()->Result<(),Error>> SysCallCaseTrait for SysCallCase<F>{
    fn syscall_code(&self) -> u64 {
        self.code
    }

    fn case(&self) -> Result<(), Error> {
        (self.on_case)()
    }
}

fn build_syscall1<'m,T:WasmIntType>(build_context:&'m BuildContext,int_type:&'m Type,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let syscall1_type = Type::function(int_type,&[int_type,int_type],false);
    let syscall1 = build_context.module().set_declare_function(&WasmCompiler::<T>::wasm_function_name("__syscall1"),syscall1_type);
    build_context.builder().build_function(build_context.context(),syscall1,|_,_|{
        let n = syscall1.get_first_param().ok_or(NotExistValue)?;
        let a = n.get_next_param().ok_or(NotExistValue)?;
        let mut cases:Vec<Box<SysCallCaseTrait>> = vec![];
        if linear_memory_compiler.is_some(){
            cases.push(Box::new(
                SysCallCase{code:SYS_BRK,on_case: ||{
                    let linear_memory_compiler = linear_memory_compiler.unwrap();
                    let sys_brk_ret = build_call_and_set_brk(build_context.module(),build_context.builder(),build_get_real_address(build_context,linear_memory_compiler,a),"");
                    build_context.builder().build_ret(sys_brk_ret);
                    Ok(())
                }
                }
            ))
        }
        build_syscall_internal(build_context,syscall1,n,&cases,int_type)

    })
}


fn build_syscall3<'m,T:WasmIntType>(build_context:&'m BuildContext,int_type:&'m Type,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let syscall3_type = Type::function(int_type,&[int_type,int_type,int_type,int_type],false);
    let syscall3 = build_context.module().set_declare_function(&WasmCompiler::<T>::wasm_function_name("__syscall3"),syscall3_type);
    build_context.builder().build_function(build_context.context(),syscall3,|_,_|{
        let n = syscall3.get_first_param().ok_or(NotExistValue)?;
        let a = n.get_next_param().ok_or(NotExistValue)?;
        let b = a.get_next_param().ok_or(NotExistValue)?;
        let c = b.get_next_param().ok_or(NotExistValue)?;

        let mut cases:Vec<Box<SysCallCaseTrait>> = vec![];

        if linear_memory_compiler.is_some(){
            cases.push(
                Box::new(
                    SysCallCase{
                        code:SYS_WRITEV,
                        on_case:||{
                            let linear_memory_compiler = linear_memory_compiler.unwrap();
                            let iovec = build_get_real_address(build_context,linear_memory_compiler,b);

                            build_context.builder().build_ret(
                                build_context.builder().build_int_cast(
                                    build_call_and_set_writev(build_context.module(),build_context.builder(),a,iovec,c,""),
                                    int_type,
                                    ""
                                )
                            );



                            Ok(())
                        }
                    }
                )
            )
        }
        build_syscall_internal(build_context,syscall3,n,&cases,int_type)

    })
}



fn build_get_real_address<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:&'m LinearMemoryCompiler<T>,addr:&'m Value)->&'m Value{
    let ptr = linear_memory_compiler.build_get_real_address(build_context,0,addr,"");
    build_context.builder().build_pointer_cast( ptr ,new_real_pointer_type(build_context.context()),"")
}

fn build_syscall_internal<'m>(build_context:&'m BuildContext,function:&'m Value,  n:&'m Value,cases:&'m [Box<SysCallCaseTrait + 'm>],int_type:&'m Type)->Result<(),Error>{
    let builder = build_context.builder();
    for case in cases{
        let case = case.as_ref();
        let code = Value::const_int(int_type,case.syscall_code(),false);
        let cond = builder.build_icmp(IntPredicate::LLVMIntEQ,code,n,"");
        let then_block = function.append_basic_block(build_context.context(),"");
        let else_block = function.append_basic_block(build_context.context(),"");
        builder.build_cond_br(cond,then_block,else_block);
        builder.position_builder_at_end(then_block);
        case.case()?;
        builder.position_builder_at_end( else_block);
    }
    builder.build_ret(Value::const_int(int_type,0,false));
    Ok(())
}

