extern crate filetime;

use std::env;
use std::process::Command;
use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsString;
fn main(){


    let out_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let dir = "test_cases";
    build_wat(dir,&format!("{}/{}",out_dir,dir));

    Command::new("wat2wasm").args(&["test_cases/wasm_compiler/return_only/return_only.wat","-o"])
        .arg(&format!("{}/return_only.wasm",out_dir))
        .status()
        .unwrap();

}


fn build_wat(dir:&str,out_dir:&str){
    fs::create_dir_all(out_dir);
    for entry in fs::read_dir(dir).unwrap(){
        let entry = entry.unwrap();
        let meta_data = entry.metadata().unwrap();
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();
        let entry_path = entry.path();
        let ext = entry_path.extension().map_or_else(||"".to_string(),|ext|ext.to_str().unwrap_or("").to_string());
        let stem = entry_path.file_stem().map_or_else(||"".to_string(),|v|v.to_str().unwrap_or("").to_string());

        let out_path = OsString::from( format!("{}/{}",out_dir,stem ));
        let out_path = Path::new(  &out_path) ;
        let path_in_project = OsString::from(format!("{}/{}",dir,file_name));
        let path_in_project = Path::new(&path_in_project);
        let path_in_project_str = path_in_project.to_str().unwrap_or("");

        if   meta_data.is_file() && ext == "wat"  &&  (!out_path.exists() || filetime::FileTime::from_last_access_time(&meta_data) >= filetime::FileTime::from_last_access_time(&out_path.metadata().unwrap())) {
            Command::new("wat2wasm").args(&[path_in_project_str,"-o"])
                .arg(&format!("{}.wasm",out_path.to_str().unwrap()))
                .status()
                .unwrap();
        } else if meta_data.is_dir(){
            build_wat(path_in_project_str,out_path.to_str().unwrap());
        }

    }
}
