use super::*;
use failure::Error;
use error::RuntimeError::*;
const SYS_BRK:u64=0x2d;
const SYS_WRITEV:u64=0x92;
const SYS_MMAP2:u64=0xc0;

pub fn build_syscalls<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let int_type = Type::int_wasm_ptr::<T>(build_context.context());
    build_syscall1::<T>(build_context,int_type,linear_memory_compiler)?;
    build_syscall3::<T>(build_context,int_type,linear_memory_compiler)?;
    Ok(())
}


struct SysCallCase<'a>{
    code:u64,
    on_case:Box<Fn()->Result<(),Error> + 'a>
}



fn build_syscall1<'m,T:WasmIntType>(build_context:&'m BuildContext,int_type:&'m Type,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let syscall1_type = Type::function(int_type,&[int_type,int_type],false);
    let syscall1 = build_context.module().set_declare_function(&WasmCompiler::<T>::wasm_function_name("__syscall1"),syscall1_type);
    build_context.builder().build_function(build_context.context(),syscall1,|_,_|{
        let n = syscall1.get_first_param().ok_or(NotExistValue)?;
        let a = n.get_next_param().ok_or(NotExistValue)?;
        let mut cases = vec![];
        if linear_memory_compiler.is_some(){
            cases.push(SysCallCase{code:SYS_BRK,on_case: Box::new( ||{
                let linear_memory_compiler = linear_memory_compiler.unwrap();
                let sys_brk_ret = build_call_and_set_brk(build_context.module(),build_context.builder(),build_get_real_address(build_context,linear_memory_compiler,a),"");
                build_context.builder().build_ret(sys_brk_ret);
                Ok(())
            })
            })
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

        let mut cases = vec![];

        if linear_memory_compiler.is_some(){
            cases.push(
                SysCallCase {
                    code: SYS_WRITEV,
                    on_case: Box::new( || {
                        let linear_memory_compiler = linear_memory_compiler.unwrap();
                        let iovec = build_get_real_address(build_context, linear_memory_compiler, b);

                        build_context.builder().build_ret(
                            build_context.builder().build_int_cast(
                                build_call_and_set_writev(build_context.module(), build_context.builder(), a, iovec, c, ""),
                                int_type,
                                ""
                            )
                        );


                        Ok(())
                    })
                }
            )
        }

        build_syscall_internal(build_context,syscall3,n,&cases,int_type)

    })
}

fn build_syscall6<'m,T:WasmIntType>(build_context:&BuildContext,int_type:&'m Type,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let syscall6_type = Type::function(int_type,&[int_type,int_type,int_type,int_type,int_type,int_type,int_type],false);
    let syscall6 = build_context.module().set_declare_function(&WasmCompiler::<T>::wasm_function_name("__syscall6"),syscall6_type);
    build_context.builder().build_function(build_context.context(),syscall6,|_,_|{
        let n = syscall6.get_first_param().ok_or(NotExistValue)?;
        let a = n.get_next_param().ok_or(NotExistValue)?;
        let b = a.get_next_param().ok_or(NotExistValue)?;
        let c = b.get_next_param().ok_or(NotExistValue)?;
        let d = c.get_next_param().ok_or(NotExistValue)?;
        let e = d.get_next_param().ok_or(NotExistValue)?;
        let f = e.get_next_param().ok_or(NotExistValue)?;
        let mut cases = vec![];
        if linear_memory_compiler.is_some(){
            cases.push(
                SysCallCase{
                    code:SYS_MMAP2,
                    on_case:Box::new(||{
                        let linear_memory_compiler = linear_memory_compiler.unwrap();
                        let ret = build_mmap2(build_context,linear_memory_compiler,syscall6,a ,b,c,d,e,f)?;
                        build_context.builder().build_ret(ret);
                        Ok(())
                    })
                }
            )
        }
        build_syscall_internal(build_context,syscall6,n,&cases,int_type)
    })
}


fn build_get_real_address<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:&'m LinearMemoryCompiler<T>,addr:&'m Value)->&'m Value{
    let ptr = linear_memory_compiler.build_get_real_address(build_context,0,addr,"");
    build_context.builder().build_pointer_cast( ptr ,new_real_pointer_type(build_context.context()),"")
}

pub fn build_mmap2<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:&'m LinearMemoryCompiler<T>,syscall6:&Value, addr:&Value,len:&'m Value,prot:&'m Value,flats:&'m Value,fd:&'m Value,off_t:&'m Value)->Result<&'m Value,Error>{
    let unit_size = Value::const_int(Type::int_ptr(build_context.context()),LinearMemoryTypeContext::UNIT_SIZE as u64,false);
    let requested =  build_context.builder().build_add(
        build_context.builder().build_udiv(len,unit_size,""),
        build_context.builder().build_urem(len,unit_size,""),
        ""
    );

    let grow_ret = linear_memory_compiler.build_grow(build_context,requested,0)?;
    Ok(build_context.builder().build_mul(grow_ret,unit_size,""))
}

fn build_syscall_internal<'m>(build_context:&'m BuildContext,function:&'m Value,  n:&'m Value,cases:&'m [SysCallCase],int_type:&'m Type)->Result<(),Error>{
    let builder = build_context.builder();
    for case in cases{
        let case = case;
        let code = Value::const_int(int_type,case.code,false);
        let cond = builder.build_icmp(IntPredicate::LLVMIntEQ,code,n,"");
        let then_block = function.append_basic_block(build_context.context(),"");
        let else_block = function.append_basic_block(build_context.context(),"");
        builder.build_cond_br(cond,then_block,else_block);
        builder.position_builder_at_end(then_block);
        (case.on_case)()?;
        builder.position_builder_at_end( else_block);
    }
    builder.build_ret(Value::const_int(int_type,0,false));
    Ok(())
}

