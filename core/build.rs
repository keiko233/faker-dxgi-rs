fn main() {
    if std::env::var("FAKE_GPU_NAME").is_err() {
        println!("cargo:rustc-env=FAKE_GPU_NAME=NVIDIA GeForce GTX 1050 Ti");
    }
    println!("cargo:rerun-if-env-changed=FAKE_GPU_NAME");
}
