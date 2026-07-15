fn main() {
  let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = manifest_dir.join("generated");
  #[path = "src/token_gen.rs"]
  mod token_gen;
  token_gen::write_generated(&manifest_dir, &out_dir).expect("token generation");

  let theme = std::fs::read_to_string(out_dir.join("theme.slint")).expect("theme.slint");
  let body = std::fs::read_to_string(manifest_dir.join("ui/app.slint")).expect("app.slint");
  let bundled = out_dir.join("app.bundle.slint");
  std::fs::write(&bundled, format!("{theme}\n{body}")).expect("bundle slint");

  let config = slint_build::CompilerConfiguration::new()
    .with_bundled_translations(manifest_dir.join("i18n"));
  slint_build::compile_with_config(bundled, config).expect("slint compile");

  println!("cargo:rerun-if-changed=tokens/");
  println!("cargo:rerun-if-changed=ui/");
  println!("cargo:rerun-if-changed=i18n/");
  println!("cargo:rerun-if-changed=src/token_gen.rs");
}
