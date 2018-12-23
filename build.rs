extern crate filetime;

use std::env;
use std::process::Command;
use std::fs;
use std::error::Error;
use std::path::Path;
use std::ffi::OsString;
use std::io::ErrorKind;

const WAT_COMPILER:&str = "wat2wasm";
fn main(){

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);

    let out_dir = env::var("CARGO_TARGET_DIR").map(|v|Path::new(&v).to_path_buf()).unwrap_or_else(|_| manifest_dir.join("target"));
    let dir = "test_cases";
    if let Ok(_) = Command::new(WAT_COMPILER).spawn(){
        build_wat(manifest_dir.join(dir).as_path(), out_dir.join(dir).as_path());
    }
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
        let out_wasm_path = out_path.with_extension("wasm");
        let path_in_project = dir.join(file_name);

        if   meta_data.is_file() && ext == "wat"  &&  (!out_wasm_path.exists() || filetime::FileTime::from_last_modification_time(&meta_data) >= filetime::FileTime::from_last_modification_time(&out_wasm_path.metadata().unwrap())) {

            if let Err(e) =Command::new(WAT_COMPILER).args(&[path_in_project.to_str().unwrap(),"-o"])
                .arg(&out_wasm_path)
                .status() {
            }
        } else if meta_data.is_dir(){
            build_wat(path_in_project.as_path(),out_path.as_path());
        }

    }
}
