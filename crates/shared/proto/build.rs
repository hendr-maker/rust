fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile auth service proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/auth.proto"], &["proto/"])?;

    // Compile user service proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/user.proto"], &["proto/"])?;

    Ok(())
}
