fn main() {
    use re_build_tools::Environment;

    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let vism_dir = manifest_dir.join("vism");
    println!("cargo:rerun-if-changed={}", vism_dir.display());
    let mut paths: Vec<_> = std::fs::read_dir(&vism_dir)
        .expect("vism directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("wgsl" | "fs" | "frag" | "glsl" | "json")
            )
        })
        .collect();
    paths.sort();
    let mut inventory = String::from("pub(crate) const VISM_SOURCES: &[EmbeddedVismSource] = &[\n");
    for path in paths {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("vism stem");
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .expect("vism extension");
        let file = path
            .file_name()
            .and_then(|file| file.to_str())
            .expect("vism file");
        inventory.push_str(&format!(
            "    EmbeddedVismSource {{ name: {name:?}, extension: {extension:?}, source: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/vism/{file}\")) }},\n"
        ));
    }
    inventory.push_str("];\n");
    // 作者の札の絵(manifest の `THUMBNAIL` が名指す png)は shader と同じ dir に置く。
    let mut pictures: Vec<_> = std::fs::read_dir(&vism_dir)
        .expect("vism directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("png"))
        .collect();
    pictures.sort();
    inventory.push_str("pub(crate) const VISM_PICTURES: &[EmbeddedVismPicture] = &[\n");
    for path in pictures {
        let file = path.file_name().and_then(|file| file.to_str()).expect("picture file");
        inventory.push_str(&format!(
            "    EmbeddedVismPicture {{ file: {file:?}, bytes: include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/vism/{file}\")) }},\n"
        ));
    }
    inventory.push_str("];\n");
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("out dir"));
    std::fs::write(out_dir.join("vism_inventory.rs"), inventory).expect("write vism inventory");

    let environment = Environment::detect();
    let is_release = cfg!(not(debug_assertions));
    let targets_wasm =
        re_build_tools::get_and_track_env_var("CARGO_CFG_TARGET_FAMILY").unwrap() == "wasm";

    println!("cargo::rustc-check-cfg=cfg(load_shaders_from_disk)");

    let load_shaders_from_disk =
        environment == Environment::DeveloperInWorkspace && !is_release && !targets_wasm;
    if load_shaders_from_disk {
        println!("cargo:rustc-cfg=load_shaders_from_disk");
    }
}
