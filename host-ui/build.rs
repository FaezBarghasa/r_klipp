fn main() {
    // Recompile when any slint or asset changes
    println!("cargo:rerun-if-changed=ui/");
    slint_build::compile("ui/main.slint").unwrap();
}
