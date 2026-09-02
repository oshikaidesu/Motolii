# 「普通」へ一晩で到達する実行計画

- appetite: **10時間**。T+7:00で新規実装freeze、残り3時間は実窓検収と修正だけ。
- target condition: 白紙から素材を入れ、編集し、音付きで再生し、保存・再読込・書き出しまで説明なしで通る。
- 方法: [Shape Up](https://basecamp.com/shapeup/1.2-chapter-03)の固定時間/可変scope、
  [Spike](https://www.agilealliance.org/wp-content/uploads/2017/08/AgileExtension_V2-Member-Copy.pdf)と
  [Improvement Kata](https://www.lean.org/lexicon-terms/kata/)の短い学習cycle、WIP制限を使う。

## 2026-09-02 画面所有委譲後の自走計画

- 許可: 利用者からMotolii実窓の操作を委譲された。外部送信・削除・install・system設定変更は
  この許可に含めない
- 目標: **普通にAEの操作ができること。AEの内部都合を覚えさせないこと**
- 完成像: 白紙から素材／文字／形を置き、選択・時間・合成を編集し、音付きで確認し、
  Undo、保存、再読込、音付き書き出しまで同じ作品意味で2巡できる
- UI停止線: 新しいmode、専用panel、事前変換、precomp、node graph、機能専用の確定操作を作らない。
  新しい名詞は既存Browser群、値はInspector、時間はTimeline／Ease、結果はStage／Outputへ出す
- 技術・検証停止線: 新規コードとtest／実窓検収の前に[憲法](../motolii/AGENTS.md)の順で
  参考文献・既存実装・上流oracle・外部の定規を検索し、`REUSE / REMAP / REDUCE`を処分する。
  検索receiptの無いcomponent／contract／閾値／期待値は施工・採用しない

### 状態機械

| phase | 仕事 | 出口 | code |
|---|---|---|---|
| **A0 AUTHORITY** | 現行code・test・台帳を再照合。古いgapを再実装候補から外す | 各rowが`現存 / 実窓待ち / 本当に欠落`のどれか | 禁止 |
| **A1 FIRST RUN** | `最初の一本`を白紙から実窓で通し、最初の停止点を採る | action trace、停止点1件、同じ時刻のStage／Output | 修正禁止 |
| **A2 ORDINARY EDIT** | 選択、移動、trim、split、duplicate、key追加／移動、ease、Undo | 編集後の関係が壊れない不変量＋実窓操作 | 最初の停止点だけ |
| **A3 COMPOSITE** | Group、blend、effect、matte、maskを同じ層文法で通す | precomp／flatten前置きなしに画が変わる | 同上 |
| **A4 FINISH** | 音付き再生、Save/Open、Output、音付きExport、Cancel | 保存前後とPreview／Exportの作品意味一致 | 同上 |
| **A5 TWO PASSES** | clean launchから同じ台本を2巡 | 2巡とも結果・Undo一致、新規停止点0 | 新規実装禁止 |
| **DONE** | gap／ordinary-upstream／説明書を実測へ更新 | 未観測をPASSにせず、残件は名前付き | docs/dataのみ |

phaseは順番にしか進まない。Cargoや静的grepで次phaseへ進めない。実窓の停止点が出たら、
その1件だけを`参考文献 → upstream → 現行owner → exact gap → oracle`へcompileする。

### A0で既に再確認できた在庫

`ordinary-upstream.tsv`の文言をそのまま仕事にしない。2026-09-02の現行code検索では次が実在する。

| 台帳上の古い見え方 | 現行の実在 | A0処分 |
|---|---|---|
| 層を複数選べない | `session::Selection(Vec<LayerId>)` | 実窓で加除選択を確認するまで`実窓待ち` |
| key時刻を動かせない | `TimelineWidget::DragMode::Key`→`MoveKey` | 同上 |
| 層移動でkeyが置き去り | `keyframe_shift_intents` | 同上 |
| Groupの通常UIが無い | `Cmd+G`／`Cmd+Shift+G`→`group_layers/ungroup_layers` | 同上 |
| Matte source/mode UIが無い | Inspector `LayerChoiceRow`／`MatteModeRow` | 同上 |
| Saveがatomicでない | `Document::save_atomic` | Save/Open実窓とdirty guardを分けて再判定 |
| 二つ目Vismの共通経路が未証明 | `vism_catalog::two_manifest_only_visms_share_the_catalog_and_effect_stack` | focused PASS |

A0時点で静的に残っていたMask未消費、音声mux、Preview／Export層準備重複は、A3/A4 preflightで
既存部品へ接続・縮約済み。現在の正本は下の各receiptと`ordinary-upstream.tsv`であり、
実窓未検収を実装欠落へ戻して数えない。

### 実窓で通す二つの台本

### 台本の外部定規

「期待どおり」を会話やMotoliiの現在挙動から作らない。各一手は次の外部定規へ写す。

| 台本の一手 | 外部の定規 | Motoliiへの写像 |
|---|---|---|
| pointerで掴む／離す／取消 | [W3C Pointer Events 3](https://www.w3.org/TR/pointerevents3/)と[WPT](https://github.com/web-platform-tests/wpt/tree/master/pointerevents)、UIKit recognizer、Android touch slop | capture、cancel、必ず終端、slopの**意味**を採る。固定3pxは規格値ではない |
| layer単一／複数選択、Stage移動 | [Adobe: Selecting and arranging layers](https://helpx.adobe.com/after-effects/desktop/work-with-layers/select-and-arrange-layers/selecting-arranging-layers.html)、[Apple Motion: Select layers and groups](https://support.apple.com/guide/motion/select-layers-and-groups-motn466036e1/mac) | clickで単一、Commandで非連続追加、Shiftで範囲。選択層を掴むと同じ相対配置で動く |
| layer時間移動／端trim | [Adobe: Selecting and arranging layers](https://helpx.adobe.com/after-effects/desktop/work-with-layers/select-and-arrange-layers/selecting-arranging-layers.html) | duration bar dragはlayerと全keyを動かす。端dragはIn/Outだけを変え、key時刻を動かさない |
| duplicate／split | 同Adobe公式のCopy or duplicate／Split a layer | duplicateはeffects、keys、masksを保つ。split両側は元keyを元時刻で保持し、matte順も保つ |
| key選択／移動／copy | [Adobe: Editing, moving, and copying keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/editing-moving-copying-keyframes.html) | 複数keyを選んで相対間隔のまま動かす。同型propertyへcopyできる |
| Group | [Apple Motion: groups](https://support.apple.com/en-my/guide/motion/motnc775e403/mac)と[Cavalry Group](https://cavalry.studio/docs/nodes/shapes/group/) | 子を同じ木に保ち、Group操作が子へ効く。AE precompの別timelineを定規にしない |
| Blend | [W3C Compositing and Blending 1](https://www.w3.org/TR/compositing-1/)と`reference/compositing-coverage.tsv` | mode名・式・Porter-Duff意味を借りる。Motolii独自の見た目を正解にしない |
| Matte／Mask | [Adobe Track Mattes](https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-track-mattes-and-traveling-mattes/track-mattes-and-traveling-mattes.html)と[Adobe Masks](https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-alpha-channels-and-masks/alpha-channels-masks-mattes.html) | alpha／luma／反転のcoverage、path順、変形可能なsourceを採る。precompose推奨は採らない |
| Effect順 | [Adobe render order](https://helpx.adobe.com/mena_en/after-effects/desktop/work-with-compositions/precomposing-and-nesting/precomposing-nesting-pre-rendering.html)と[Autograph Modifiers](https://help.maxon.net/ag/en-us/Content/html/Generators_modifiers.html) | 上から下の順序が画へ出る。Transformだけの逆順例外はMotoliiへ移さない |
| 値の駆動 | [Apple Motion Parameter Behaviors](https://support.apple.com/en-ie/guide/motion/motn49eb56eb/mac) | 原因をkeyへ焼かず対象parameterへ付ける。最低限AE台本の後で扱う |
| Save/Open、音声clock、audio mux、Export cancel | Rust `rename`、CPAL stream clock、FFmpeg explicit `-map`／raw f32le／`shortest`、現行Rerun RRD encoder/decoder | 定規取得済み。下のA4 receiptへ検索範囲・写像・oracleを固定 |

この表に無い操作は、その場で期待値を作らない。A0へ戻り、外部定規を追加してから再開する。

#### 台本1 — 最初の一本

[最初の一本](wiki/start.md)をそのまま使う。Import → Stage移動 → Positionの◇ → 時刻移動 →
Stage移動 → Ease → Playback → Export。説明書と窓が違った最初の一行で止める。

#### 台本2 — 最低限AE

1. 白紙を作り、画像・動画・音声を入れる
2. TextとRectangleをCreateから置く
3. 複数層を選び、移動・trim・split・duplicate・Group／Ungroup
4. Position／Scale／Rotation／Opacityへkeyを置き、key時刻を動かしEaseを付ける
5. Blend、Vism Effect、Matte、Maskを適用し、事前precompなしで結果を見る
6. 音付きで再生し、Undo／Redoを往復する
7. Saveして開き直し、同じ時刻・同じ選択対象・同じ画を確認する
8. 音付きでExportし、Outputと一致することを確認する

### 各修正の検索receipt

codeまたは検証へ触るsliceは、計画または台帳へ次の6項目を先に残す。

| field | 内容 |
|---|---|
| `NEED` | 台本のどの一手で何が止まったか |
| `SEARCHED` | 現行repo／既決、pin済み依存・上流source、公式一次資料・製品先例 |
| `DISPOSITION` | `REUSE / REMAP / REDUCE`。該当なしなら検索範囲 |
| `OWNER/ROUTE` | Document／Intent／StoreView／Rerun／既存UI componentのどこへ接続するか |
| `RULER` | 上流oracle、公式規格、公式manualの挙動、既存製品の収束した合否。採れない時は検索範囲 |
| `ORACLE` | `RULER`をMotoliiへ写す最小adapter。独自閾値・都合のよいgoldenを発明しない |

### 自走中の報告規則

- 60分以上無言にしない。phase、最初の停止点、検索結果、実窓変化だけを短く報告する
- compile green、test green、logだけで「普通に使える」と言わない。test自体の定規も出典を問う
- 新しいUI語彙を発明しそうになったら停止し、後発競合比較と上流資料へ戻る
- 現在のdirty worktreeは共有成果。名指しした自分の差分以外を戻さない
- A5終了までcommit／pushしない。外部への送信は別許可が必要

### A1実行記録(2026-09-02、Computer Use)

RULERは上の外部定規表。Motolii独自の「動いたように見える」では判定していない。

| 一手 | 結果 |
|---|---|
| File > New | 白紙とCamera行へ遷移 |
| File > Import > `logo.png` | Mediaに1 card。card clickで256×256の層が出力枠中央へ立つ |
| Stage drag | Position `832,412 → 1669,412`。Positionの◇まではkey 0、押すと現在時刻に1 key |
| Timeline seek + Stage drag | frame 20(0.67秒)で2本目のkey。中央`832,411`へ戻る |
| Ease | 最初のkeyを選びElasticを即時適用。曲線と`Elastic`表示が同時に変化 |
| Space playback | playheadとStageが進み、Spaceで停止 |
| Export | Outputにframe進捗と保存先。完了後FFprobeでH.264 / 1920×1080 / 30fps / 60秒 / 1800frames |
| 出力frame | 9秒をPNGへ復号し、中央の赤いロゴを確認 |

外部定規から見た残件:

1. **Output completion redraw** — 完了直後にOutputが黒くなり、Stage→Outputと開き直すと
   同じ赤い像と`Wrote <path>`へ戻った。出力fileは正しいため、作品評価でなく完了wake／再投影の穴。
2. 最初の300px Stage dragは選択だけ、続く140〜170px dragは動いた。Computer Useのevent列と
   製品の長距離dragを分離できていないため欠陥認定しない。W3C/WPT対応harnessで再検証する。
3. `MOTOLII_TESTDATA`付きprocessとは別にComputer Useがapp bundleを開いたため、環境付きprocessを
   未使用のまま二重起動した。未使用processだけ終了。製品経路はFile Importで検証した。

A1判定: **PASS with named A4 defect**。A2へ進む。

### A2-KEY検索receipt — Command shortcutが窓で無反応

| field | 内容 |
|---|---|
| `NEED` | TimelineでRectangleを選択後、`Cmd+D`で行が増えない。`Cmd+A`／`Cmd+G`も無反応。Spaceは届く |
| `SEARCHED` | `ui/app.rs` root key handler、`ui/keymap.rs`、pin済みBlitz `convert_events.rs`／event driver、`keyboard-types 0.7`、Dioxus HTML keyboard data、W3C UI Events／UIEvents-Key |
| `DISPOSITION` | **REMAP**。新しいshortcut/focus機構は作らない |
| `OWNER/ROUTE` | BlitzはmacOS winit `meta_key()`を`keyboard_types::Modifiers::SUPER`へ変換。Motoliiのprimary判定だけが`.meta()`=`META`を見て落としている。`keymap`の一関数で`META | SUPER`をprimaryへ写し、root handlerが使う |
| `RULER` | [W3C UIEvents-Key](https://www.w3.org/TR/uievents-key/)はApple Commandを`Meta`と規定。pin済みBlitzの実契約は`SUPER`。両方を受ける |
| `ORACLE` | `keyboard-types`の実bitで`META`／`SUPER`がprimaryになるfocused unit。実窓で選択済みRectangleへ`Cmd+D`、`Cmd+A→Cmd+G`、`Cmd+Z`を再実行 |

### A2実行記録(2026-09-02、Computer Use)

最初の再検証は05:05の古いapp bundleを操作していたため不採用。08:55のbuildをbundleへ入れ、
`dev.motolii.app`を一意に指定した結果だけを証拠にした。

| 一手 | 実窓とownerの結果 |
|---|---|
| `Cmd+D` | 選択済み層が16行から17行へ増加。`Modifiers(SUPER) → Intent::Duplicate → Document`を同じrunで記録 |
| Create | File > New後、Createの同じRectangle cardから2層。Cameraを含め3行 |
| `Cmd+A → Cmd+G` | Group行と2つの子行が同じTimelineへ現れ、InspectorもGroupを投影。別timeline／precompなし |
| `Cmd+Z` | `history moved=true`の一手でCamera + Rectangle 2行へ復帰 |

RULERはW3C UIEvents-Key、pin済みBlitzのmodifier変換、Apple Motion／CavalryのGroup。
focused unitは`META`と`SUPER`のadapterだけを検査し、合否は上の実窓とDocument writeで決めた。

A2判定: **PASS**。A3へ進む。trim／split／key移動／EaseはA1の実窓証拠と既存oracleを維持し、
A5の通し台本で再度同じ作品上にまとめる。

### A3-EFFECT-MATTE検索receipt — Effect付きtargetがMatteで全画面化

| field | 内容 |
|---|---|
| `NEED` | `motolii.gradient`を掛けたlogoへRectangleのAlpha Matteを選ぶと、局所像でなくGradientがcomp全面へ広がった。Effect無しの同じlogoはRectangle範囲へ正しく切れた |
| `SEARCHED` | `engine/render.rs`の2つのlayer組立経路、`compositor/render_effects.rs`のlocal texture effect、`sequential.rs::matte_layer`、既存`matte_relationship`／`vism_catalog`、Adobe公式render order／Track Mattes |
| `DISPOSITION` | **REMAP**。Effect別shader、Matte別UI、precompを増やさず、既存layer pipelineの順序を直す |
| `OWNER/ROUTE` | 現状は`matte_layer`がtargetをcomp大textureへした後に`LayerWithPasses`を実行するため、0-input Effectが透明域を塗る。`target local effect → transform → matte coverage → blend`をEngineの一経路にする |
| `RULER` | Adobe公式はraster layerをmask→effects→transformの順、Effectは一覧の上から下とする。Track Matte例は「pattern Effectを持つfill layerがMatte形状内に見える」。Matte sourceのalpha／lumaと変形も結果へ使う |
| `ORACLE` | 0-input Gradient付きtarget + 半面Alpha Matteで、Matte外alpha=0、内側だけEffect色。Effect無しMatteの既存oracleも維持し、同じ作品を実窓で再試験 |

### A3実行記録(途中、2026-09-02)

| 一手 | 外部定規に対する結果 |
|---|---|
| Blend | 白Rectangle上の赤logoをScreenへ変更。中心`255,255,255`、白の無い縁`235,47,35`、外側`19,19,19`でW3C Screen式と一致 |
| Effect | 同じlogoへEffects cardから`motolii.gradient`。Inspectorの同じEffect stackへ現れ、Stageの局所像がgradientへ変化 |
| Alpha Matte単体 | Rectangle sourceで96px logoが約74pxのsource範囲へ切られ、source自身は通常出力から除外 |
| Effect + Matte修正前 | Gradientがcomp全面へ拡張。自動oracleでもMatte外alpha=`255`のRED |
| Effect + Matte修正後 | `target_effect_stays_inside_the_alpha_matte`、従来Matte単体、`matte_source_effect_contributes_to_coverage`の3本がPASS |
| Mask render | 左半面Addで外alpha=0、Add外形→Subtract内形で穴alpha=0。既存Path／coverage／Matte Vismだけを接続 |
| Mask entry | Create棚のMask cardが選択2D層へ中央矩形を`AddMask`一Intentで付け、cardへattached数を返す。任意Path編集は未実装 |

Mask entry追試: Blitz product DOMでTimeline層を選択→Create→Mask card→Document masks `0→1`、
card表示`1 attached`を確認。既存semantic button/disabled試験も4 cardsへ更新して双方PASS。

修正後の実窓再試験中、OS Import sheetへ操作を送ったComputer Use helperが終了待ち(`UE`)となり、
native pipeを再開できなくなった。製品processを閉じてもhelperだけが残るため、A3は**未完了**。
静的GREENを実窓PASSへ昇格せず、helper復帰後に同じRectangle + logo + Gradient + Alpha Matteを再実行する。
同じ再試験でMask card→Stage変化→Undoも確認する。

再起動後の実窓再試験(2026-09-02): Rectangle + logoへGradient + Alpha Matteを適用しても
comp全面へ広がらず、代表画素は内`125,129,154`／外`19,19,19`。Mask単体は赤像を中央60%へ切り、
内`229,54,35`／mask外`20,18,21`、cardは`1 attached`、`Cmd+Z`で0へ復帰。

A3判定: **PASS**。A4へ進む。

### A3-MASK検索receipt — DocumentのMaskが描画で未消費

| field | 内容 |
|---|---|
| `NEED` | `ResolvedLayer.masks`、path評価、opacity／expansion／modeは在るが、Engineのlayer組立が一度も読まず、Maskを持つ作品でも画素alphaが変わらない |
| `SEARCHED` | `doc/store/mask.rs`、`view::resolved_masks`、`engine/mask.rs::fold_masks`、vector coverage、既存`matte.wgsl`、Rerun texture manager、Adobe公式Alpha channels and masks／render order |
| `DISPOSITION` | **REUSE_WRAP**。新しいmask renderer／shader／panelを作らず、既存Path→CoverageとMatte Vismのalpha積をlocal layer textureへ接続する |
| `OWNER/ROUTE` | `Document Mask → StoreView ResolvedMask → fold_masks(layer frame) → cached coverage texture → existing matte Vism → Effect → Transform → Track Matte → Blend`。3Dは明示flatten後だけ同じ2D経路へ入る |
| `RULER` | [Adobe Alpha channels and masks](https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-alpha-channels-and-masks/alpha-channels-masks-mattes.html)はMaskが層に属し、同層内のstack順・Add/Subtract/Intersect/Lighten/Darken/Difference・opacity・invert・expansionでalphaを変えると規定。raster順はMask→Effect→Transform |
| `ORACLE` | 8×8 opaque targetへ左半面Add Maskで、内alpha>0・外alpha=0。Add外形→Subtract内形は穴だけalpha=0。Documentなしの自作goldenでなくAdobe mode定義を既存`fold_masks`へ写す |

### A4-SAVE検索receipt(preflight) — 通常Saveだけatomic helperを迂回

| field | 内容 |
|---|---|
| `NEED` | `Document::save_atomic`はauto-saveだけが使い、File > Saveの`Document::save`は保存先を直接`File::create`して旧projectを先にtruncateする |
| `SEARCHED` | `doc/store/persist.rs`、File menu `put_away`、export snapshot、Rust `std::fs::rename`公式、Rerun RRD encoder／decoder現行経路 |
| `DISPOSITION` | **REUSE**。新しいproject format／journal／backup UIを作らず、通常Saveを既存same-directory temp→renameへ通す |
| `OWNER/ROUTE` | `File Save → Document::save → flattened RRD encode(temp) → rename(destination)`。UIは成功後だけproject pathを更新する既存経路のまま |
| `RULER` | [Rust std::fs::rename](https://doc.rust-lang.org/std/fs/fn.rename.html)は既存destinationを置換する。完成前bytesをdestinationへ書かず、同じdirectoryのtempだけをrenameする |
| `ORACLE` | 旧project保存後、temp pathをdirectoryで塞いで次Saveを失敗させる。失敗後も旧projectをloadでき、旧layer名が残る |

結果: `Document::save`を既存`save_atomic`へ一本化。失敗保持・通常round-trip・blank projectの3 testがPASS。
dirty guardと実窓Save/Openは未検収なのでA4は未通過。

### A4-AUDIO-MUX検索receipt(preflight) — Export MP4がvideo-only

| field | 内容 |
|---|---|
| `NEED` | Preview用`AudioProgram`は48kHz stereoで作品全層をmixできるが、`Encoder`はrawvideo stdin一入力だけでMP4にaudio streamが無い |
| `SEARCHED` | `AudioProgram::from_view/mix_audio`、canonical 48kHz stereo、`doc/export.rs`、`media/encode.rs`、FFmpeg公式stream selection／raw PCM muxer／shortest |
| `DISPOSITION` | **REUSE_WRAP**。mixer／resampler／codecを作らず、既存AudioProgramのf32leを一時inputにしFFmpegのAAC encoderへ渡す |
| `OWNER/ROUTE` | `Document → AudioProgram → canonical f32le temp → FFmpeg input 1`、videoは既存stdin input 0。`-map 0:v:0 -map 1:a:0 -c:a aac -shortest`で同じMP4へmux |
| `RULER` | [FFmpeg stream selection](https://www.ffmpeg.org/ffmpeg.html)は`-map`で入力streamを明示、[Formats](https://ffmpeg.org/ffmpeg-formats.html)はraw f32leと`shortest`を規定。CPAL device formatはExportの定規にせず、作品のcanonical programを使う |
| `ORACLE` | 1秒toneを持つDocumentを書き出し、ffprobeでvideo+audio stream、48kHz stereo AACを確認。audio streamをf32leへ復号し非無音sampleを確認。音無しDocumentは従来video-onlyのまま |

結果: 440Hz / 1秒の通常audio layerを同じExportへ通し、H.264 video + AAC 48kHz stereoを
ffprobeで確認。audio streamのf32le復号にも非無音sampleが在り、focused test PASS。
さらにUI export snapshotと同じ`Document::save → Document::load → Export`へoracleを延ばし、
再読込後も同じ音声stream／非無音を維持してPASS。
実窓の音付きPreview／Output一致は未検収。

### A4-EXPORT-CANCEL検索receipt(preflight) — Cancelが既存destinationを消す

| field | 内容 |
|---|---|
| `NEED` | FFmpegはdestinationを`-y`で直接開き、Cancel時`remove_partial(destination)`する。同名movieが既に在る場合も旧fileを失う |
| `SEARCHED` | `export_range_with_progress`、`Encoder::Drop`、Output Cancel state machine、Rust same-directory rename、FFmpeg output format inference |
| `DISPOSITION` | **REUSE**。Saveと同じtemp→rename。Cancel専用file管理や復旧UIは作らない |
| `OWNER/ROUTE` | destinationと同じdir・同じextensionのowned tempへEncoderを向け、完走後だけrename。Cancel/errorはguardがtempだけを除去しdestinationへ触らない |
| `RULER` | Rust renameの完成後置換とFFmpegのextensionによるmuxer選択。Cancelは未完成artifactを公開せず、既存完成物を保持する |
| `ORACLE` | destinationへsentinel bytesを置き、開始前Cancelで`Cancelled`を返す。終了後もsentinelが同一で、temp残骸が無い |

結果: Encoderをdestination同dir・同extensionのowned tempへ向け、完走時だけrename。
事前Cancel後も既存destinationのsentinelが同一、音声付きExport oracleも維持して2 test PASS。
実窓Cancelと完了redrawは未検収。

### FrameEvaluation縮約

Mask／Effect／Matteの順をreadbackとpresentable targetへ二重実装していたため、
`render_with_camera_override`も既存`layers_from_resolved`へ接続し、旧`texture_for_layer`一式を削除した。
縮約後にaudio export 2、Mask 2、Matte 3、Vism stack 1の計8 focused testがPASS。
Preview／Output同一時刻の実窓比較は未検収。

2026-09-02収束run: `cargo test --locked --no-fail-fast`は全84 test、exit 0。
内訳にGUI Mask入口、audio Save→Load→mux、Cancel destination保持、Mask/Matte/Vism、
既存`preview_equals_export` 2本、project round-trip、waveformを含む。実窓gateの代用にはしない。

### A4-OUTPUT-REDRAW検索receipt(preflight) — 完了時だけOutputが黒くなる

| field | 内容 |
|---|---|
| `NEED` | Export完了直後のOutputが黒く、Stage→Outputで開き直すと同じ像と`Wrote`が戻る。fileは正しいため完了wake後のprojection更新だけが欠ける |
| `SEARCHED` | `ExportController::complete`／`Poke`／`Host::wake_all`／`Panes::echo`、`OutputPanel`、pin済みDioxus 0.7 `Properties::memoize`とcomponent diff |
| `DISPOSITION` | **REMAP**。新しいpoll threadやOutput state copyを作らず、既存echo generationをOutputStatus propsへ渡す |
| `OWNER/ROUTE` | background state change→Poke→Host echoは既存。controller pointerだけではprops同値になるため、echo値をPanel／StatusBar両projectionのgenerationにする |
| `RULER` | pin済みDioxusはstatic component propsが同値ならmemoizeする。外部状態の変更世代をpropsに含め、同じOutputStatusが再読込されること |
| `ORACLE` | controllerをRunning→Completedへ変えgenerationを進めると、PanelとStatusBarが`Wrote`を返す。最終合否は実窓で完了直後にtab reopen無しで像とWroteが残ること |

結果: echo generationをPanel／StatusBar両方の`OutputStatus` propsへ接続し、pin済みDioxusの
`Properties::memoize`がgeneration変化でfalseになるfocused test PASS。実窓合否は未検収。

### TIMEBASE検索receipt — UIの一部だけ30fps固定

| field | 内容 |
|---|---|
| `NEED` | PlaybackControllerはComposition fpsを読む一方、shortcut split/step/marker/trim、Timeline key/layer drag、Ease適用、Create時刻が30fps定数を持ち、24/25/29.97fps作品で別frameへ書く |
| `SEARCHED` | `CompositionTimebase`、app/browser/ease/timeline_widgetの全30fps使用、Document `Composition.fps`、Adobe公式Frame rate／Timecode and time display |
| `DISPOSITION` | **REUSE_REDUCE**。新しいtimebaseを作らず、Document/PlaybackControllerのfps・frame変換口へ全UIを寄せ30fps定数を削る |
| `OWNER/ROUTE` | `Composition.fps → PlaybackController current/seek frame`。Documentを書いているTimeline/Ease/Createは同じDocumentからfpsを読む |
| `RULER` | [Adobe Frame rate](https://helpx.adobe.com/after-effects/desktop/work-with-footage-items/import-and-interpret-footage-items/importing-interpreting-footage-items.html)はcomposition fpsが1秒のframe数、time ruler区分、keyframeを置ける時刻を決めると規定 |
| `ORACLE` | 24fps compositionでframe step 1、split/trim/marker/Create/Ease/Timeline key moveが1/24秒へ写る。30fps fixtureの既存操作も維持 |

結果: PlaybackControllerへcurrent/seek frameとframe durationを集約し、shortcut split/step/marker/trim、
Create、Ease、Timeline key/layer drag、Ease timecodeをComposition fpsへ接続。24fpsのframe 37 navigation、
Timeline key `12→13`、Text content frame 37のfocused oracleがPASS。30fpsはblank/fixture fallbackだけに残した。

### A4-AUDIO-PREROLL検索receipt — 実Playback初回にCPAL underrun

| field | 内容 |
|---|---|
| `NEED` | Audio layer実窓Playbackでdevice clockは進んだが、CPAL error callbackに`buffer underrun or overrun`。現行SessionはstreamをplayしてからMixProducerをspawnする |
| `SEARCHED` | `PlaybackSession`、`OutputStream::open_negotiated_shared`、MixProducer/rtrb、PlaybackCounters、CPAL StreamTrait／output callback timestamp |
| `DISPOSITION` | **REMAP**。buffer量やsleep値を発明せず、CPALのpaused build契約へ順序を合わせる |
| `OWNER/ROUTE` | `negotiate → build stopped stream → spawn producer → first successful ring push ready → stream.play`。以後のproducer/consumer/clockは既存 |
| `RULER` | [CPAL StreamTrait](https://docs.rs/cpal/latest/cpal/traits/trait.StreamTrait.html)はbuildしたstreamが停止状態で、`play`後にcallbackが始まると規定 |
| `ORACLE` | device無しunitでproducerがreadyを返す時consumerにsampleが在る。実窓で同じsoundtrackを再生しCPAL error 0、playhead進行、停止を確認 |

結果: stopped streamをbuildし、MixProducerの初回push ready後だけplay。unit PASS。
同じsoundtrackの実窓再生でplayhead進行・停止、CPAL error 0件。

### A4-DIRTY検索receipt — New/Open/Closeが未保存作品を即時破棄

| field | 内容 |
|---|---|
| `NEED` | File New/Openとmain window CloseにDocument revisionと最後のSave revisionの比較が無く、編集直後でも無確認で作品を失う |
| `SEARCHED` | `Session project_path`、Document revision、File New/Open/Save、Host CloseRequested、rfd MessageDialog、Apple HIG File management／NSDocument edited state |
| `DISPOSITION` | **REUSE_WRAP**。独自modal／autosave schedulerを作らず、Document revisionとOS message/save dialogを使う |
| `OWNER/ROUTE` | Sessionがsaved revisionを1つ記憶。Save成功／Open／Newで更新し、全Document Intent後はrevision差だけでdirty。New/OpenはSave・Don’t Save・Cancel、Closeは破棄確認 |
| `RULER` | [Apple File management](https://developer.apple.com/design/human-interface-guidelines/file-management)は未保存変更を示し、close/quit等でsave dialogを提示することを要求。[NSWindow documentEdited](https://developer.apple.com/documentation/appkit/nswindow/isdocumentedited)は変更ごとのedited stateとclose前確認を規定 |
| `ORACLE` | editでdirty、成功Saveでclean、追加editでdirty、Open/Newでclean。Save失敗／dialog Cancelでは元Document・path・saved revision不変。製品DOMはstatusにEditedを表示 |

結果: Sessionへsaved revisionを一つだけ置き、Save成功／Open／Newだけが進める。File New/Openは
OSのSave／Don’t Save／Cancel、main CloseはClose Without Saving／Cancelへ接続。revision unitと
製品DOMの`Edited → mark_saved → 消去`がPASS。OS dialogと実Openは実窓未検収。

### A4実窓記録(2026-09-02)

- `ordinary-pass1.rrd`へMask／Matte／Audioを含む作品をSave。`Edited`消去後、Openで4行・masked像・waveform復元
- dirty edit後のNewは親窓sheetでSave／Don’t Save／Cancelを表示。Cancel後も元作品を保持
- soundtrack Playbackはdevice clockで進行・停止。pre-roll修正後のCPAL errorは0件
- 既存`first-one.mp4`へExport→Cancel後もSHA-256
  `1f7af23ea3e2a90f7fa24a7121a880aac26daa1b6c08fe4aad0db9a3b1f57b4c`不変、temp残骸0
- `ordinary-pass1.mp4`完走。完了直後にtab reopen無しでOutput像とPanel／StatusBarの`Wrote`を維持
- FFprobe: H.264 1920×1080 / 30fps / 60秒 + AAC 48kHz stereo / 60秒。
  先頭3秒はmean -21.1dB、max -17.8dB

A4判定: **PASS**。A5へ進む。

### A5-RENDER-FRAME検索receipt — Export中にprevious encoder警告

| field | 内容 |
|---|---|
| `NEED` | Mask/Matte作品の60秒Export中、各frameで`There was still a command encoder from the previous frame`。fileは完成するがframe lifecycleが未閉鎖 |
| `SEARCHED` | pin済み`re_renderer::RenderContext::{begin_frame,before_submit}`、Compositor `flush_pending/finalize_readback`、旧`render_basic`、実Export log |
| `DISPOSITION` | **REMAP**。renderer／queue／submit機構を作らず、上流のbegin前before_submit契約へ揃える |
| `OWNER/ROUTE` | `flush_pending`後のpoll→2回目begin直前に`before_submit`。旧basic readbackも同じ順。frame-global encoderを次frameへ持ち越さない |
| `RULER` | pin済みRerun sourceはbegin時に残encoderをエラーとして検出し、通常flowをbegin→record→before_submitとする |
| `ORACLE` | 2frame以上のmasked render/exportで警告0、画素oracle不変。Pass 1の再Export logで警告0を確認 |

結果: readbackの2回目`begin_frame`直前に`before_submit`を明示。repeated masked frame 3 test PASS。
Pass 1の1800frame再Exportで`previous encoder`警告0、Output画素とH.264+A​​AC仕様を維持。

### A5-MAC-BRACKET検索receipt — Option+[がlogical `“`になりTrim無反応

| field | 内容 |
|---|---|
| `NEED` | Pass 1でOption+[を送ると`Key::Character("“")`、keymapは`[`を探してno-binding。macOS文字生成とshortcut identityが混線 |
| `SEARCHED` | Dioxus KeyboardData key/code、keyboard-types `Code::BracketLeft/Right`、pin済みBlitz key conversion、W3C UI Events code/key分離 |
| `DISPOSITION` | **REMAP**。shortcutやIMEを作らず、Bracket物理codeだけ既存`[`/`]` bindingへ写す |
| `OWNER/ROUTE` | root KeyboardData `key + code`→keymap。文字入力中は既存typing guardで無効、shortcut時だけIntentへ届く |
| `RULER` | W3C UI Eventsの`key`は修飾後の文字、`code`は物理key位置。Optionで文字が変わるshortcutはcodeをidentityにする |
| `ORACLE` | logical `“` + `Code::BracketLeft` + Altが`TrimToPlayhead(false)`。実窓でbar頭がplayheadへ移る |

結果: physical BracketLeft oracle PASS。実窓Option+[はlogical `“`のまま既存Trim Intentへ届き、
Text bar頭をframe 359へ移動。

### A5 Pass 1(2026-09-02)

clean launchから白紙→Rectangle／logo／soundtrack／Text→Duplicate→全層Group→Undo→Position 2 keys→
Elastic→Split→Trim→Mask/Matte→device Playback→Save/Open→Export Cancel→音声付き最終Exportを同一作品で完走。
`ordinary-pass1-final.mp4`: H.264 1920×1080 30fps 60秒 + AAC 48kHz stereo 60秒、size 396,696 bytes。
新規停止点はmac bracketとrenderer frame lifecycleの2件で、同じPass内に修正・再検収済み。

### A5-AUDIO-CANDIDATE検索receipt — 画像層までSymphoniaへ渡してERROR

| field | 内容 |
|---|---|
| `NEED` | Pass 2でlogoのDocument同期ごとにSymphoniaが872-byte PNGをprobeし、`probe reached EOF`をERROR出力した。音声trackは正しく1本だが、非音声層を音声decoderへ渡している |
| `SEARCHED` | `AudioProgram::from_view`／`project_soundtrack_input`、素材棚の唯一の`asset_type_for_extension`、Symphonia 0.6 `Hint`／`get_probe`公式docs、標準MIME top-level media type |
| `DISPOSITION` | **REUSE_REMAP**。codec sniff、MIME table、別asset registryを作らず、既存の拡張子分類口で`audio/*`と音声trackを持ち得る`video/*`だけを既知candidateにする。未分類の旧projectは従来probeへfallback |
| `OWNER/ROUTE` | `LayerSource::File path → media::asset_type_for_extension → AudioProgram candidate gate → existing Symphonia decode`。画像／3D／dataはdecoderの外、audio／videoは既存mix経路 |
| `RULER` | [Symphonia Hint](https://docs.rs/symphonia-core/latest/symphonia_core/formats/probe/struct.Hint.html)はextension/MIMEをembedderが渡す文脈とし、default probeは選択featureのformatだけを登録する。[MDN MIME types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/MIME_types)は`audio`／`video`／`image`を別top-level typeとする |
| `ORACLE` | 存在しない`.png`のFile層を含むDocumentでもAudioProgram構築がI/O/probeへ触れず、source／waveform 0。WAV二層cacheと動画音声candidateを維持。最終合否はclean launchログでPNG由来Symphonia ERROR 0 |

結果: `.png`を音声probeへ渡さないfocused testを追加し、waveform 6/6 PASS。修正版bundleの
clean launchから`ordinary-pass2.rrd`を開くとwaveform 1本が復元し、PNG由来Symphonia ERRORは0。
同じ作品をdevice Playbackしてplayheadも進行した。動画は埋込み音声を持ち得るcandidateとして維持するため、
音声無しMP4ではSymphoniaのcontainer WARNが出るが、画像を繰返しdecoderへ渡す停止点は閉じた。

### A5 Pass 2(2026-09-02)

別processのclean launchから、白紙→Rectangle／Text／logo／soundtrack→Duplicate→全層Group→Undo→
Position 2 keys→Elastic→Split→物理Option+[ Trim→Gradient→Rectangle Alpha Matte→Mask→device Playback→
Save/Open→Export Cancel→音声付き最終Exportを同一作品で完走した。

- `ordinary-pass2.rrd`: Effect／Matte／Mask／Audio／key／split／trimを含めSaveし、Openで同じ7行、
  gradientの局所像、Matte source、Mask `1 attached`、waveformを復元
- Cancel: `ordinary-pass2-cancel.mp4`も同directoryの一時出力も残らず、実窓statusは`Cancelled`
- `ordinary-pass2-final.mp4`: 1800/1800後、tab reopen無しで`Wrote`を1秒後も維持。
  FFprobeはH.264 1920×1080 30fps 60秒 + AAC 48kHz stereo 60秒、size 765,115 bytes。
  先頭3秒はmean -21.1dB、max -17.8dB。Rerun previous-encoder警告0
- 修正版の追加clean launchで`clip.mp4`をImport→Video card→Timelineへ置き、次frameで640×360の
  test patternをStageに表示。Inspector Position Xを`640→800`へ直接入力し、Undo→Redoも往復

### DONE-HEADLESS-WINDOW検索receipt — dirty guardがGUI harnessを窓必須にした

| field | 内容 |
|---|---|
| `NEED` | final `cargo test`でGUI 20本が同じ`Could not find context Arc<dyn Window>`からDOM 0件になった。製品windowは正常だが、New/Openの親窓sheet用`use_window()`をApp rootで必須化したためheadless harnessがbuild不能 |
| `SEARCHED` | pin済みDioxus Native `use_window`実装、Dioxus 0.7 `try_consume_context`、rfd `AsyncMessageDialog::set_parent`、既存`blitz-test-harness`のroot contexts |
| `DISPOSITION` | **REMAP**。偽Window／別App／test分岐を作らず、Dioxusのoptional contextをそのままdialogのoptional parentへ写す |
| `OWNER/ROUTE` | product hostは`Arc<dyn Window>` contextを供給→parented sheet。headless harnessはWindow context無し→同じApp DOM、dialogを呼ぶ時だけunparented fallback |
| `RULER` | [Dioxus `try_consume_context`](https://docs.rs/dioxus/latest/dioxus/prelude/index.html)はcontextが無い時`None`を返し、`consume_context`はpanicする。rfd 0.17の`set_parent`自体がoptional API |
| `ORACLE` | `cargo test --lib`でGUI 20本を含む全62 testがDOMを構築しPASS。製品clean launchではNewのdirty sheetが従来どおり親窓へ付く |

結果: Window contextをoptionalにし、GUI 20本を含む`cargo test --lib` 62/62 PASS。
修正版bundleのclean launchでRectangle編集→File Newを行い、親窓sheetにSave／Don't Save／Cancelが出ること、
Cancel後もRectangleと`Edited`が残ることを再確認した。

## 最初の問い — 実装する前に疑う

各責任は、最初に次をこの順で調べる。

1. pin済みRerun/Blitz/winit/FFmpeg/symphonia/vgpu/OSに同じ機構がないか。
2. Motolii内に、未接続の機械として既にないか。
3. 一般編集ソフトで収束している操作文法は何か。
4. Motoliiが所有すべき編集の意味か。

結果は`REUSE / WRAP / PORT / PATTERN / OWN`のどれか。新しいtrait、registry、renderer、parser、
codec、focus manager、dock機構を作り始めたら上流調査へ戻る。`OWN`できるのは編集の意味と
エフェクトのdataだけ。

T+1:00の実測分類は [ordinary-upstream.tsv](../motolii/reference/ordinary-upstream.tsv)。

## 学習cycleとcircuit breaker

- 20分を1cycleにし、開始時に「仮説／期待する証拠」、終了時に「実際／学び／次」を1行ずつ残す。
- 1cycle目は窓が変わらなくても、新しい事実または反証が得られれば成功。
- 2cycle連続(40分)で新しい事実・反証・実窓変化が0なら、scope縮小か上流候補変更。
- delivery sliceは60分以内に最初の実窓変化を出す。出なければSpikeへ戻す。
- 同時WIPは製品slice 2本 + read-only Spike 1本まで。空きagentを埋めるための仕事を作らない。

## component leverage gate

各sliceは「一例を完成」ではなく、同族をdataへ変える。

| gate | 合格 |
|---|---|
| owner | 同じ意味の書き場所がcomponent/typed port一つ |
| second instance | 二つ目がmanifest/data/compositionだけで通る |
| repair | 一つの修正が全surface/instanceへ届く |
| deletion | feature固有branch、local copy、受渡しadapterが増えず、可能なら減る |
| reward | 実装数でなく、不要になった将来codeと保守点を報告 |

二つ目のためにShell、Intent、renderer、Inspectorを再編集したら1つ目は未完。逆に一つのcomponentが
複数の意味族を知ったら万能component化なので分ける。

## timetable

| 経過 | 作業 | 通過条件 |
|---|---|---|
| T+0:00–0:20 | Git/build/実窓baseline、短い制作経路 | 起動条件と最初の停止点が確定 |
| T+0:20–1:00 | UI/input、Timeline/audio、Composite/Vism、Project/outputの上流調査 | 全領域が`REUSE等`で分類、scratch候補0 |
| T+1:00–1:20 | 今夜のscopeをshape | 独立slice最大2本、Spike最大1本 |
| T+1:20–3:20 | Wave A: Window/Input + Timeline/Audio | 各sliceが実窓に出てcomponent gateを通る |
| T+3:20–3:40 | 「最初の一本」途中まで実操作 | 次の最初の停止点を1つ特定 |
| T+3:40–5:40 | Wave B: Project/Output + Inspector/Composite | save/open/exportとblend/mask/effectが製品経路で動く |
| T+5:40–6:40 | 全窓の視覚バランス | Stage主、Timeline第二、Browser/Inspector補助。切れ・死にchrome 0 |
| T+6:40–7:00 | Q0掃討、新規実装freeze | 無反応、fixture表示、未完branch 0 |
| T+7:00–8:00 | 白紙から制作経路を完走 | import→編集→再生→保存→再読込→export |
| T+8:00–8:40 | latency/hot reload/gesture | 既存器具でQ/B違反確認。計測基盤は作らない |
| T+8:40–9:20 | clean launch最終検収 | 同じ操作を2回通し、結果とUndoが一致 |
| T+9:20–10:00 | buffer、部分実装撤去、gap更新 | 朝へ未完成chrome・一時経路を残さない |

## 朝のPASS

白紙→画像/動画/音声drop→Stage直接操作→Timeline scrub/zoom/trim/split/duplicate→Inspector値入力→
key/ease→blend/mask/effect→音付き再生→Undo/Redo→Save/Open→Outputと一致する音付きexportを実窓で2回通す。

Q0/Q1/Q3/Q5/Q7/Q9を必須にし、視覚は新しいthemeを作らず、既存tokenで整列・比率・間隔・
contrast・hover/pressed/selected/focusを揃える。1600x1000と1280x800で切れ、重なり、極端な余白を確認する。

## 2026-09-02 実行結果

**判定: 閾値へ大きく前進したが、上の「朝のPASS」全体は未認定。** 実装済みと実窓で通した物を分ける。

| slice | component leverage | 証拠 |
|---|---|---|
| File/View | `SemanticMenu`/`SemanticControl`一族。死んだmenubar項目を撤去 | 外クリック、Escape、相互切替、click-through防止を実窓とGUI testで確認 |
| Dock | `dioxus-workbench::PanelLayout`がtree/tab/active/share/reconcileを所有。`dioxus-dnd::transition`がtap/6px drag/drop/cancelを所有し、Blitz/Taffy実寸を一つの投影が読む | 旧7 REDを全てtestで閉じ、同一実窓でclick→split→Reset Layoutを確認。完成DnD componentはBlitzの`get_client_rect`二重borrowを起こすため純粋algorithmだけ採用 |
| Inspector choice | `ChoiceId` + 窓所有のdismiss一つ。Parent/Blend/Matte source/modeが共有 | Matte候補を開き、Stage外クリックで閉じることを実窓確認 |
| Matte | Parentと同じ`LayerChoiceRow`、書込みは`SetAttrs`一回 | 4 mode/None/一Undo、さらに選択がproduct pixelを変えるtest |
| Group | `Document::group_layers/ungroup_layers`が全入口の意味を所有 | 共通親、順序、一Undo、atomic rejection、nested一段を6 testで確認 |
| Audio/Timeline | Symphonia対応拡張子を`media`の分類口一つへ接続。`PlaybackController::sync_document`が時間軸/audio programを再投影 | 一時WAVをAudio棚→カード→層→包絡波形まで実窓確認。PCM/peak cacheは二層目もdataだけ |
| Output | `ExportController` + 同じ`OutputStatus`をpanel/status barへ投影 | OS保存先→`797/1800`進捗→完成まで実窓。H.264 1920x1080/60秒をffprobe確認。Cancel表示も確認 |
| Vism | manifest/inventory/program mapをshader fileから生成 | effect固有catalog/matchを削除。full manifest/layout HMRは未達のまま明記 |

検証は`cargo check --locked --tests`、`cargo test --locked --no-fail-fast`の全73 test、
`scripts/check-docs.sh`、`git diff --check`を通過。実窓はホスト内部の初回paint `1640x992`、
取得画像 `1280x768`で、Stage主・Timeline第二・Browser/Inspector補助の比率、死にchrome、候補の切れを確認した。

未通過gateは次の4つ。ここを通すまで「普通になった」とは言わない。

1. 音声deviceを使う実再生(就寝中のため音を出さず、波形だけ実通)。
2. 実ExportのCancel作用(表示までは実通。就寝中にGUI削除を起こさず、完走へ切替)。
3. 白紙→import→編集→Save/Open→音声付きexportの実窓2巡。
4. Vismの新規file/manifest/Inspector layoutまで含むfull hot reload。

Dockの残余は機構修正ではなく適用範囲である。再起動を越えるlayout保存、keyboardだけでのdock、
別monitor/DPIを跨ぐdetach/reattachを製品範囲へ入れる時に、それぞれ独立した受入を追加する。

同日の適用範囲拡張で、Stage/Timeline/Easeのcancelを`GestureSurface`一つへ、click-only chromeを
`SemanticButton`一族へ、Finder dropとImport dialogを`ImportSummary`一口へ畳んだ。file hover/cancelと
失敗理由は製品treeへ出る。multi-window hot reloadの実窓検収中に、全proxy eventを共有wakeへ変換する
循環を発見したため、外部状態変更`Woken`だけが全windowを起こすようHostを修正した。
公式Dioxus 0.7の`apply_changes → View::poll`経路へ揃え、`dx serve --hotpatch`でmainとdetachedの
両方に同じRSX変更と復帰が届くことを実窓確認した。

## 2026-09-02 最終追補 — 現在判定

上の「未認定」はA0時点のsnapshotであり、A1〜A5のreceipt後の現在判定は **PASS**。
目標だった「AEの内部都合を増やさず、白紙から素材・編集・合成・音付き確認・保存再読込・
音付き書き出しまで実窓で2巡」は、Pass 1／Pass 2の別process clean launchで完走した。

| gate | 最終証拠 |
|---|---|
| 素材 | logo／soundtrackを両PassでImport→card→層。追加clean launchで`clip.mp4`の640×360 frameをStage表示 |
| 編集 | Stage move、Inspector直入力、Duplicate、Group、Undo/Redo、Position 2 keys、Elastic、Split、物理Option+[ Trim |
| 合成 | W3C Screen画素、Gradient、Alpha Matte、Mask coverageとUndo。Effect+Matteの全面化なし |
| 音 | waveform、device-clock Playback、CPAL error 0。PNGをaudio probeへ渡すERRORも修正版clean launchで0 |
| Project | `ordinary-pass1.rrd`／`ordinary-pass2.rrd`をatomic Save→Openし、層・key・Mask・Matte・Effect・Audioを復元。dirty NewのCancelで元作品保持 |
| Output | 両PassでCancelと1800-frame完走。完了直後から`Wrote`維持、Rerun previous-encoder警告0 |
| 外部測定 | 2本ともH.264 1920×1080 30fps 60秒 + AAC 48kHz stereo 60秒。Pass 2はmean -21.1dB／max -17.8dB |
| 回帰 | `cargo test --locked --no-fail-fast` **93/93 PASS**、`scripts/check-docs.sh` PASS、`git diff --check` PASS |

適用範囲残余は、Finderからの実drop、任意Mask path編集、main window Closeの保存選択再試験、
Vismの新規manifest／Inspector layoutまで含むfull HMR。いずれも今回の二巡を仮合格にするための
隠れ条件ではなく、次の機能／環境受入として[上流採択台帳](../motolii/reference/ordinary-upstream.tsv)に残す。

A5判定: **PASS**。DONE判定: **PASS**。
