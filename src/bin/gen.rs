use walkdir::WalkDir;

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_path = "./proto"; // this is the proto folder from penumbra's github repo
    let protos: Vec<PathBuf> = WalkDir::new(proto_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("proto".as_ref()))
        .inspect(|p| println!("{}", p.path().display()))
        .map(|e| e.path().to_owned())
        .collect();

    let e = tonic_build::configure()
        .build_client(true)
        .file_descriptor_set_path("descriptor.bin")
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile(&protos, &["./proto/penumbra", "./proto/rust-vendored"]);

    if let Err(err) = &e {
        println!("{}", err);
    }
    Ok(())
}
