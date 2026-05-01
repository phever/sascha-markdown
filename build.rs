use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("resources.gresource");
    
    // Copy the compiled resource to the OUT_DIR so the macro can find it
    std::fs::copy("res/resources.gresource", dest_path).expect("Failed to copy resource file");
    
    println!("cargo:rerun-if-changed=res/resources.gresource");
}
