fn main() {
    println!("cargo:rerun-if-env-changed=NOS_USER_ELF");
    if let Ok(path) = std::env::var("NOS_USER_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    // Cargo cannot infer file dependencies from a linker flag in .cargo/config.
    println!("cargo:rerun-if-changed=linker.ld");
}
