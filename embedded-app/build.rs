
use std::env;
use std::fs;
use std::io::Write;
use std::str::FromStr;


fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    println!("cargo:rustc-link-arg-bins=-Trom_functions.x");

    println!("cargo:rerun-if-changed=../client-app/dist");

    include_client_app();
}



fn include_client_app(){
    let client_app_dir = env::var("CARGO_MANIFEST_DIR").unwrap() + "/../client-app/dist";
    
    let mut file = fs::File::create("src/client_app.rs").unwrap();

    file.write_all(b"// Do Not Modify! \n// This file is generated automaticly at compile time\n\n").unwrap();
    
    let entries = fs::read_dir(&client_app_dir);
    match entries{
        Ok(entries) => {
            for entry in fs::read_dir(&client_app_dir).unwrap(){
                let entry = entry.unwrap();
                if entry.file_name().to_str().unwrap() == "index.html"{
                    let path = String::from_str(entry.path().to_str().unwrap()).unwrap().replace("\\", "/");
                    file.write_all(b"pub const INDEX_HTML: &[u8] = include_bytes!(\"").unwrap();
                    file.write_all(path.as_bytes()).unwrap();
                    file.write_all(b"\");\n").unwrap();
                    break;
                }
            }
            let count = fs::read_dir(&client_app_dir).unwrap().count() - 1;
            file.write_all(b"pub const CLIENT_APP_FILES: [(&str, &[u8]);").unwrap();
            file.write_all(count.to_string().as_bytes()).unwrap();
            file.write_all(b"] = [\n").unwrap();
            for entry in entries {
                let entry = entry.unwrap();
                if entry.file_name().to_str().unwrap() == "index.html"{
                    continue;
                }
                let path = String::from_str(entry.path().to_str().unwrap()).unwrap().replace("\\", "/");
                file.write_all(b"   (\"").unwrap();
                file.write_all(entry.file_name().to_str().unwrap().as_bytes()).unwrap();
                file.write_all(b"\", include_bytes!(\"").unwrap();
                file.write_all(path.as_bytes()).unwrap();
                file.write_all(b"\")),\n").unwrap();
            }
            file.write_all(b"];").unwrap();
        },
        Err(_) => {
            file.write_all(b"// Error reading client-app dir!\n\n").unwrap();
            file.write_all(b"pub const CLIENT_APP_FILES: [(&str, &[u8]);0] = []").unwrap();
        }
    }
}
