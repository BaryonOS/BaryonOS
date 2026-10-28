fn main() {
    println!("cargo:rerun-if-env-changed=NOS_QSIM_ELF");
    if let Ok(path) = std::env::var("NOS_QSIM_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=NOS_M6C_ELF");
    if let Ok(path) = std::env::var("NOS_M6C_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=NOS_M6B_ELF");
    if let Ok(path) = std::env::var("NOS_M6B_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=NOS_M6_ELF");
    if let Ok(path) = std::env::var("NOS_M6_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=NOS_M5_ELF");
    if let Ok(path) = std::env::var("NOS_M5_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=NOS_USER_ELF");
    if let Ok(path) = std::env::var("NOS_USER_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    // Cargo cannot infer file dependencies from a linker flag in .cargo/config.
    println!("cargo:rerun-if-env-changed=NOS_IPC_ELF");
    if let Ok(path) = std::env::var("NOS_IPC_ELF") {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-changed=linker.ld");
}
