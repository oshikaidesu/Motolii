# Rerun `re_ui` module inventory（2026-07-20）

ステータス: **観察／比較中**。`re_ui`を一括採用する決定ではなく、固定sourceの各moduleをMotoliiの既存fixtureへ対応付けるRR-0追補である。候補分類は反対側レビュー前の仮置きであり、依存追加、vendoring、移植、発注を許可しない。

## 1. 先に固定するMotolii側の目的

調査順は[Rerun学習・転移計画 §9](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)に従う。

| 項目 | 現行事実 |
|---|---|
| Motolii authority | M3仕様、[UI境界規律](2026-07-14-m3-ui-boundary-prevention.md)、[UI視覚言語](../ui-visual-language.md)、[UI参照地図](../ui-reference-map.md) |
| 現行code | mainの`motolii-ui`はSlintのlink確認だけを持つ最小骨格。egui component、shell、fixtureはまだmainへ統合されていない |
| 現行prototype | `codex/m3-mock-components`のReact component mapは、primitive 7、pattern 11、surface 12、screen 4の安定IDを持つ。React/DOM/CSS境界は製品APIではない |
| 調査対象gap | eguiでdense component、Browser、form、診断、DnD、snapshot試験をどう小さく成立させるかの実装証拠がmainに無い |
| 非目標 | Rerun風画面、Rerun command、Entity/Blueprint/store、theme値、font/icon、serdeをMotolii要件へ昇格しない |
| 入場条件 | M3入場PRと個別転移裁定より前はread-only調査だけ。M1/M2の公開契約、Document、plugin契約を変更しない |

したがって本調査は`re_uiにあるものをMotoliiへ足す`のではなく、`React安定IDとM3 taskが既に要求する問題に、re_uiのどのfileが実装証拠を提供するか`を答える。

## 2. 再現anchor

| anchor | 用途 | archive SHA-256 |
|---|---|---|
| main監査commit [`954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui) | module/APIの主監査 | `a891a52e4a56ced5f9d438527894d295fefe0f0ba9e10bf0d47a219f94f07af4` |
| 安定release [`0.34.1`](https://github.com/rerun-io/rerun/tree/0.34.1/crates/viewer/re_ui) | 出荷版に存在する範囲と追従差分 | `3c8e251659fc7c3f211c84597a499da39d6417be03841b53e083c5e5c9f3dbb3` |

取得手順:

```sh
curl -L --fail -o /tmp/rerun-main.tar.gz \
  https://github.com/rerun-io/rerun/archive/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e.tar.gz
curl -L --fail -o /tmp/rerun-0.34.1.tar.gz \
  https://github.com/rerun-io/rerun/archive/refs/tags/0.34.1.tar.gz
shasum -a 256 /tmp/rerun-main.tar.gz /tmp/rerun-0.34.1.tar.gz
```

## 3. crate全体を依存してはいけない理由

main監査commitの[`Cargo.toml`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui/Cargo.toml)と`src/`を機械集計した。

- production Rust sourceは59 files、15,196行。安定release 0.34.1は13,846行で、短期間に1,350行増えている
- runtimeに8個の必須内部`re_*` crate、任意の`re_analytics`、build時の`re_build_tools`を要求する
- `eframe(wgpu)`、Wayland、hot reload、Rerun command、Entity/Time型まで同じcrateに含む
- 非コード資産はInter Medium、dark/light theme、color table、103 SVG + 1 PNG icon、logo 2点
- 独立test sourceは703行、snapshot PNGは39点。component実装と試験資産が同居する
- 0.34.1からmain監査commitまでに、button、command palette、context、design token、notification、text edit、time drag、`UiExt`等が変更され、fuzzy matcherとcommand分割が増えた

よって`re_ui` crate全体の`DEPEND`候補はここで**棄却候補**とする。これは個別moduleの`VENDOR/PORT/PATTERN`比較を棄却するものではない。

## 4. module inventory

行数はmain監査commitのRust source。`候補`は観察であって裁定ではない。

| module群（行数） | sourceが実際に持つ責務 | Motolii側の既存入口 | 結合・不足 | 候補 |
|---|---|---|---|---|
| `button` 314、`combo_item` 247、`menu` 38、`section_collapsing_header` 89 | size/variant、icon+text、combo row、compact menu、折畳み見出し | `primitive.action-button`、`primitive.icon-button`、`pattern.panel-header`、U0e | token/icon/egui型へ直結。Motoliiのstate matrixとCJKは未証明 | 限定`PORT`比較 |
| `list_item/*` 2,488 | label/property/custom content、indent、selection、hover button、navigation、collapsing row | Browser hierarchy、Inspector parameter row、Inbox、Settings、U4a/U4d/U6 | まとまった実戦資産だが大きく、Rerun tokenと`UiExt`へ結合。大量行virtualization自体は証明しない | `VENDOR`対`PORT`比較の最優先 |
| `re_form/*` 328 | field strip、分数幅、selectable/toggle、統一frame | `pattern.parameter-row`、`surface.inspector`、U4a | NodeDesc→widget→commandや全型fallbackは持たない | layoutを`PORT`、生成契約はMotolii所有 |
| `filter_widget` 864、`fuzzy` 281 | query state、path match範囲、highlight、`nucleo-matcher`によるfuzzy match | `pattern.discovery-browser-shell`、Browser 3面、U4d/U6 | 0.34.1に`fuzzy.rs`は無くmain側が先行。検索意味、tag、selection commitはMotolii固有 | `nucleo-matcher`直接`DEPEND`対wrapper `PORT`比較 |
| `egui_ext/card_layout` 238 | cardごとの寸法、wrap配置、hover/drag下のinteraction rect | Browser thumbnail/list toggle、candidate shelf、U4d/U6 | virtualization、elide、可変thumbnail設定は別問題 | geometryを`PORT`比較 |
| `drag_and_drop` 367 | 階層listのbefore/inside/after drop zoneとindicator geometry | Browser tag drop、hierarchy、panel layout、U1e/U4d/U6 | MotoliiのCommit Intent、Undo、別parent規則は持たない | geometry `PORT`、command意味 `REJECT` |
| `text_edit` 168 | suggestion popupつきautocomplete | Browser search、parameter text、G0-6 IME | IME preedit/CJK、shortcut抑制、commit/cancelは証明しない | `PATTERN`。IME fixture後だけ`PORT`比較 |
| `command_palette` 457、`command/*` 1,580 | generic providerに見えるpaletteに加え、Rerun recording/server/table/UI commandとshortcutを同梱 | 現行React mapにcommand palette要求なし | command意味、time cursor、recording、serverがRerun固有。mainで大幅更新中 | paletteは要求成立まで`延期`、commandは`REJECT` |
| `modal` 421、`alert` 168 | modal stack、area、button row、alert variants | plugin recovery、Settings、diagnostic dialog、U2c | focus trap、IME、OS accessibilityは未証明 | presentation `PORT`対限定`VENDOR`比較 |
| `notifications` 679 | toast + history、unread level、details field、dismiss/never-show | `pattern.diagnostic-feedback`、`pattern.status-brief`、plugin recovery、U2c | `re_error`/`re_log`を受ける。Motolii typed diagnostic envelopeと抑制寿命は別 | 責任分割`PATTERN`、表示shell `PORT`比較 |
| `loading_indicator` 119 | available rectからradiusを決めるspinner描画 | readiness/rendering/stale、U3f | generation/stale/mailboxを証明せず、理由文字列だけ | `PATTERN`。単独採用しない |
| `design_tokens` 1,188、`color_table` 234、`hot_reload_design_tokens` 166、`context_ext` 212 | RON theme読込、dark/light、style適用、font登録、hot reload | G0-6、UI視覚言語 | Rerun値をDTCG正本へできない。mainと0.34.1でtheme値も変化 | pipelineは`PATTERN`、token値/serdeは`REJECT` |
| `icons` 264、`icon_text` 100 | compile-time SVG/PNG registry、image loader、icon+shortcut text | Motolii icon grid、transport、Browser、panel actions | 103 SVGにRerun固有語彙とlogoを含む。MotoliiはLucide候補を別審査中 | loader `PATTERN`、asset一括`REJECT` |
| `ui_layout` 223、`egui_ext/*` 688 | left/right/center layout、group、boxed widget、response/widget text helpers | shared primitive/pattern全般 | 多くはegui convenience。公開component境界へすべきでない | 必要fileだけ`PORT`比較 |
| `ui_ext` 1,433 | link、checkbox/radio、list item、markdown、copy、context menu等の巨大extension trait | 複数surface | unrelated helperとanalytics/error/hyperlinkが集中し、追従差分も大きい | 一括`VENDOR/DEPEND`は`REJECT`、関数単位で再照合 |
| `relative_time_range` 415、`time_drag_value` 280、`time` 61 | sequence/timestamp/durationの入力と表示 | Timeline、transport、parameter scrub | Rerun `TimeType`/`TimeInt`/`Timestamp`へ直結。Motolii `RationalTime`、audio clock、Undoと異なる | widget責任`PATTERN`、型/実装`REJECT` |
| `syntax_highlighting` 511 | EntityPath、InstancePath、time、URL等の色分け | 現行M3要求なし | `re_entity_db`/`re_log_types`を直接要求 | `REJECT` |
| `help` 203、`markdown_utils` 39 | OS別shortcut/help表示、markdown用input表現 | help surfaceは現行安定IDに無い | product動線が先に必要 | `延期` |
| `testing` 52 + integration tests 703 + snapshots 39 | theme付き`egui_kittest` harness、UI/3D snapshot option、component state fixture | U0e/G0-6、React reference比較 | snapshotだけではIME、性能、意味、accessibilityを証明しない | harness責任とstate matrixを`PATTERN` |
| `wayland` 218 + window decoration helpers | Wayland decoration交渉、OS別custom frame既定 | U1 shell | eframe/lifecycle採択後のOS実機問題。制作動線とは独立 | `PATTERN`、問題再現前の移植は`REJECT` |

## 5. React安定IDから見た優先照合

`re_ui` module名から着手順を作らず、既存動線から次の小さい問題へ分ける。

| 優先 | Motolii problem / fixture | Rerun evidence file | 次に必要なMotolii oracle |
|---|---|---|---|
| A | BrowserのSearch / Sources / Collections / Resultsで、長いCJK名を1行elideしつつ検索一致を読める | `filter_widget.rs`、`fuzzy.rs`、`card_layout.rs` | `screen.plugin-browser-candidate`と同一データ、日英中韓+pseudo-locale、keyboard/IME、thumbnail/list 3表示 |
| B | Inspector/Browser/Settingsで同じdense rowを複製しない | `list_item/*`、`re_form/*` | `primitive.*`/`pattern.parameter-row` state matrix、hover/focus/disabled/error、NodeDesc fallback |
| C | 階層とtagへのdrop位置を、文字や色だけでなく形とindicatorで判別する | `drag_and_drop.rs` | before/inside/afterの負例、別parent、cancel、1 gesture=1 command、Document不変のpreview |
| D | typed diagnosticをtoast、history、recoveryへ一貫投影する | `notifications.rs`、`alert.rs`、`modal.rs` | U2c envelope、dedupe、dismiss寿命、詳細copy、正常時非占有、screen reader/focus |
| E | componentを感想でなくstate fixtureで固定する | `testing.rs`、`tests/*.rs`、39 snapshots | Motolii dark/light/custom theme、CJK、grayscale、pseudo-locale、interaction testを証跡分離 |

command palette、Rerun time widget、syntax highlight、window decorationは上のA〜Eを横断させない。必要なMotolii problemが別に成立した時だけ新しい照合ticketを作る。

## 6. 日本語・UTF-8/CJKの判断

懸念は文字列encodingより**glyph供給・fallback・IME・固定密度**にある。

- Rust/eguiの文字列経路はUTF-8だが、それだけでは日本語glyphを描けない
- [`DesignTokens::set_fonts`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui/src/design_tokens.rs#L564-L580)は同梱`Inter-Medium.otf`を比例fontの先頭へ置くだけで、CJK fontを同梱・選択していない
- Rerun自身の[CJK issue #12770](https://github.com/rerun-io/rerun/issues/12770)がegui/eframe側作業待ちであるため、Rerunの出荷実績をMotoliiのCJK合格証拠にしてはいけない
- `text_edit.rs`のautocompleteはIME preedit、candidate window、shortcut衝突の合格証拠ではない

したがってInterとRerun font設定の一括転移は`REJECT`候補。MotoliiはG0-6で日本語・英語・簡体中文・韓国語、pseudo-locale、caret、IME preeditを同じfixtureへ通し、font/license/binary sizeを別裁定する。

## 7. M1/M2への関与

`re_ui`の直接の席はM3であり、M1/M2実装へ依存・vendoring・型を持ち込まない。

- M1/plugin: `re_form`はHost所有の自動生成panelを実装するM3先例に限る。第三者pluginへegui UIを開放する根拠ではなく、`NodeDesc`/plugin公開契約を変えない
- M2/Document: list、DnD、time input、notificationのegui state/px/serdeをDocument、journal、commandへ流さない
- 横断して学べるのはcomponent fixture、typed boundary、snapshot harnessという`PATTERN`だけ。M1/M2の完成条件を`re_ui`に合わせて変更しない

## 8. 次の裁定単位

最初の個別反対側レビューは、Aの**Browser検索・結果表示だけ**を対象にする。

1. React `screen.plugin-browser-candidate`の同一fixtureを固定する
2. 現行Motolii codeで未成立の事実をM3入場後に再確認する
3. `nucleo-matcher`直接依存、小さい自作、`fuzzy/filter/card`限定PORTを比較する
4. CJK/IME、elide、keyboard、selection commit、100k item性能をMotolii oracleにする
5. `DEPEND/VENDOR/PORT/PATTERN/REJECT`を一つ裁定してから、初めて発注可否を判断する

`list_item`は価値が大きい一方、2,488行とtoken結合を持つため、Browser照合へ便乗させない。Bとして独立裁定する。
