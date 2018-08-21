#![feature(libc)]
#![feature(const_fn)]
#![deny(unused_must_use)]
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate nameof;
extern crate llvm_sys;
extern crate core;
extern crate parity_wasm;
extern crate env_logger;
extern crate libc;
extern crate num;

use std::env;

#[macro_use] mod engine;
pub mod error;

use failure::Error;
fn main() {
    env_logger::init();
    ::std::process::exit(match build( &env::args().collect::<Vec<_>>()){
        Ok(_)=>0,
        Err(e)=>{
            println!("{}",e);
            for cause in  e.causes(){
                debug!("causes:{cause}",cause = cause);
            }

            debug!("backtrace:{backtrace}", backtrace = e.backtrace());
            1
        },
    });


}


fn build(args:&[String])->Result<(),Error>{

    let wasm_file_name:&str =  args.get(2).ok_or(error::RuntimeError::Application{
        name:"Now, the argument is given only wasm file.".to_string()
    })?;

    let wasm_module = parity_wasm::deserialize_file(wasm_file_name)?;

    let engine = engine::Engine::<u32>::new();
    engine.build(&wasm_module)

}
