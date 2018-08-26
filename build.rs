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
    build_wat(Path::new(dir), Path::new(&out_dir).join(dir).as_path());

    Command::new("wat2wasm").args(&["test_cases/wasm_compiler/return_only/return_only.wat","-o"])
        .arg(&format!("{}/return_only.wasm",out_dir))
        .status()
        .unwrap();

}


fn build_wat(dir:&Path,out_dir:&Path){
    fs::create_dir_all(out_dir);
    for entry in fs::read_dir(dir).unwrap(){
        let entry = entry.unwrap();
        let meta_data = entry.metadata().unwrap();
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();
        let entry_path = entry.path();
        let ext = entry_path.extension().map_or_else(||"".to_string(),|ext|ext.to_str().unwrap_or("").to_string());
        let stem = entry_path.file_stem().map_or_else(||"".to_string(),|v|v.to_str().unwrap_or("").to_string());

        let out_path = out_dir.join(&stem);
        let path_in_project = dir.join(file_name);

        if   meta_data.is_file() && ext == "wat"  &&  (!out_path.exists() || filetime::FileTime::from_last_access_time(&meta_data) >= filetime::FileTime::from_last_access_time(&out_path.metadata().unwrap())) {
            Command::new("wat2wasm").args(&[path_in_project.to_str().unwrap(),"-o"])
                .arg(&format!("{}.wasm",out_path.to_str().unwrap()))
                .status()
                .unwrap();
        } else if meta_data.is_dir(){
            build_wat(path_in_project.as_path(),out_path.as_path());
        }

    }
}
