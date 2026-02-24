use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=Info.plist");
    println!("cargo:rerun-if-changed=assets/snapmark-icon.svg");
    println!("cargo:rerun-if-changed=assets/icon.png");
    println!("cargo:rerun-if-changed=assets/status_icon_template.png");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is missing"));
    let plist_dst = out_dir.join("Info.plist");
    let _ = fs::copy("Info.plist", plist_dst);
}
