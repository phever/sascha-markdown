use std::path::Path;
fn main() {
    let p = Path::new("file.smd");
    let is_smd = p.extension().and_then(|e| e.to_str()).map(|s| s == "smd").unwrap_or(false);
    println!("file.smd -> {}", is_smd);
    
    let p2 = Path::new("file.txt");
    let is_smd2 = p2.extension().and_then(|e| e.to_str()).map(|s| s == "smd").unwrap_or(false);
    println!("file.txt -> {}", is_smd2);
}
