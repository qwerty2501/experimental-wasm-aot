use super::*;
use failure::Error;
use error::RuntimeError::*;
const WASM_SYS_UMASK:u64 = 60;
const WASM_SYS_BRK:u64=45;
const WASM_SYS_WRITEV:u64=146;
const WASM_SYS_MMAP2:u64=192;
const WASM_SYS_LLSEEK:u64= 140;
const WASM_SYS_FUTEX:u64 = 240;
const WASM_SYS_IOCTL:u64 = 54;

const SYS_FUTEX:u64 = 202;
const SYS_LLSEEK:u64 = 2;

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
    let int32_type = Type::int32(build_context.context());
    build_context.builder().build_function(build_context.context(),syscall1,|_,_|{
        let n = syscall1.get_first_param().ok_or(NotExistValue)?;
        let a = n.get_next_param().ok_or(NotExistValue)?;
        let mut cases = vec![];
        if linear_memory_compiler.is_some(){
            /*
            cases.push(SysCallCase{code: WASM_SYS_BRK,on_case: Box::new( ||{
                let linear_memory_compiler = linear_memory_compiler.unwrap();
                let sys_brk_ret = build_call_and_set_brk(build_context.module(),build_context.builder(),build_get_real_address(build_context,linear_memory_compiler,a),"");
                build_context.builder().build_ret(sys_brk_ret);
                Ok(())
            })
            });

            cases.push(
                SysCallCase{
                    code: WASM_SYS_UMASK,
                    on_case:Box::new(
                        ||{
                            let mask = build_context.builder().build_int_cast(a,int32_type,"");
                            let ret = build_call_and_set_umask(build_context.module(),build_context.builder(),mask);
                            build_context.builder().build_ret(
                                build_context.builder().build_int_cast(
                                    ret,
                                    int_type,
                                    ""
                                )
                            );
                            Ok(())
                        }
                    )
                }
            )
            */
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
        let real_ptr_type = new_real_pointer_type(build_context.context());
        let mut cases = vec![];
        if linear_memory_compiler.is_some(){
            /*
            cases.push(
                SysCallCase {
                    code: WASM_SYS_WRITEV,
                    on_case: Box::new( || {

                        let ret = build_call_and_set_writev::<T>(build_context.module(), build_context.builder(), a, b, c, "");
                        build_ret(build_context,ret,int_type);

                        Ok(())
                    })
                }
            );
            */
/*
            cases.push(
                SysCallCase{
                    code: WASM_SYS_FUTEX,
                    on_case:Box::new(||{
                        let linear_memory_compiler = linear_memory_compiler.unwrap();
                        let uaddr = build_get_real_address(build_context,linear_memory_compiler,a);
                        let op = build_context.builder().build_int_cast(b,int32_type,"");
                        let val = build_context.builder().build_int_cast(c,int32_type,"");
                        let ret = build_call_and_set_futex3(build_context.module(),build_context.builder(),uaddr,op,val);
                        build_ret(build_context,ret,int_type);
                        Ok(())
                    })
                }
            );

            cases.push(
                SysCallCase{
                    code: WASM_SYS_IOCTL,
                    on_case:Box::new(
                        ||{
                            let linear_memory_compiler = linear_memory_compiler.unwrap();
                            let fd = build_context.builder().build_int_cast(a,int32_type,"");
                            let request = build_context.builder().build_int_cast(b,int_ptr_type,"");
                            let argp = build_get_real_address(build_context,linear_memory_compiler,c);
                            let ret = build_call_and_set_ioctl(build_context.module(),build_context.builder(),fd,request,&[argp]);
                            build_ret(build_context,ret,int_type);
                            Ok(())
                        }
                    )
                }
            )
            */
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
            // TODO: should implements in the near future.
            /*
            cases.push(
                SysCallCase{
                    code: WASM_SYS_LLSEEK,
                    on_case: Box::new(
                        ||{

                            let linear_memory_compiler = linear_memory_compiler.unwrap();
                            let fd = build_context.builder().build_int_cast(a,int32_type,"");
                            let offset_high = build_context.builder().build_int_cast(b,int_ptr_type,"");
                            let offset_low = build_context.builder().build_int_cast(c,int_ptr_type,"");
                            let result = build_get_real_address(build_context,linear_memory_compiler,d);
                            let whence = build_context.builder().build_int_cast(e,int32_type,"");
                            let ret = build_call_and_set_llseek(build_context.module(),build_context.builder(),fd,offset_high,offset_low,result,whence);
                            build_ret(build_context,ret,int_type);
                            Ok(())
                        }
                    )
                }
            )
            */
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
                    code: WASM_SYS_MMAP2,
                    on_case:Box::new(||{
                        let linear_memory_compiler = linear_memory_compiler.unwrap();

                        let addr = build_get_real_address(build_context,linear_memory_compiler,a);
                        let len = build_context.builder().build_int_cast(b,int_ptr_type,"");
                        let prot = build_context.builder().build_int_cast(c,int32_type,"");
                        let flags = build_context.builder().build_int_cast(d,int32_type,"");
                        let fd = build_context.builder().build_int_cast(e,int32_type,"");
                        let off_t = build_context.builder().build_int_cast(f,int64_type,"");
                        let ret = build_mmap2(build_context,linear_memory_compiler,syscall6,addr ,len,prot,flags,fd,off_t)?;
                        build_ret(build_context,ret,int_type);
                        Ok(())
                    })
                }
            );
/*
            cases.push(
                SysCallCase{
                    code: WASM_SYS_FUTEX,
                    on_case:Box::new(||{
                        let linear_memory_compiler = linear_memory_compiler.unwrap();

                        let uaddr = build_get_real_address(build_context,linear_memory_compiler,a);
                        let op = build_context.builder().build_int_cast(b,int32_type,"");
                        let val = build_context.builder().build_int_cast(c,int32_type,"");
                        let timeout = build_get_real_address(build_context,linear_memory_compiler,d);
                        let uaddr2 = build_get_real_address(build_context,linear_memory_compiler,e);
                        let val3 = build_context.builder().build_int_cast(f,int32_type,"");
                        let ret = build_call_and_set_futex6(build_context.module(),build_context.builder(),uaddr,op,val,timeout,uaddr2,val3);
                        build_ret(build_context,ret,int_type);
                        Ok(())
                    })
                }
            )
*/
        }
        build_syscall_internal(build_context,syscall6,n,&cases,int_type)
    })
}

fn build_ret<'m>(build_context:&'m BuildContext,ret:&'m Value,int_type:&'m Type){
    build_context.builder().build_ret(
        build_context.builder().build_int_cast(
            ret,
            int_type,
            ""
        )
    );
}
fn build_get_real_address<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:&'m LinearMemoryCompiler<T>,addr:&'m Value)->&'m Value{
    linear_memory_compiler.build_get_real_address(build_context,0,addr,"")
}

pub fn build_mmap2<'m,T:WasmIntType>(build_context:&'m BuildContext,linear_memory_compiler:&'m LinearMemoryCompiler<T>,syscall6:&Value, addr:&Value,len:&'m Value,prot:&'m Value,flags:&'m Value,fd:&'m Value,off_t:&'m Value)->Result<&'m Value,Error>{
    let unit_size = Value::const_int(Type::int_wasm_ptr::<T>(build_context.context()),LinearMemoryTypeContext::UNIT_SIZE as u64,false);
    let len = build_context.builder().build_int_cast(len,Type::int_wasm_ptr::<T>(build_context.context()),"");
    let requested =  build_context.builder().build_add(
        build_context.builder().build_udiv(len,unit_size,""),
        build_context.builder().build_urem(len,unit_size,""),
        ""
    );

    let requested = build_context.builder().build_int_cast(requested,Type::int_wasm_ptr::<T>(build_context.context()),"");

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

fn build_call_and_set_writev<'m,T:WasmIntType>(module:&'m Module,builder:&'m Builder,d:&'m Value,iovec:&'m Value,iovec_count:&'m Value,name:&str)->&'m Value{
    let context = module.context();
    let int_type = Type::int_wasm_ptr::<T>(context);
    let writev_type = Type::function(int_type,&[
        int_type,
        int_type,
        int_type,
    ],false);
    let func_name = WasmCompiler::<T>::wasm_function_name("writev_c");
    let writev = module.set_declare_function(&func_name,writev_type);
    builder.build_call(writev,&[d,iovec,iovec_count],name)
}


fn build_call_and_set_futex3<'m>(module:&'m Module,builder:&'m Builder,uaddr:&'m Value,op:&'m Value,val:&'m Value)->&'m Value{
    let context = module.context();

    let int_type = Type::int_ptr(context);
    let n = Value::const_int(int_type,SYS_FUTEX,false);
    let params = vec![
        builder.build_ptr_to_int(uaddr,int_type,""),
        builder.build_int_cast(op,int_type,""),
        builder.build_int_cast(val,int_type,""),
    ];
    build_call_and_set_libc_syscall(module, builder, n, &params)
}

fn build_call_and_set_futex6<'m>(module:&'m Module,builder:&'m Builder,uaddr:&'m Value,op:&'m Value,val:&'m Value,timeout:&'m Value,uaddr2:&'m Value,val3:&'m Value)->&'m Value{
    let context = module.context();

    let int_type = Type::int_ptr(context);
    let n = Value::const_int(int_type,SYS_FUTEX,false);
    let params = vec![
        builder.build_ptr_to_int(uaddr,int_type,""),
        builder.build_int_cast(op,int_type,""),
        builder.build_int_cast(val,int_type,""),
        builder.build_ptr_to_int(timeout,int_type,""),
        builder.build_ptr_to_int(uaddr2,int_type,""),
        builder.build_int_cast(val3,int_type,""),
    ];
    build_call_and_set_libc_syscall(module, builder, n, &params)
}

fn build_call_and_set_umask<'m>(module:&'m Module,builder:&'m Builder,mask:&'m Value)->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let umask_type = Type::function(int32_type,&[int32_type],false);
    let umask = module.set_declare_function("umask",umask_type);
    builder.build_call(umask,&[mask],"")
}

fn build_call_and_set_ioctl<'m>(module:&'m Module,builder:&'m Builder,fd:&'m Value,request:&'m Value,argp:&[&'m Value])->&'m Value{
    let context = module.context();
    let int32_type = Type::int32(context);
    let int_ptr_type = Type::int_ptr(context);
    let real_ptr_type = new_real_pointer_type(context);
    let ioctl_type = Type::function(int32_type,&[int32_type,int_ptr_type,real_ptr_type],true);
    let mut params = vec![fd,request];
    params.extend_from_slice(argp   );
    let ioctl = module.set_declare_function("ioctl",ioctl_type);
    builder.build_call(ioctl,&params,"")
}

fn build_call_and_set_libc_syscall<'m>(module:&'m Module, builder:&'m Builder, n:&'m Value, args:&[&'m Value]) ->&'m Value{
    let context = module.context();
    let int_type = Type::int_ptr(context);
    let syscall_type = Type::function(int_type,&[int_type,int_type],true);
    let syscall = module.set_declare_function("syscall",syscall_type);
    let mut params = vec![n];
    params.extend_from_slice(args);
    builder.build_call(syscall,&params,"")
}

fn new_real_pointer_type(context:&Context)->&Type{
    Type::ptr(Type::int8(context),0)
}
