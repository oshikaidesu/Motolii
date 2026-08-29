//! `re_renderer` の `load_shaders_from_disk` cfg は crate ごとに立てる必要がある
//! (crate 単位の cargo cfg のため)。判定は `re_ui/build.rs` と同じ形。

fn main() {
    use re_build_tools::Environment;

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
