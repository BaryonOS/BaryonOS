fn main() {
    // Cargo cannot infer file dependencies from a linker flag in .cargo/config.
    println!("cargo:rerun-if-changed=linker.ld");
}
