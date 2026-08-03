# M5-A2 OBJ lowering compatibility probe

M5-A1のGLB preflight後に、`tobj`でOBJを同じprivate faithful asset境界へlowerできるかを検証する。
製品workspace、Document、serde、公開API、renderer、M4 resource ownerには接続しない。

## 検証する境界

- OBJのtriangle／single-index化をprivate meshへ保持する。
- normal／UVの欠落を`Option`のまま保持し、推測値を生成しない。
- MTLがない、または参照先を取得できない場合を`MaterialBinding::Missing`として保持する。
- MTLの値をglTF PBRやunlitへ黙って変換しない。
- malformed OBJとmaterial index不整合をpanicでなく型付き失敗へする。

## Commands

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-A2/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-A2/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-A2/Cargo.toml
```

## Disposition

`tobj`はOBJ入口のprivate leafとしてKEEP候補にする。OBJをglTFへ変換したこと、MTLがPBRとして
忠実であること、製品asset cache／importer／rendererが成立したことはこのprobeでは証明しない。
