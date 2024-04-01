use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("grpc.bin");
    tonic_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .compile(&["src/grpc/grpc.proto"], &["src/grpc"])?;

    Ok(())
}
