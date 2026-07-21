# M3 U0e-1 DTCG token generator契約

作成日: 2026-07-21
状態: **決定 / U0e-1実装完了**

実装は`crates/motolii-ui-token-gen`と
`crates/motolii-ui/tests/fixtures/u0e1-token-generator/`に閉じた。
`generate`はfixture生成だけを明示実行し、通常検証はread-onlyの`check`を使う。
製品token、`egui::Style`、component、theme設定には接続していない。

## 1. 目的

U0e-1は、DTCG 2025.10 token JSONからRust / egui用の型付きadapter sourceを
決定的に生成し、生成物の手編集と生成漏れをread-only checkで拒否する。

このチケットは生成**機構**だけを実装する。Motolii Dark / Lightの具体色、
spacing、radius、icon寸法、motion値、component state、製品theme選択・fallbackは
G0-6H後のU0e-3まで確定・導入しない。U0e-1で使うtoken値は
`tests/fixtures/`内の合成fixtureに限り、製品binary、製品component、User settingsへ
接続しない。

## 2. 正本と証拠の処分

- [UI視覚言語「tokenの格納形式」](../ui-visual-language.md#tokenの格納形式)は、
  DTCG 2025.10 JSONを単一正本とし、generator version、入力hash、生成先を固定して
  手編集・未生成差分を拒否すると決定している
- [Uシリーズ分割](2026-07-16-m3-ui-concept-to-tickets.md#32-tokencomponenticonを3段階に分ける)は、
  U0e-1をgenerator、U0e-2を5画面fixture、G0-6Hを人間審判、
  U0e-3を製品値・component導入へ分離している
- DTCG Format / Color Module 2025.10は交換formatの正本である。
  Motoliiのrole名、製品値、egui投影先を定める根拠ではない
- PR #184は証拠・抽出元であり、採用するのは型付きflatten、安定順生成、
  theme間key parity、manifest、生成差分検査という**機構候補**だけである
- PR #184のSlint source/output、`build.rs`自動上書き、具体token値、
  theme preference、shell、fallback、region ID、contrast閾値実装は持ち込まない
- PR #184の範囲外値を`clamp`して受理する挙動と、`$schema`の部分文字列一致は棄却する。
  不正入力は元値を変えず型付き拒否する

## 3. 所有境界と生成先

generatorは新しい非公開workspace tool crate
`crates/motolii-ui-token-gen`に置く。

- crateは`publish = false`とし、Document、journal、plugin、render、User settingsへ
  依存しない
- crate自身は`egui`へ依存しない。egui型名を含むRust sourceを生成するだけとし、
  toolkit依存は生成先の`motolii-ui`内に閉じる
- 製品crateの`build.rs`から生成物を上書きしない。生成は明示的なCLI `generate`だけ、
  CIと通常検証はファイルを書かない`check`だけを使う
- U0e-1の正本入力とcommit対象生成物は
  `crates/motolii-ui/tests/fixtures/u0e1-token-generator/`だけに置く。
  生成Rustはintegration testから`include!`してegui 0.35でcompileするが、
  `motolii-ui::lib`、製品app、style、theme selectorへ接続しない
- generatorのlibrary APIはtool crate内の検証用であり、
  `motolii-ui`や他の製品crateから公開再exportしない

## 4. U0e-1で受理するDTCG閉subset

入力はUTF-8 JSON objectで、rootの`$schema`が
`https://www.designtokens.org/schemas/2025.10/format.json`と完全一致しなければならない。
root、group、token、`$value`、`$extensions`内部を含む**全階層の全JSON object**で
重複keyをlast-winsにせず、JSON pathを持つ型付きerrorとして拒否する。

token/group名はDTCG 2025.10の禁止文字を満たすことに加え、Motolii生成subsetでは
`[a-z][a-z0-9]*(?:[-_][a-z0-9]+)*`へ限定する。非ASCII、大文字、空白、
先頭数字、連続separator、末尾separatorを拒否し、transliterateやcase foldをしない。
tokenは`$value`を持つobject、groupは`$value`を持たないobjectとして区別し、
tokenとchild token/groupの混在を拒否する。`$type`はtoken自身または最も近い親groupから
継承し、値の形から型を推測しない。「最も近い親」は即時親に限定せず、
tokenからroot方向へ祖先groupを走査して最初に`$type`を持つgroupを指す。
途中に`$type`の無いgroupが何段あっても走査を続け、token自身の`$type`が常に優先する。

U0e-1の生成対象は、現行UI視覚言語が具体値確定前から要求する次の4型だけとする。

| DTCG `$type` | 受理値 | Rust / egui生成形 |
|---|---|---|
| `color` | object `{ "colorSpace": "srgb", "components": [r, g, b], "alpha"?: a }`。array長は3、4値はJSON numberかつ有限・`0..=1`。alpha省略時は`1.0`。記載外keyを拒否 | component/alphaを`round(value * 255)`した`egui::Color32::from_rgba_unmultiplied` |
| `dimension` | exact object `{ "value": number, "unit": "px" }`。有限・非負。記載外keyを拒否 | eguiのDPI変換前logical pointとして`f32` |
| `duration` | exact object `{ "value": number, "unit": "ms" | "s" }`。有限・非負。記載外keyを拒否 | millisecondへ正規化した`f32` |
| `cubicBezier` | JSON array `[x1, y1, x2, y2]`。長さexact 4、全要素numberかつ有限。x1/x2だけ`0..=1`、y1/y2はclampしない | `[f32; 4]` |

`dimension`の`px`は物理framebuffer pixelやDocument座標ではなく、
eguiへ渡す前のlogical UI単位として1:1に扱う。DPI変換値をtokenや生成物へ焼かない。
`rem`、他color space、`number`、composite型、alias/reference、`$root`、
`$extends`はU0e-1で意味を発明せず、variantとtoken pathを持つ
`UnsupportedFeature`として拒否する。必要になった型は別契約で追加する。

`$description`はstring、`$deprecated`はbooleanまたはstring、
`$extensions`はobjectの場合だけ値生成に寄与しないmetadataとして受理する。
`$extensions`内部はopaque JSONとして走査生成しないが、重複keyは拒否する。
未知の`$` propertyは黙って無視しない。root以外の`$schema`も拒否する。
groupに置かれた`$type`も使用有無によらず4型以外は即時拒否する。
空groupは許すが、token 0件は拒否する。
入力は1 MiB、nest 32段、4096 token、1 segment 128 byte、完全path 512 byteを上限とし、
超過を型付き拒否する。

## 5. 複数themeと識別

library/CLIは`ThemeSource { id, bytes }`の1件以上を入力とする。
theme IDもtoken segmentと同じ
`[a-z][a-z0-9]*(?:[-_][a-z0-9]+)*`だけを許す。
ID重複を拒否し、入力順ではなくID昇順で生成する。

全themeはtoken pathの集合と各pathの型が完全一致しなければならない。
一方の欠落、余分なpath、型違いを`ThemeMismatch`として全差分つきで拒否し、
後勝ち、既定値補完、最初のthemeへのfallbackを行わない。

token pathからRust fieldへは、各segment内の`-`を`_`へ置換し、path segment境界を
`__`で連結する。入力grammar上case変換は起きない。たとえば
`color.surface-raised`は`color__surface_raised`となる。変換後fieldの衝突と
Rust 2021 keyword完全一致は元pathを含む型付きerrorとして拒否し、
suffix付与や順序依存のrenameで隠さない。

theme enum variantはtheme IDを`-`または`_`で分割し、各wordの最初のASCII小文字だけを
大文字化して連結する。`fixture-dark`と`fixture_dark`はいずれも`FixtureDark`になるため
同じbundleでは衝突として拒否する。数字は変換しない。入力grammarにより空word、
先頭数字、非ASCIIは発生しない。変換後variantがRust 2021 keyword（`Self`を含む）と
完全一致する場合はtheme IDを持つ型付きerrorとして拒否し、別の変換規則を加えない。

## 6. 決定的生成とprovenance

`GENERATOR_ID`は`motolii-ui-token-gen`、
`GENERATOR_VERSION`は初版`1`で固定する。生成bundleは次の2 fileだけを持つ。

1. `tokens.rs`: 自動生成header、generator ID/version、bundle input SHA-256、
   path由来の型付きfieldを持つ`GeneratedTheme`、theme IDの閉enum、
   各themeを返す関数
2. `manifest.json`: schema URI、generator ID/version、bundle input SHA-256、
   ID昇順theme一覧、path昇順token一覧と型、生成先相対path

SHA-256入力は、ID昇順themeごとに`id byte length`を**8-byte unsigned u64 big-endian**、
ID byte列、`source byte length`を**8-byte unsigned u64 big-endian**、元JSON byte列の順に
連結したbyte列とする。theme件数やseparatorを別途入れない。入力上限によりlength変換は
常に成功し、変換失敗をwrapしない。
file path、mtime、absolute directory、locale、hash map iteration、CLI引数順を含めない。

全JSON numberはまず`serde_json::Number`から`f64`へ取り出し、`f64::is_finite`と
§4の範囲を判定する。その後だけ`as f32`で変換し、変換後もfiniteであることを検査する。
durationの`s`→`ms`乗算もf64で行い、finite/range確認後にf32化する。
color量子化は検証済みf64に対し`(value * 255.0).round()`を使う。
Rust `f64::round`どおりhalf-wayは0から遠い整数へ丸め、結果をu8へ変換する。
範囲外をcast/clampで隠さない。

token/path/themeはUTF-8 byteの辞書順へ固定し、改行はLF、最終newlineは1個とする。
浮動値は生成Rustでlowercase 8桁hexの`f32::from_bits(0x........)`として出力し、
formatter差を避ける。generator version 1の`tokens.rs`は次の順・文面を正本とする。

1. `// @generated by motolii-ui-token-gen v1; DO NOT EDIT.`
2. `// input-sha256: <lowercase 64 hex>`
3. 空行
4. `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`付き`pub enum GeneratedThemeId`。
   variantはID昇順、4-space indent、1行1variant
5. 空行
6. `#[derive(Debug, Clone, Copy, PartialEq)]`付き`pub struct GeneratedTheme`。
   fieldはpath昇順、4-space indent、型はcolor=`egui::Color32`、
   dimension/duration=`f32`、cubicBezier=`[f32; 4]`
7. 空行
8. `pub fn generated_theme(id: GeneratedThemeId) -> GeneratedTheme`。
   ID昇順match armの中でpath昇順に全fieldを構築する。
   colorは`egui::Color32::from_rgba_unmultiplied(r, g, b, a)`、
   他数値は上記`f32::from_bits`を使う

enum/struct/functionのbrace、comma、空行は次のplaceholder templateを繰り返す。
`<...>`自体は出力しない。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedThemeId {
    <Variant>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeneratedTheme {
    pub <field>: <type>,
}

pub fn generated_theme(id: GeneratedThemeId) -> GeneratedTheme {
    match id {
        GeneratedThemeId::<Variant> => GeneratedTheme {
            <field>: <value>,
        },
    }
}
```

generator自身がこのformatを直接出し、別工程で`rustfmt`を呼ばない。
fixtureの生成sourceをparse/compileする試験を正とする。

`manifest.json`は`serde_json::to_string_pretty`の2-space indent、UTF-8、
同関数のescape、LF、最終newline1個で、top-level key順とshapeを次に固定する。
manifest専用のSerialize structを使い、object内keyも記載順を変えない。

```json
{
  "schema": "https://www.designtokens.org/schemas/2025.10/format.json",
  "generator": {
    "id": "motolii-ui-token-gen",
    "version": 1
  },
  "input_sha256": "<lowercase 64 hex>",
  "themes": [
    "<ID昇順>"
  ],
  "tokens": [
    {
      "path": "<path昇順>",
      "type": "<DTCG type>"
    }
  ],
  "outputs": [
    "tokens.rs",
    "manifest.json"
  ]
}
```

同じ入力byte集合は、別directory、異なるmtime、逆CLI順、連続2回の実行でも
2 fileがbyte一致しなければならない。

## 7. CLIと手編集拒否

CLIは次の閉じた操作だけを持つ。

- `generate --theme <id>=<path>... --out-dir <dir>`:
  全入力をmemory上で検証・生成してから2 fileを書き出す。validation/generation errorでは
  既存出力へ一切書かない。disk I/O errorでは片方だけ更新される可能性を明示的に許すが、
  successを返さない。後続`check`はその時点の2 fileを独立に期待byteと比較し、
  観測されたbundleが一致すれば成功、不一致ならdriftとする。I/O errorだけを理由に
  driftを捏造せず、cross-file atomicityやrollbackも偽装しない
- `check --theme <id>=<path>... --out-dir <dir>`:
  同じmemory生成結果と既存2 fileをbyte比較する。file欠落、余分な対象file、
  1 byteの手編集、古いgenerator version/input hashを型付きdriftとして非zero終了する。
  fileは作成・更新・削除しない

`generate`は存在しない`out-dir`、空directory、またはentry名が
`tokens.rs`/`manifest.json`の部分集合で各entryがregular fileのdirectoryを受理する。
`check`は両方のregular fileが存在する場合だけbyte比較へ進み、0件または1件なら
`MissingOutput`とする。両operationともsubdirectory、symlink、hidden fileを含む
上記2名以外の全entryを`UnexpectedOutputEntry`とする。
「余分な対象file」という部分集合allowlistは作らない。

`check`を`build.rs`やformatterによる再生成で緑にせず、commit済み生成物との差分を
read-onlyで検出する。`generate`後にRust formatterやJSON formatterを別工程で当てない。
生成formatの変更は`GENERATOR_VERSION`を上げ、fixture出力を同じ変更で更新する。

## 8. 自動審判

1. 合成2-theme fixtureが4型を各1件以上持ち、生成`tokens.rs`を
   `motolii-ui` integration testからcompileして全値とtheme IDを参照できる
2. 同一入力2回、別directory、mtime差、逆theme引数順で`tokens.rs`とmanifestがbyte一致する
3. commit済み生成物に1 byte変更、欠落、余分な対象fileを注入すると`check`が拒否し、
   対象directoryの全byteとentry集合が実行前後で不変
4. 4型の境界値を受理し、範囲外component/alpha/x、負dimension/duration、
   非有限化を狙う入力、unit/type/value不一致をclampや推測なしで拒否する
5. 親groupからの`$type`継承を受理し、型不明、token-child混在、重複JSON key、
   禁止名、変換後field/variant衝突を型付き拒否する
6. themeの欠落/余分path/型違い、unsupported alias/`$extends`/typeを型付き拒否する
7. manifestのversion、schema URI、SHA-256、theme/path/type、生成先が実生成と一致する
8. generator crateの依存にegui/eframe/winit、Motolii製品crateが無く、
   生成fixture以外に具体token値、theme ID、raw color/dimension/motion値を追加しない
9. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
   `./scripts/check-ui-toolkit-deps.sh`、
   `cargo clippy --workspace --all-targets -- -D warnings`、
   `cargo test --workspace`を通す

commit済みfixtureの生成差分検査は次で再現する。

```bash
cargo run -p motolii-ui-token-gen -- check \
  --theme fixture-dark=crates/motolii-ui/tests/fixtures/u0e1-token-generator/sources/fixture-dark.json \
  --theme fixture-light=crates/motolii-ui/tests/fixtures/u0e1-token-generator/sources/fixture-light.json \
  --out-dir crates/motolii-ui/tests/fixtures/u0e1-token-generator/generated
```

## 9. 非目標

- Motolii Dark / Light / custom themeの製品値・既定値・fallback
- contrast、lightness、grayscale、CVD、reference screen（U0e-2）
- 人間による階層・識別・馴染み・過剰装飾審判（G0-6H）
- 製品`egui::Style`、component state、icon、font、gradient allowlistへの適用（U0e-3）
- theme選択のUser settings codec、hot reload、filesystem watcher
- DTCG全type、Resolver Module、alias、group extension、remote resource
- runtime custom-theme loader、診断UI、fallback
- Document、journal、Undo、plugin契約、render/evalへのtoken保存

## 10. STOP条件

次のいずれかが必要に見えた時点で実装を止める。

- 合成fixture外の具体色、spacing、radius、icon、duration、easing値を製品値として選ぶ
- 生成物を製品app/style/componentへ接続する
- `build.rs`がcommit済み生成物を自動上書きする
- 不正値をclamp、default、first-theme fallbackで受理する
- Display文字列、raw JSON文字列走査、key名の部分一致で型やroleを推測する
- 未対応DTCG型、alias、extensionの意味を局所実装する
- egui/eframe/winit依存をtool crateまたは`motolii-ui`外の製品crateへ出す
- 公開API、Document、User settings形式、plugin契約、永続形式の変更が必要になる
