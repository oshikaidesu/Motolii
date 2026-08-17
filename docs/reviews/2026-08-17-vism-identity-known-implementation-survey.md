# Vism identity 未決6問の既知実装対照

作成日: 2026-08-17

状態: **比較中**（候補の提示であり、採否は未決）

対象: [VSM-B0 identity期待値マトリクス](2026-08-17-vsm-b0-identity-fixture.md) が残した `UNDETERMINED` 6問。

関連: [既知実装採択モデル](../known-implementation-adoption-model.md)、[Vism既知実装採択マップ](../vism-known-implementation-adoption-map.md)、[Vismコンセプト](../vism-package-concept.md)

## 0. なぜこの形にしたか

`concept.md:110` は「**発明工程を持たない**」として、`identity`、Undo、layout、scheduler、codec等を「解決済みのOSS、標準、実装patternを採択し、薄いtranslation／admission adapter、製品policy、fixtureだけを製品固有codeとして持つ」と定める。**identityは名指しで採択対象である。**

一方 [Vism既知実装採択マップ](../vism-known-implementation-adoption-map.md) は、表現側（Depth provider、Mesh deformation、Glass surface等）には `K-WGPU` / `K-RERUN-SPATIAL` / `K-GRAPH` を固定しているが、基盤側（Generator／Materialize、Kit／Preset／typed graph、package／admission／配布）には既知解を置かず `B0→B1→B2` へ送っている。施工順の5番目は明示的に「**engine／containerを選ばず**」と書く。

その結果 `VSM-B0` は identity の意味論を自分の台帳から演繹し、36セルが `UNDETERMINED` になった。**台帳が黙っているのは決めていないからではなく、参照先が外にあるからである。**

本書は「engine／containerを選ばない」を維持したまま、**意味論のパターンだけ**を先行実装から拾う。container選択（npm風／OCI／Nix風）は別問であり、本書では決めない。

## 1. 対照表

`REUSE`（そのまま使う）／`PATTERN`（解き方を写す）／`WRAP`（境界を挟んで使う）／`REJECT` は[既知実装採択モデル](../known-implementation-adoption-model.md)の語彙に従う。

| 問い | 既知実装が出している答え | 分類候補 |
|---|---|---|
| **U1** version更新をまたぐentry identity | **OFX**: `pluginIdentifier` は「その plugin と**その関連versionすべて**に対する一意のid」であり、**serialize してplugin を同定するために使う**。version は `pluginVersionMajor` / `pluginVersionMinor` の別フィールドで、majorのincrementは後方互換の破壊を意味する | `PATTERN` |
| **U2** fork差替え後のProject instance identity | 直接の先行実装が見つからなかった。cargoの`[patch]`／`[replace]`はsourceを差し替えても参照名を保つが、これは依存グラフの話でDocument内instanceの採番ではない。**本書では候補を出せない** | 未充足 |
| **U3** 表示名とartifactの関係 | **CLAP**: `clap_plugin_descriptor_t` が `id` / `name` / `vendor` / `version` を**別フィールド**として持つ。**OFX**: `pluginIdentifier` と表示名が別。**cargo**: lockfileは `name` + `version` + `source` + `checksum` を並べ、checksumは内容に対して取る | `PATTERN` |
| **U4** Projectがartifact identityを固定するか | **cargo**: `Cargo.lock` が name／version／source URL に加え **checksum** を記録し、取得物がそれと一致するか検証する。lockfileを持つビルドマシンへ改竄物を送っても検出される | `PATTERN` |
| **U5** 同一version再導入とartifactの同一性 | **cargo**: 同一性は**仮定せず検証する**。同じversionを取り直しても checksum 照合を通す。content-addressed store（Nix等）は同じ考えをstore path自体に埋める | `PATTERN` |
| **U6** entry identityがpackageに閉じるか | **OFX**: `OfxGetNumberOfPlugins()` と `OfxGetPlugin(int nth)` で**1 binaryが複数pluginを持つ**。各pluginは自分の `pluginIdentifier` を持つ。**CLAP**: factoryの `get_plugin_count` / `get_plugin_descriptor(index)` で同じ形。**どちらも entry identity は bundle path から導出されない** | `PATTERN` |

## 2. 読み取れること

**U1・U6 は同じ答えを指している。** OFXとCLAPはどちらも「1つの配布物が複数のentryを持ち、entryは自分のidを持ち、versionは別フィールド」という形である。`VSM-B0` が追加したケース5（一packageが異なるkindのentryを複数持つ）は、**この2規格では既定の形**である。B0が「ケース5を足しても未決が増えなかった」と観測したのは、外を見れば当然だったことになる。

**U3・U4・U5 は同じ道具で解けている。** 表示名とidentityを別フィールドに分け、内容はhashで同定し、同一性は仮定せず検証する。cargo／npm／Nixが共通して採る形である。`VSM-B0` で未決の半分（18/36）がartifact identityに集中したのは、この道具立てを台帳が持っていなかったためで、決め方が難しいからではない。

**U2 だけ性質が違う。** これはDocument内のinstance採番の問題で、パッケージ管理の領域に対応物が無い。`VSM-B0` の他5問と同じ棚に置いたのが誤りかもしれない。Undo／複製／参照の所有者はProject Documentであり、[Vism / Kitモデル](../vism-kit-model.md) の側で閉じる問いである可能性が高い。

## 3. 本書が決めないこと

- **採否**。上表は候補であり、`REUSE / PATTERN / WRAP / REJECT` の確定は別の裁定による
- **container／engine の選択**。`vism-known-implementation-adoption-map.md` の「engine／containerを選ばず」は維持する。意味論のパターンを採ることと、配布形式を採ることは別である
- **schema、manifest key、型**。`VSM-B3` の領域
- **U2 の答え**。先行実装が見つからなかったため、埋めずに残す

## 4. 出典

- [The OfxPlugin Struct — OpenFX 1.5.1](https://openfx.readthedocs.io/en/main/Reference/ofxPluginStruct.html)
- [Packaging OFX Plug-ins — OpenFX 1.5.1](https://openfx.readthedocs.io/en/main/Reference/ofxPackaging.html)
- [clap/include/clap/plugin.h](https://github.com/free-audio/clap/blob/main/include/clap/plugin.h)
- [Track checksum in Cargo.lock · rust-lang/cargo#4800](https://github.com/rust-lang/cargo/issues/4800)
- [Checksum in cargo_lock::package](https://docs.rs/cargo-lock/latest/cargo_lock/package/enum.Checksum.html)

出典はいずれも**一次資料または規格の公式ドキュメント**である。ただし本書はAPIヘッダを直接読んではおらず、公式docsの記述に依拠する。採択票を書く段階では `clap/include/clap/plugin.h` と `ofxCore.h` の原文にあたること。
