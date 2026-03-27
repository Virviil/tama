fn main() {
    println!("cargo:rerun-if-changed=web/dist");
    println!("cargo:rerun-if-env-changed=TAMAD_LINUX_AMD64_SHA256");
}
