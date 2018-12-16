use super::*;
use failure::Error;
use error::RuntimeError::*;
const SYS_BRK:u64=0x2d;
const SYS_WRITEV:u64=0x92;
const SYS_MMAP2:u64=0xc0;
const SYS_LLSEEK:u64= 0x8c;
const SYS_FUTEX:u64 = 0xf0;

pub fn build_syscalls<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let int_type = Type::int_wasm_ptr::<T>(build_context.context());
    build_syscall1::<T>(build_context,int_type,linear_memory_compiler)?;
    build_syscall3::<T>(build_context,int_type,linear_memory_compiler)?;
    build_syscall5::<T>(build_context,int_type,linear_memory_compiler)?;
    build_syscall6::<T>(build_context,int_type,linear_memory_compiler)?;
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
        let int_ptr_type = Type::int_ptr(build_context.context());
        let int32_type = Type::int32(build_context.context());
        let int64_type = Type::int64(build_context.context());
        let mut cases = vec![];

        if linear_memory_compiler.is_some(){
            cases.push(
                SysCallCase {
                    code: SYS_WRITEV,
                    on_case: Box::new( || {
                        let linear_memory_compiler = linear_memory_compiler.unwrap();
                        let d = build_context.builder().build_int_cast(a,int32_type,"");
                        let iovec = build_get_real_address(build_context, linear_memory_compiler, b);
                        let iovec_count = build_context.builder().build_int_cast(c,int32_type,"");

                        build_context.builder().build_ret(
                            build_context.builder().build_int_cast(
                                build_call_and_set_writev(build_context.module(), build_context.builder(), d, iovec, iovec_count, ""),
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

fn build_syscall5<'m,T:WasmIntType>(build_context:&'m BuildContext,int_type:&'m Type,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
    let syscall5_type = Type::function(int_type,&[int_type,int_type,int_type,int_type,int_type,int_type],false);
    let syscall5 = build_context.module().set_declare_function(&WasmCompiler::<T>::wasm_function_name("__syscall5"),syscall5_type);
    build_context.builder().build_function(build_context.context(),syscall5,|_,_|{
        let n = syscall5.get_first_param().ok_or(NotExistValue)?;
        let a = n.get_next_param().ok_or(NotExistValue)?;
        let b = a.get_next_param().ok_or(NotExistValue)?;
        let c = b.get_next_param().ok_or(NotExistValue)?;
        let d = c.get_next_param().ok_or(NotExistValue)?;
        let e = d.get_next_param().ok_or(NotExistValue)?;
        let int_ptr_type = Type::int_ptr(build_context.context());
        let int32_type = Type::int32(build_context.context());
        let int64_type = Type::int64(build_context.context());
        let mut cases = vec![];
        if linear_memory_compiler.is_some(){
            cases.push(
                SysCallCase{
                    code:SYS_LLSEEK,
                    on_case: Box::new(
                        ||{
                            let linear_memory_compiler = linear_memory_compiler.unwrap();
                            let fd = build_context.builder().build_int_cast(a,int32_type,"");
                            let offset_high = build_context.builder().build_int_cast(b,int_ptr_type,"");
                            let offset_low = build_context.builder().build_int_cast(c,int_ptr_type,"");
                            let result = build_get_real_address(build_context,linear_memory_compiler,d);
                            let whence = build_context.builder().build_int_cast(e,int32_type,"");


                            build_context.builder().build_ret(
                                build_context.builder().build_int_cast(
                                    build_call_and_set_llseek(build_context.module(),build_context.builder(),fd,offset_high,offset_low,result,whence),
                                    int32_type,
                                    ""
                                )
                            );
                            Ok(())
                        }
                    )
                }
            )
        }

        build_syscall_internal(build_context,syscall5,n,&cases,int_type)
    })
}

fn build_syscall6<'m,T:WasmIntType>(build_context:&'m BuildContext,int_type:&'m Type,linear_memory_compiler:Option<&'m LinearMemoryCompiler<T>>)->Result<(),Error>{
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
        let int_ptr_type = Type::int_ptr(build_context.context());
        let int32_type = Type::int32(build_context.context());
        let int64_type = Type::int64(build_context.context());
        let mut cases = vec![];
        if linear_memory_compiler.is_some(){
            cases.push(
                SysCallCase{
                    code:SYS_MMAP2,
                    on_case:Box::new(||{
                        let linear_memory_compiler = linear_memory_compiler.unwrap();

                        let addr = build_get_real_address(build_context,linear_memory_compiler,a);
                        let len = build_context.builder().build_int_cast(b,int_ptr_type,"");
                        let prot = build_context.builder().build_int_cast(c,int32_type,"");
                        let flags = build_context.builder().build_int_cast(d,int32_type,"");
                        let fd = build_context.builder().build_int_cast(e,int32_type,"");
                        let off_t = build_context.builder().build_int_cast(f,int64_type,"");
                        let ret = build_mmap2(build_context,linear_memory_compiler,syscall6,addr ,len,prot,flags,fd,off_t)?;
                        build_context.builder().build_ret(build_context.builder().build_int_cast(ret,int_ptr_type,""));
                        Ok(())
                    })
                }
            );

            cases.push(
                SysCallCase{
                    code:SYS_FUTEX,
                    on_case:Box::new(||{
                        let linear_memory_compiler = linear_memory_compiler.unwrap();

                        let uaddr = build_get_real_address(build_context,linear_memory_compiler,a);
                        let op = build_context.builder().build_int_cast(b,int32_type,"");
                        let val = build_context.builder().build_int_cast(c,int32_type,"");
                        let timeout = build_get_real_address(build_context,linear_memory_compiler,d);
                        let uaddr2 = build_get_real_address(build_context,linear_memory_compiler,e);
                        let val3 = build_context.builder().build_int_cast(f,int32_type,"");
                        let ret = build_call_and_set_futex(build_context.module(),build_context.builder(),uaddr,op,val,timeout,uaddr2,val3);
                        build_context.builder().build_ret(build_context.builder().build_int_cast(ret,int_ptr_type,""));
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

pub fn build_mmap2<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:&'m LinearMemoryCompiler<T>,syscall6:&Value, addr:&Value,len:&'m Value,prot:&'m Value,flags:&'m Value,fd:&'m Value,off_t:&'m Value)->Result<&'m Value,Error>{
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

fn build_call_and_set_brk<'m>(module:&'m Module,builder:&'m Builder,addr:&Value,name:&str)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let brk_type = Type::function(int32_type,&[new_real_pointer_type(context)],false);
    let brk = module.set_declare_function("brk",brk_type);
    builder.build_call(brk,&[addr],name)
}

fn build_call_and_set_writev<'m>(module:&'m Module,builder:&'m Builder,d:&'m Value,iovec:&'m Value,iovec_count:&'m Value,name:&str)->&'m Value{
    let context = module.context();
    let int_type = Type::int_ptr(context);
    let writev_type = Type::function(int_type,&[
        Type::int32(context),
        new_real_pointer_type(context),
        Type::int32(context),
    ],false);
    let writev = module.set_declare_function("writev",writev_type);
    builder.build_call(writev,&[d,iovec,iovec_count],name)
}

fn build_call_and_set_llseek<'m>(module:&'m Module,builder:&'m Builder,fd:&'m Value,offset_high:&'m Value,offset_low:&'m Value,result:&Value,whence:&'m Value)->&'m Value{
    let context = module.context();
    let int_type = Type::int_ptr(context);
    let int32_type = Type::int32(context);
    let ptr_type = new_real_pointer_type(context);
    let llseek_type = Type::function(int32_type,&[int32_type,int_type,int_type,ptr_type,int32_type],false);
    let llseek = module.set_declare_function("_llseek",llseek_type);
    builder.build_call(llseek,&[fd,offset_high,offset_low,result,whence],"")
}

fn build_call_and_set_futex<'m>(module:&'m Module,build_context:&'m Builder,uaddr:&'m Value,op:&'m Value,val:&'m Value,timeout:&'m Value,uaddr2:&'m Value,val3:&'m Value)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let ptr_type = new_real_pointer_type(context);
    let futex_type = Type::function(int32_type,&[ptr_type,int32_type,int32_type,ptr_type,ptr_type,int32_type],false);
    let futex = module.set_declare_function("futex",futex_type);
    build_context.build_call(futex,&[uaddr,op,val,timeout,uaddr2,val3],"")
}

fn new_real_pointer_type(context:&Context)->&Type{
    Type::ptr(Type::struct_create_named(context,"real_struct"),0)
}
