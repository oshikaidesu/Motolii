# UI 入口台帳調査(κ)— S 空間スコアの土台(RESEARCH_RETURN 保全)

日付: 2026-08-21 / 発注: 後任セッション(S 空間スコア構想の第一手)/ レーン: read-only 調査(sonnet)
正典化: 本台帳を土台に **[ui-spatial-score.md](../ui-spatial-score.md)(S 空間スコア)** を制定。違反在庫の焼却状況はそちらと lane board が正本。

## 入口台帳(`|` 区切り)

列: `操作名 | 種別(a-d,自信度) | 現在の入口 | Message/Intent | map行id | S0期待入口(m:s:p:pref) | 差(S0>a-d 辞書式)`

```
Undo | c(高) | headerボタン | Message::Undo | 437(3:3:0:0)/466 | shortcut優勢 | S0違反: Cmd+Z未配線(grep実測)・メニュー非存在
Redo | c(高) | headerボタン | Message::Redo | 435(3:3:0:0) | shortcut優勢 | 同上
AddLayer | b(中) | headerボタン | Message::AddLayer | 900(0:1:0:0) | shortcut | 軽微
素材Import | c(中) | OSドロップのみ | AdmitPaths/DropReceived | 592(4:1:0:0) | menu優勢 | S0強違反: freq4でメニュー相当入口皆無
Inspector Transform編集(type/scrub) | b(高) | 値セル直接 | FieldInput等/ValuePressed等 | ― | ― | 適合
Inspector Name編集 | b(高) | ident帯直接 | NameInput/Submit | 785(0:1:0:0薄) | shortcut薄 | 大差なし
Inspector ToggleHidden | b(高) | ATTRSボタン | ToggleHidden | ― | ― | 適合
Inspector CycleBlendMode | b(中) | 巡回ボタン | CycleBlendMode | 78(0:1:0:0) | shortcut | S0とb文法が矛盾(freq1薄・裁定要)
Inspector Speed編集/Reset | b(中) | Speed欄+Reset | SpeedInput等 | 169(3:2:0:0) | menu+shortcut | S0中違反(判定分かれる・自信中)
Timeline行選択 | b(高) | レーンバー直接 | Select | ― | ― | 適合
Timeline scrub | a(低・迷い) | ルーラー直接+transport slider | ScrubTo | ― | ― | ―
M/S/L glyph | b(高) | glyph直接 | ToggleMute/Solo/Lock | 1345/1355/1339(各1:0:0:0) | menu(採取偏重疑い) | データ信頼度低
Clip move/trim | b(高) | バー直接drag | BarGrabbed系 | ― | ― | 適合
Key select/削除 | b(高) | diamond直接+Delete | KeySelect/DeleteSelectedKeys | ― | ― | ―
Key時刻drag/retime | b(高) | diamond直接drag | KeyGrabbed系 | ―(Motolii固有) | ― | ―
NudgeKeyframe | b(高) | キーのみ・affordance皆無 | NudgeKeyframe | ― | ― | 発見可能性ゼロ
StepPlayhead | a(中) | キーのみ | StepPlayhead | 1042/1043(0:3:0:0) | shortcut | S0適合・a文法(chrome)矛盾
JumpPlayheadToStart/End | a(中) | キーのみ | 同名 | 1044/1045(0:2:0:0) | shortcut | 同上
JumpMeaningPoint/ClipEdge | a(中) | 裸key(J/K/I/O) | 同名 | ― | ― | ―
CopyLayer | b(高) | **入口なし** | CopyLayer | 429(4:2:0:0) | menu+shortcut freq4 | 群0最大級
PasteLayer | b(高) | **入口なし** | PasteLayer | 430(4:2:0:0) | 同 | 同上
CutLayer | b(高) | **入口なし** | CutLayer | 432(3:2:0:0) | 同 | 群0
DuplicateLayer | b(高) | **入口なし** | DuplicateLayer | 434(3:1:0:0) | menu優勢 | 群0
SelectAllLayers | c(高) | **入口なし** | SelectAllLayers | 436(3:3:0:0) | menu+shortcut | 群0
DeselectAllLayers | c(高) | **入口なし** | DeselectAllLayers | 433(3:3:0:0) | 同 | 群0
TogglePlayback | c(中) | Space+transportボタン(両方) | TogglePlayback | 1041(1:3:0:0) | shortcut優勢 | **S0適合の良好事例**
ToggleSettingsPanel | a(高) | headerボタン | ToggleSettingsPanel | 1144(2:1:0:1) | ― | 適合
ToggleCheckerboard | a(高) | Settings内ボタン | ToggleCheckerboard | ― | ― | **文法違反: a型がd住所に同居**
BackgroundPreset/Channel編集 | c(中) | Settings内 | BackgroundPreset等 | ― | ― | 文法違反(仮): c型がd住所
UiScale編集 | d(高) | Settings内 | UiScaleInput/Submit | ― | ― | 適合
Observe(観測カメラ) | a(高) | Stage直接操作のみ・chrome痕跡ゼロ | stage::Observe | ― | ― | 文法違反: a型でchrome手がかり皆無
ResetToRenderCamera | a(高) | Shift+F(仮)のみ | ResetToRenderCamera | ― | ― | 同上・復帰操作が発見不能
Stage preview解像度 | d(中) | **完全に入口なし**(固定定数 STAGE_HANDLE_SYNC_BUDGET_BYTES) | ― | 未特定 | ― | 群(i)最優先(裁定21隣接)
```

## 違反ランキング

**群0(S0 段差・最優先)**: (1) Copy/Paste/Cut/Duplicate/SelectAll/DeselectAll = freq3〜4 で入口ゼロ (2) Undo/Redo = shortcut 多数派なのに Cmd+Z 未配線 (3) 素材 Import = menu 優勢なのに D&D のみ (4) CycleBlendMode = S0 と b 文法の矛盾(薄い・裁定要)
**群(i) 入口なし**: preview 解像度 / 編集動詞6種 / NudgeKeyframe・ResetToRenderCamera(affordance 皆無)/ Import
**群(ii) 種別×住所不適合**: 市松(a が Settings に同居)/ 背景色(c が Settings)/ 観測カメラ(a で chrome 痕跡ゼロ)/ playhead ナビ(S0 は適合・a 文法とは矛盾 — 辞書式で軽微)
**群(iii) freq×距離**: Copy/Paste(freq4×∞)> Undo/Redo・SelectAll > Import

## 器具化材料

- `screenshot.rs` は widget-tree walker では**ない**(Dimensions からの手計算並行実装 — 座標抽出の基盤に不向き、二重保守リスク)
- 真の walker = `q0_fence.rs::collect_targets`(`iced_test::simulator`、フル Shell::view を走査可能)。**最小の口 = これを汎用化して TSV(`id\tx\ty\tw\th\tcontent`)を吐く dev 専用 test file(`entrance_atlas_dump.rs`)を1本**
- `iced_test` は dev-only 境界(Cargo.toml 明記)— CLI フラグ化は境界違反、tests/ 内 instrument が筋
- 既知の限界: Target はスタイル情報(色・border)を持たない → S1 には十分・S3 は screenshot 画素側から測る(別器具)

## EVIDENCE_GAP

1. preview 解像度の normal-map 対応行未特定(Resolve proxy 系が近い可能性)
2. 観測カメラ・playhead ナビ系は map に対応行が乏しい(4製品の語彙が独立 entry 化していない可能性)
3. M/S/L の map 行(entries=1:0:0:0)は track-header 常設文化を過小採取している疑い — S0 判定信頼度低
4. Inspector 個別フィールドの一部は freq=1 の薄い出典依存

## FINDING

1. **doc と実装の乖離**: lib.rs L278-280 は「仮割当 Cmd+C/V/X/D/A あり」と主張、実装は割当ゼロ(→ λ レーンで根治)
2. **右クリック基盤がアプリ全体に不在**(cancel 専用2箇所のみ)— 種別 b の住所階梯の中段が構造的欠落
3. **メニューバー自体が不在** — 種別 c の住所の片方が構造的欠落。c 判定の全操作がこの影響下
4. 観測カメラの入口・復帰は chrome 上の視覚的痕跡ゼロ(裁定157 の「表示専用」判断自体は妥当、発見可能性は別軸の懸念)
