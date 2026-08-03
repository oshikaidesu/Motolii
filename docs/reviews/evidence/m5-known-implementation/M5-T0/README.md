# M5-T0 text stack comparison receipt

状態: **PASS / KEEP + REDUCE**（2026-08-02）

## Scope

現行のFontique＋HarfRust単一font shapingと、Parley 0.11のprivate layout／fallback／BiDi／clusterを、
製品workspace外で比較した。fixtureは`Latin ffi café | 日本語 | שלום עולם | 👩‍🔬`、missing glyphは
`A`＋U+10FFFF、variation settingは`'wght' 650`である。これはP6公開API、Vello接続、編集／selection、
全HarfBuzz conformance、3 OS再現、製品runtime完成の証拠ではない。

## Fixed sources and licenses

| source | fixed version | checksum | license |
|---|---|---|---|
| `fontique` | crates.io `0.11.0` | `1e04c4750a17111ebd77c3e0aea00476ce33f59235bc4d9e7f0aded5033ad3fc` | MIT OR Apache-2.0 |
| `harfrust` | crates.io `0.10.0` | `f0589ddd0d2935dd2845827ac606b4081c266225d613b268ed2910f832889cab` | MIT OR Apache-2.0 |
| `parley` | crates.io `0.11.0` | `e0478b47dd9885a5e0a4f1c0782ffc42bf6ee8c41dea0917d7a9bcee3e6585fc` | Apache-2.0 OR MIT |
| `parley_data` | crates.io `0.11.0` | `a649e01a1acc917247ee147b56b8a1fa91824acf7117bd003b4204306d601255` | Apache-2.0 OR MIT |

standalone `Cargo.lock`は77 packageを固定し、製品workspaceへ依存を追加していない。

## Environment and commands

- macOS 15.5 (24F74), Apple arm64
- rustc／cargo 1.96.1
- system font discoveryはFontique／Parleyの公式system featureに限定

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-T0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-T0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-T0/Cargo.toml -- --nocapture
```

## Oracle results

| oracle | result | evidence |
|---|---|---|
| direct Fontique＋HarfRust | PASS | 36 glyphs、34 clusters、単一fontで不足glyph `0`を診断 |
| Parley fallback／layout | PASS | 8 runs、39 clusters、35 glyphs、RTL 3 runs、emoji 2 clusters |
| cluster preservation | PASS | combining／ZWJをcluster単位で回収し、glyph 0を無言置換しない |
| missing glyph diagnostic | PASS | U+10FFFF入力で`glyph_id == 0`をtyped probe結果へ残す |
| variation request | REDUCE | `'wght' 650`をAPIへ渡したが、host選択fontにvariable axisがなくnormalized coordsは0。未対応を採用と誤認しない |
| BiDi／script itemize | PASS | 手書きitemize 0。Parley＋ICU4XがRTL runを生成 |

実行時にICU4Xの日本語segmentation model不足メッセージが出たが、layout／cluster／RTL oracleはPASSで、
日本語segmenterの完全性はこのreceiptの採択範囲に含めない。

## Disposition

- 現行Fontique＋HarfRust: **KEEP / REUSE**。既知の局所run経路をP6実装の比較基準にする。
- Parley 0.11: **KEEP / PRIVATE ADOPTION PROBE**。fallback／BiDi／clusterの手書き実装を避ける候補として残す。
- Parley全layout／editing／公開API: **REDUCE**。このprobeからMotolii公開契約へ昇格させない。
- variation: **REDUCE / RETEST**。variable fontを固定fixtureへ供給できるまで採用判定を保留する。

## Remaining gates

Fontique＋HarfRust＋Velloの製品run API、fallback診断UI、variation固定fixture、全script／vertical text、
編集／selection、Windows／Linux、P6公開意味、M5-A0Sの歴史decision recoveryは未完了である。
