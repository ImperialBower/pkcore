fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only compile proto files if tonic-build is available
    // This allows the project to build without gRPC dependencies
    #[cfg(feature = "grpc")]
    {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile_protos(&["proto/dealer.proto"], &["proto"])?;
    }
    Ok(())
}
