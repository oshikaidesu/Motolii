# Rerun `re_ui` module inventory（2026-07-20）

ステータス: **観察／比較中**。`re_ui`を一括採用する決定ではなく、固定sourceの各moduleをMotoliiの既存fixtureへ対応付けるRR-0追補である。候補分類は個別裁定前の仮置きであり、依存追加、vendoring、移植、発注を許可しない。

> 改訂(2026-07-20): [Fable反対側レビュー](https://github.com/oshikaidesu/Motolii/pull/226#issuecomment-5019498741)(ACCEPT WITH CHANGES)のP1×1・P2×4を反映し、§4.1(file-level粒化)と§5.1(re_ui証拠なしID帯)を追加した。レビューは両anchorの全source再取得と`wc -l`機械再集計で行われ、`command/*`行数と`egui_ext`包含以外の全数値が一致した。

## 1. 先に固定するMotolii側の目的

調査順は[Rerun学習・転移計画 §9](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)に従う。

| 項目 | 現行事実 |
|---|---|
| Motolii authority | M3仕様、[UI境界規律](2026-07-14-m3-ui-boundary-prevention.md)、[UI視覚言語](../ui-visual-language.md)、[UI参照地図](../ui-reference-map.md) |
| 現行code | mainの`motolii-ui`はSlintのlink確認だけを持つ最小骨格。egui component、shell、fixtureはまだmainへ統合されていない |
| 現行prototype | `codex/m3-mock-components`(集計commit `7572376`)のReact component mapは、primitive 7、**pattern 13**(reference-candidate 6+design-candidate 7)、surface 12、screen 4の安定IDを持つ。branch移動で数は動くため、照合時は必ずcommitを添えて数え直す。React/DOM/CSS境界は製品APIではない |
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

検証手段の非対称(2026-07-20記録): archive SHA-256とicon実数(103 SVG+1 PNG)・snapshot実数(39 PNG)の根拠は**ローカルarchive集計**であり、codeloadが遮断された環境からは再確認できない。その場合の代替はtree/raw経由のfile単位取得+`wc -l`照合で、反対側レビューはこの方法により全59/51 fileの行数一致を確認した(icons.rsのコード登録は100 SVG+1 PNGで、on-disk数と矛盾しない)。

## 3. crate全体を依存してはいけない理由

main監査commitの[`Cargo.toml`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui/Cargo.toml)と`src/`を機械集計した。

- production Rust sourceは59 files、15,196行。安定release 0.34.1は13,846行で、短期間に1,350行増えている
- runtimeに8個の必須内部`re_*` crate、任意の`re_analytics`、build時の`re_build_tools`を要求する。ただし最重量の`re_entity_db`/`re_log_types`はCargo.tomlコメント上**syntax-highlighting専用**で、上流に`TODO(emilk): move InstancePath`がある — 上流がこれを移した時点で依存重量の前提が変わるため、**一括DEPEND棄却候補の再評価トリガー**として記録する
- `eframe(wgpu)`はハード依存(`default-features = false, features = ["wgpu"]`)。外部closureも`egui_commonmark`/`egui_extras`/`jiff`/`ron`/`notify`/`wayland-client`/`wayland-protocols`まで届く
- Wayland、hot reload、Rerun command、Entity/Time型まで同じcrateに含む
- 非コード資産はInter Medium、dark/light theme、color table、103 SVG + 1 PNG icon、logo 2点
- 独立test sourceは703行、snapshot PNGは39点。component実装と試験資産が同居する
- 0.34.1からmain監査commitまでに、button、command palette、context、design token、notification、text edit、time drag、`UiExt`が変更され、`fuzzy.rs`・`command/`分割・`egui_ext/kb_shortcut_ext.rs`・`egui_ext/layout_job_ext.rs`が新設された(両tree差分の機械確認)

よって`re_ui` crate全体の`DEPEND`候補はここで**棄却候補**とする。これは個別moduleの`VENDOR/PORT/PATTERN`比較を棄却するものではない。

## 4. module inventory

行数はmain監査commitのRust source。`候補`は観察であって裁定ではない。

| module群（行数） | sourceが実際に持つ責務 | Motolii側の既存入口 | 結合・不足 | 候補 |
|---|---|---|---|---|
| `button` 314、`combo_item` 247、`menu` 38、`section_collapsing_header` 89 | size/variant、icon+text、combo row、compact menu、折畳み見出し | `primitive.action-button`、`primitive.icon-button`、`pattern.panel-header`、U0e | token/icon/egui型へ直結。Motoliiのstate matrixとCJKは未証明 | 限定`PORT`比較 |
| `list_item/*` 2,488 | label/property/custom content、indent、selection、hover button、navigation、collapsing row | Browser hierarchy、Inspector parameter row、Inbox、Settings、U4a/U4d/U6 | まとまった実戦資産だが大きく、Rerun tokenと`UiExt`へ結合。大量行virtualization自体は証明しない | `VENDOR`対`PORT`比較の最優先。table系surfaceは外部leaf crate [`egui_table`](https://github.com/rerun-io/egui_table)(rerun-io、[references.md](../references.md)登録済み)の`DEPEND`も比較集合へ入れる |
| `re_form/*` 328 | field strip、分数幅、selectable/toggle、統一frame | `pattern.parameter-row`、`surface.inspector`、U4a | NodeDesc→widget→commandや全型fallbackは持たない | layoutを`PORT`、生成契約はMotolii所有 |
| `filter_widget` 864、`fuzzy` 281 | query state、path match範囲、highlight、`nucleo-matcher`によるfuzzy match | `pattern.discovery-browser-shell`、Browser 3面、U4d/U6 | 0.34.1に`fuzzy.rs`は無くmain側が先行。検索意味、tag、selection commitはMotolii固有 | `nucleo-matcher`直接`DEPEND`対wrapper `PORT`比較 |
| `egui_ext/card_layout` 238 | cardごとの寸法、wrap配置、hover/drag下のinteraction rect | Browser thumbnail/list toggle、candidate shelf、U4d/U6 | virtualization、elide、可変thumbnail設定は別問題 | geometryを`PORT`比較 |
| `drag_and_drop` 367 | 階層listのbefore/inside/after drop zoneとindicator geometry | Browser tag drop、hierarchy、panel layout、U1e/U4d/U6 | MotoliiのCommit Intent、Undo、別parent規則は持たない | geometry `PORT`、command意味 `REJECT` |
| `text_edit` 168 | suggestion popupつきautocomplete | Browser search、parameter text、G0-6 IME | IME preedit/CJK、shortcut抑制、commit/cancelは証明しない | `PATTERN`。IME fixture後だけ`PORT`比較 |
| `command_palette` 457、`command/*` 1,570 | generic providerに見えるpaletteに加え、Rerun recording/server/table/UI commandとshortcutを同梱 | 現行React mapにcommand palette要求なし | command意味、time cursor、recording、serverがRerun固有。mainで大幅更新中 | paletteは**未裁定**(Motolii problem不成立のため照合ticket未作成。5分類は要求成立時に裁定)、commandは`REJECT` |
| `modal` 421、`alert` 168 | modal stack、area、button row、alert variants。`modal.rs`は**egui 0.35標準`egui::Modal`のwrapper**(L95・L209-214で`egui::Modal::new`を使用) | plugin recovery、Settings、diagnostic dialog、U2c | focus trap、IME、OS accessibilityは未証明 | 最小案は**egui標準`egui::Modal`直用+Motolii style**。その上でpresentation `PORT`対限定`VENDOR`比較 |
| `notifications` 679 | toast + history、unread level、details field、dismiss/never-show | `pattern.diagnostic-feedback`、`pattern.status-brief`、plugin recovery、U2c | `re_error`/`re_log`を受ける。Motolii typed diagnostic envelopeと抑制寿命は別 | 責任分割`PATTERN`、表示shell `PORT`比較 |
| `loading_indicator` 119 | available rectからradiusを決めるspinner描画 | readiness/rendering/stale、U3f | generation/stale/mailboxを証明せず、理由文字列だけ | `PATTERN`。単独採用しない |
| `design_tokens` 1,188、`color_table` 234、`hot_reload_design_tokens` 166、`context_ext` 212 | RON theme読込、dark/light、style適用、font登録、hot reload | G0-6、UI視覚言語 | Rerun値をDTCG正本へできない。mainと0.34.1でtheme値も変化 | pipelineは`PATTERN`、token値/serdeは`REJECT` |
| `icons` 264、`icon_text` 100 | compile-time SVG/PNG registry、image loader、icon+shortcut text | Motolii icon grid、transport、Browser、panel actions | 103 SVGにRerun固有語彙とlogoを含む。MotoliiはLucide候補を別審査中 | loader `PATTERN`、asset一括`REJECT` |
| `ui_layout` 223、`egui_ext/*` 450(card_layout 238を除く残り8 file) | left/right/center layout、group、boxed widget、shortcut/layout job/response/widget text helpers | shared primitive/pattern全般 | 多くはegui convenience。公開component境界へすべきでない | 必要fileだけ`PORT`比較 |
| `ui_ext` 1,433 | link、checkbox/radio、list item、markdown、copy、context menu等の巨大extension trait | 複数surface | unrelated helperとanalytics/error/hyperlinkが集中し、追従差分も大きい | 一括`VENDOR/DEPEND`は`REJECT`、関数単位で再照合 |
| `relative_time_range` 415、`time_drag_value` 280、`time` 61 | sequence/timestamp/durationの入力と表示 | Timeline、transport、parameter scrub | Rerun `TimeType`/`TimeInt`/`Timestamp`へ直結。Motolii `RationalTime`、audio clock、Undoと異なる | widget責任`PATTERN`、型/実装`REJECT` |
| `syntax_highlighting` 511 | EntityPath、InstancePath、time、URL等の色分け | 現行M3要求なし | `re_entity_db`/`re_log_types`を直接要求 | `REJECT` |
| `help` 203、`markdown_utils` 39 | OS別shortcut/help表示、markdown用input表現 | help surfaceは現行安定IDに無い | product動線が先に必要 | **未裁定**(照合ticket未作成) |
| `testing` 52 + integration tests 703 + snapshots 39 | theme付き`egui_kittest` harness、UI/3D snapshot option、component state fixture | U0e/G0-6、React reference比較 | snapshotだけではIME、性能、意味、accessibilityを証明しない | harness責任とstate matrixを`PATTERN` |
| `wayland` 218 + window decoration helpers | Wayland decoration交渉、OS別custom frame既定 | U1 shell | eframe/lifecycle採択後のOS実機問題。制作動線とは独立 | `PATTERN`、問題再現前の移植は`REJECT` |

表の対象外はcrate root `lib.rs`(311行。re-export、icon size定数、accessor等)のみで、これで59 file / 15,196行が検算一致する。

### 4.1 優先クラスタのfile-level粒化(2026-07-20追補)

§5のA〜Eを裁定単位までさらに割る。行数はmain監査commitの実測(`wc -l`)。「確認質問」に答えることがそのfileを読む完了条件であり、答えを本文書へ書き足すのではなく、対応するMotolii fixture/ticket側へ回収する。

| クラスタ | file(行数) | 確認質問 |
|---|---|---|
| A 検索 | `filter_widget.rs`(864) | query stateの寿命はどこで切れるか。hierarchy pathへのmatch範囲指定とhighlightを、Motoliiのelide決定(P52)と両立できるか |
| A 検索 | `fuzzy.rs`(281) | `nucleo-matcher`の呼び方はwrapper層でどこまで薄いか。直接`DEPEND`した場合に失うものは何か(0.34.1には存在しないmain先行file) |
| A 結果表示 | `egui_ext/card_layout.rs`(238) | 可変寸法cardのwrap配置とhover/drag interaction rectの計算だけを、Browser共通thumbnail寸法決定(P50/P51)へ写せるか |
| A 試験 | `tests/filter_widget_test.rs`(35) | 検索fixtureの最小形。CJK query・IMEは含まれない(Motolii側で必須追加) |
| B dense row | `list_item/list_item.rs`(682)、`scope.rs`(312) | 行の高さ・interaction・selectionの所有はどこか。`scope`のtoken伝播はDesignTokens無しで成立するか |
| B dense row | `label_content.rs`(286)、`property_content.rs`(370)、`custom_content.rs`(212) | label/property/customの3内容型の境界。Motolii `pattern.parameter-row`のwide/narrowへ写像できるか |
| B dense row | `item_buttons.rs`(264)、`navigation.rs`(165)、`button_content.rs`(74)、`debug_content.rs`(35)、`mod.rs`(88) | hover時button・keyboard navigationの責任分離。Rerun `UiExt`への依存点の列挙 |
| B form | `re_form/form_strip.rs`(132)、`selectable.rs`(122)、`fields.rs`(67)、`mod.rs`(7) | 分数幅field stripの実装量。`NodeDesc`自動生成panelの下地に足りない部分(全型fallback・command接続)の確認 |
| B 試験 | `tests/list_item_tests.rs`(242) | state matrix(selection/hover/indent)の網羅粒度。Motolii側へ必要な追加軸(CJK・pseudo-locale・dark/light) |
| C DnD | `drag_and_drop.rs`(367) | `ItemKind`/`find_drop_target`/`DropTarget`のgeometry契約(before/inside/after、root制限)。Motolii Commit Intent/Undoへ**繋がない**ままgeometryだけ検証できるか |
| D 診断 | `notifications.rs`(679) | toast/history/unread level/NeverShowAgainの状態機械。U2c typed envelopeへの写像で捨てる部分(`re_error`/`re_log`結合) |
| D 診断 | `alert.rs`(168)、`modal.rs`(421) | alert variantの意味色とMotolii semantic色の対応。egui標準`egui::Modal`直用で足りない差分は何か |
| D 試験 | `tests/notification_test.rs`(84)、`modal_tests.rs`(66) | 診断fixtureの合否条件。dismiss寿命・focusの検査有無 |
| E 試験基盤 | `testing.rs`(52) | theme付きharnessの最小コスト。`egui_kittest` optional依存の切り方 |
| E 試験基盤 | `tests/command_palette_test.rs`(213)、`help_ui_test.rs`(63) | snapshot対象の選び方(高頻度更新moduleほど試験が厚いか)の観察のみ。palette/help自体は未裁定 |
| 参考(REJECT境界の実量) | `command/recording_command.rs`(541)、`ui_command.rs`(481)、`mod.rs`(262)、`redap_server_command.rs`(171)、`table_command.rs`(98)、`environment.rs`(17) | 読む必要はない。`command/*` 1,570行の大半がRerun domain commandである事実だけを、palette「未裁定」とcommand`REJECT`の根拠として保持する |

粒化の停止線: この表は読解の完了条件であり、file単位の発注許可ではない。1裁定は§8のとおり1クラスタずつ、5分類のいずれか一つで閉じる。

### 4.2 獲得価値の粒化 — 実証済みで再発明を節約できる設計判断(2026-07-20追補)

§3〜§4.1は結合・リスク側(持ち込まない理由)を粒化した。本節はその対で、**採用可否と独立に、読むだけでMotoliiが再発明せずに済む「解決済みの設計判断」**を粒化する。いずれも一次sourceの該当行で確認済みの実装事実であり、「Rerunで動いている」以上の一般性は主張しない(反例未探索。仮説と整合する実証例として扱う)。

| 解決済みの問題 | 実装事実(file、main監査commit) | Motoliiが節約する再発明 | 受け皿 |
|---|---|---|---|
| **immediate modeでの列揃え**: 1 passでは右列の揃え幅が決められない | `list_item/scope.rs` — frame nで各行が幅統計を蓄積→egui temporary memoryへ保存→frame n+1のlayoutが使う2 frame方式。ASCII図つきで文書化 | Inspectorのparameter row列揃えをwidget treeの再走査なしで成立させる方式の探索。M3で必ず踏む問題 | B(`pattern.parameter-row`)、U4a |
| **hover buttonのclick競合**: hover判定でbuttonを出すと、click瞬間に消える | `item_buttons.rs` L42のコメント「`.hovered()`は使えない — clickの瞬間にbuttonが消える」+回避実装。既定は「hover時または選択時のみ表示」 | 高密度listの行内action(Browserの表示切替・削除等)で同じ地雷を踏んでから直す工数 | B、Browser hierarchy |
| **2列property rowの標準形**: label+icon左列/編集可能右列、間隔管理、read-onlyヘルパー | `property_content.rs` L19-22, L37, L92, L106。「2〜3列を意味を保って畳む方法はない」という設計判断のコメントも残る | parameter rowの基本骨格と、狭幅時に「畳まない」判断の先例 | B、U4a |
| **検索hitの全域highlight**: 階層pathに対するmatch範囲収集→`WidgetText`装飾の一貫pipeline | `filter_widget.rs` L288(path match+highlight range)、L384(early-outせず全match収集)、L573(range列→highlight済みWidgetText) | 検索一致表示をelide決定(P52)と両立させるためのrange→装飾変換の設計 | A、Browser検索 |
| **theme編集の即時反映**: rebuildなしのtoken iteration | `hot_reload_design_tokens.rs` — cfg gate付き`notify` watcherでRON themeをlive reload(L52-64)。本番buildには含まれない | G0-6Hの人間審判とU0e token調整のiteration速度。「値を変えて見る」のたびのrebuild | E、U0e/G0-6H |
| **階層DnDのdrop位置幾何**: before/inside/afterの判定とindicator描画位置 | `drag_and_drop.rs` — `DropTarget`がindicator span_x/position_y/親ID/挿入indexを一括で返す(L39-53)。root containerへの制限も型で表現(L8) | drop indicatorの座標計算とedge case(root直下・末尾挿入)の手探り。geometryだけ借りてcommand意味は自前、の分離が可能な形 | C、Browser/panel DnD |
| **autocomplete候補のindent扱い**: 表示は字下げ、filter/確定時は除去 | `text_edit.rs` L14-16「Leading whitespaceはdisplay-only indentation」 | 候補listの視覚階層と入力値の分離という細部仕様。IME fixture設計時の観点にもなる | A、Browser検索入力 |
| **通知の二面性**: 即時toastと履歴を同一状態から投影、未読level・NeverShowAgain | `notifications.rs` — `is_unread`/`unread_notification_level`(L123-291)、`NeverShowAgain`(L258) | 「toastを見逃したら消える」問題の解法形。U2c envelopeの投影先設計の下敷き | D、`pattern.diagnostic-feedback` |
| **card格子のhover/drag描画** | `egui_ext/card_layout.rs` — `hover_fill`(L70-74)とdrag中interaction rectの扱い | Browser thumbnail格子のhover/drag視覚応答の初期値 | A、`pattern.candidate-shelf` |
| **theme付きsnapshot harnessの最小形** | `testing.rs` 52行で`egui_kittest`にtheme文脈を注入。試験厚は更新頻度の高いmoduleに寄る(§4.1 E) | component snapshot基盤を52行規模から始められるという規模感の証拠 | E、U0e/G0-4 |

読み方: この表の「節約」は**設計判断の探索費**であり、実装費のゼロ化ではない。回収先は各行の受け皿ticket/fixtureで、回収時に§6(CJK/IME)と§4.1の確認質問を併走させる。この表を根拠に分類(`DEPEND/VENDOR/PORT/PATTERN/REJECT`)を飛ばして発注することはできない。

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

### 5.1 `re_ui`証拠が存在しないReact ID(自前実装帯)

対応付けの完全性のため、**re_ui側に対応物が無い**安定IDを明示する(集計commit `7572376`)。ここへ後からRerun対応を発明しない — 将来「Rerun起点のMotolii要件」が逆流していないかの検査線として使う。

- 編集意味そのもの: `primitive.keyframe-marker`、`primitive.parameter-scrub`、`primitive.automation-mark`、`primitive.color-swatch`、`primitive.semantic-badge`、`surface.easing-graph`、`surface.curve-shelf`、`surface.color-book`、`surface.depth-rail` — Rerunにkeyframe/easing/色設計の編集domainが無い。自前実装帯
- 別crateに証拠がある(re_uiではない): `surface.timeline`(`re_time_panel`/`re_time_ruler`、RR-3)、`surface.stage-viewport`(`re_viewport`/gpu_bridge、RR-2/RR-5)、`screen.graph-view-candidate`/`surface.graph-view`(`re_view_graph`、未照合) — 該当RRレーンで扱い、本文書の裁定単位に混ぜない
- shell系: `pattern.resizable-panel-layout`(`egui_tiles`+Blueprint投影、RR-2)、`pattern.transport`(time widgetは型`REJECT`済み。transport UIの席は自前)

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
