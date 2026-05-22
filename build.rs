fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = std::path::PathBuf::from("proto/yomu");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("yomu_descriptor.bin"))
        .compile_protos(
            &[
                proto_dir.join("usersync.proto"),
                proto_dir.join("quizsync.proto"),
                proto_dir.join("league.proto"),
                proto_dir.join("health.proto"),
            ],
            &[&proto_dir],
        )?;

    Ok(())
}
