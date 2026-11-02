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
    println!("cargo:rerun-if-env-changed=NOS_IMAGE_VERSION");
    for name in [
        "NOS_VOLUME_SERVICE_ELF",
        "NOS_VOLUME_APP_ELF",
        "NOS_UPDATE_SERVICE_ELF",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        if let Ok(path) = std::env::var(name) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    for name in [
        "NOS_DESKTOP_INIT_ELF",
        "NOS_DESKTOP_CONTROLLER_ELF",
        "NOS_MATERIAAL_ELF",
        "NOS_COMPOSITOR_ELF",
        "NOS_INPUT_SOURCE_ELF",
        "NOS_INPUT_POLICY_ELF",
        "NOS_RANDOM_PROBE_ELF",
        "NOS_IMAGE_PROBE_ELF",
        "NOS_DESKTOP_INIT_ELF",
        "NOS_DESKTOP_CONTROLLER_ELF",
        "NOS_MATERIAAL_ELF",
        "NOS_COMPOSITOR_ELF",
        "NOS_INPUT_SOURCE_ELF",
        "NOS_INPUT_POLICY_ELF",
        "NOS_DISPLAY_PROBE_ELF",
        "NOS_INPUT_PROBE_ELF",
        "NOS_DISPLAY_INTRUDER_ELF",
        "NOS_MEMORY_ELF",
        "NOS_FPU_A_ELF",
        "NOS_FPU_B_ELF",
        "NOS_NET_PROBE_ELF",
        "NOS_NETWORK_SERVICE_ELF",
        "NOS_NETWORK_CLIENT_ELF",
        "NOS_SYSTEM_INIT_ELF",
        "NOS_SYSTEM_APP_ELF",
        "NOS_SYSTEM_WORKER_ELF",
        "NOS_KEYRING_SERVICE_ELF",
        "NOS_KEYRING_APP_ELF",
        "NOS_SUPERVISOR_ELF",
        "NOS_WORKER_ELF",
        "NOS_CLIENT_ELF",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        if let Ok(path) = std::env::var(name) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    println!("cargo:rerun-if-changed=linker.ld");
}
