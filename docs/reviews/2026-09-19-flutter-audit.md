# Flutter UI の組み方 監査(2026-09-19)

対象: `motolii/ui/lib`(48 file、21,950 行)+ `motolii/ui/native`(Rust 2,663 行)+ `motolii/ui/macos/Runner/MainFlutterWindow.swift`(693 行)。読むだけ、コードは触っていない。定規は docs.flutter.dev の一次資料(末尾 Sources)。行番号は HEAD `6cfb7b8ff` 時点。

## 要約 5 行

1. **骨は悪くない**。状態は `EditorSession` 1 本 → `DocumentSlice`(鍵ごとの購読)→ panel、絵は IOSurface → `Texture`(コピー無し)、Timeline/Stage は `CustomPainter` + `shouldRepaint`。Flutter の推奨(ValueListenable・Texture・CustomPainter)に沿っている。
2. **効率を落としているのは「毎コマの JSON + Dart 側の全コピー」**。再生中は vsync ごとに `render` を MethodChannel で往復し、全層の `liveLayers` を JSON で受け、`EditorSession.maps()` が層ごとに Map をコピーし、Inspector・Stage・Timeline が毎コマ読み直す。
3. **panel が 1 class 2,000 行で、build が 400〜700 行、helper method で木を組む**(ease_desk build 679 行、browser 386 行)。Flutter の「木は widget に割れ、method に割るな」に反し、`const` も `RepaintBoundary`(48 file で 0 件)も効かない。
4. **同じ状態機械が 4 回書かれている**(数値欄・ダイヤル・パッド・グラデーションの drag→preview→commit)。`panel_controls.dart` 内だけで 7 行窓の重複 46 箇所。AGENTS の「1 つ直せば同族全部」に反する。
5. **theme は 1 本(`EditorTheme` + `EditorMetrics` + raw_dimension lint)で寸法は守れているが、色は painter に 37 箇所の生 `Color(0x…)`**(timeline 20)。`flutter analyze` は 30 件、うち 29 が `colors_shelf.dart:71-111` の貼り間違い(HEAD で compile が通らない)。

## 構造

**状態の流れ**(`session/editor_session.dart`)

- `EditorSession.document: ValueNotifier<Map<String,dynamic>>`(56 行)が唯一の書類状態。返信は `_accept`(333-365)→ `absorb` → `take`(154-166)で「変わった鍵だけ差し替え」。
- 購読は `DocumentSlice`(13-30): panel が読む鍵の集合を名指しし(`slice('timeline', ['layers','selectedId',…])` timeline.dart:951-961、inspector.dart:81-95、stage.dart:991/1010)、`_spreadDocument`(135-150)が**返信ごとに全鍵を `sameValue` で深く比較**(33-51)して変わった鍵だけ通知する。設計は Flutter の「ValueListenable を細かく持て」の型そのもの。
- ただし `map()`/`maps()`(325-328)は `Map.from` で**毎回コピー**。`layers` getter(252)は呼ぶたび全層コピー。呼び出しは 48 file で 130 箇所超(inspector 31、ease_desk 16、timeline 11)。
- 型付き model は無い。`Map<String, dynamic>` が 375 箇所、`layer['id'] as num` 型の読みが panel 全域。Flutter の architecture guide が言う「UI 層は ViewModel の型付き値を読む」段が抜けている(`read_model.dart` は 9 行の別名だけ)。
- `controller` を 19 の widget が constructor で受け取り(`required this.controller`)、InheritedWidget/Provider 相当は `EditorScale`(panel_controls.dart:1689)と `_EditorMenuScope`(theme.dart:403)の 2 つだけ。深い所は引数の受け渡し。

**setState の出所**(合計 ≈150 箇所)

| file | setState | 何に使うか |
|---|---|---|
| timeline.dart | 26 | drag・scrub・lane 開閉、`frameMoved`(155-167)は再生中コマが端を越えると root の setState |
| editor_window.dart | 22 | **panel の show/close/move を root State の setState で**(181-292)→ 窓全体 rebuild |
| panel_controls.dart | 22 | 各 control 内。妥当 |
| browser.dart | 15 | 1 class 1,118 行の中 |
| ease_desk.dart | 12 / stage.dart 10 / notes_desk.dart 10 | 同上 |

**木の深さ・widget の数**

- widget class は 66。panel は「1 class + `Widget _xxx()` の helper 群」: inspector.dart は class 3(`InspectorPanel`/`_Live`/`_SectionLabel`)で helper `_effect` 169 行(1380)、`_controlBody` 150 行(1720)、`_world` 137 行(1165)、`_transform` 122 行(1040)。
- 最大 indent: panel_controls 32 段、stage 28、browser 28、timeline 27、ease_desk 26(Flutter 既定 2 space 換算)。
- `const` 付き生成 vs 無し: timeline 56:94、ease_desk 58:47、inspector 61:40。helper method 由来の木は `const` に出来ない。

**重複**(7 行窓、空白正規化、2 箇所以上): 68 窓

- `panel_controls.dart` 内 46 窓。`_end(bool cancel)` の drag 終了処理が **655-668 / 1144-1155 / 1466-1477 の 3 写し**(EditorNumericField・EditorDial・EditorPad)、`WidgetsBindingObserver` の `didChangeAppLifecycleState`/`didChangeViewFocus` も同型。`gradient_inspector.dart:40-62` が 4 写し目(panel_controls との重複 18 窓)。
- `browser.dart:628` ↔ `browser/filters.dart:777`(名前の省略 Text、7 窓)、`colors_shelf.dart` 内 7 窓、`files_shelf` ↔ `media_shelf` 3 窓。
- `EditorNumericField` は 383-900 で **約 520 行の 1 control**。

## 描画

- **CustomPainter**: timeline `_TimelinePainter`(1377-2020)・`_ArrangementOverview`(2025)、stage `_StageOverlay`(1361-1545)、panel_controls に `_DialPainter`/`_PadPainter`/`_TrackPainter`/`CheckerPainter`。全て `shouldRepaint` 実装済み(timeline 2003-2012 は `identical(rows)` + `listEquals`)。良い。
- **RepaintBoundary: 0 件 / 48 file**。Texture の周り(stage.dart:1200)、CustomPaint の周り(timeline 1037/1096/1339)にも無い。Flutter は「頻繁に描き直す部分は RepaintBoundary で隔離」(Performance best practices)。今は Stage の絵が動くたび、親 layer まで raster が伝播する。
- **1 つの Listenable に束ねて丸ごと rebuild**:
  - stage.dart:1073-1080 `AnimatedBuilder(animation: Listenable.merge([_slice, c.rendered, c.textureIds, c.playing, c.anchorPreview]))` の中に LayoutBuilder → Stack → Texture → CustomPaint の全部。`c.rendered` は再生中**毎コマ**変わる → Stage の木を毎コマ組み直し、しかも build 内で `addPostFrameCallback(_syncWindow)`(1094-1095)を毎回登録。
  - timeline.dart:1014-1018 `Listenable.merge([frame, _timeline, scrolled])` → 目盛りの GestureDetector/CustomPaint を毎コマ再生成。painter に渡す `markers` は `EditorSession.maps(state['markers'])`(1049-1051)で build のたび List を新しく作る。
- **ListView.builder vs Column**: 行の羅列は timeline が CustomPaint、inspector が `SliverList.list` + `SliverReorderableList`(1953-2050)、history が `ListView.builder`(1 件)。Column に大量の子を積む所は無い。良い。
- **毎コマの割り当て**(再生時、Dart 側)
  1. `_beginCadence`(editor_session.dart:607-623): `Ticker` が vsync ごとに `native('render')` → 返信 Map 全体を `rendered.value = next`(352)。
  2. inspector `_absorb`(inspector.dart:116-131)が `rendered` の listener → `_read`(135-160)→ `c.liveLayers()`(268-277: 全層を `maps` でコピーし overlay)→ `_byId` を全 property/effect param で組み直し → `_stampRow` を **全 pulse 行**で評価 → `_stampShape` を `sameValue` で深比較。
  3. timeline `frameMoved`(155-167)→ `overviewExtentWithoutFrame`(171-181)が `widget.controller.layers` で**全層コピー**を毎コマ。
  4. ease_desk build(526-1205)は `jsonEncode(_payload(p))` を preset ごと毎 build(544-545)。
- **絵の受け渡し**: コピー無し。Rust が `IOSurfaceRef::lookup` → Metal texture に直描き(native/src/lib.rs:152-180)、Swift `ProbeTexture: FlutterTexture` が `CVPixelBuffer` を差し替え `textureFrameAvailable`(MainFlutterWindow.swift:199-235、381-394)、Dart は `Texture(textureId:)`(stage.dart:1200)。Flutter の Texture widget の想定通り。

## テーマ

- 単一の出所はある: `foundation/theme.dart` `EditorTheme`(static const 色 8-50、`ThemeData.dark(useMaterial3).copyWith` 124-250)と `foundation/metrics.dart` `EditorMetrics`(寸法 token、`raw_dimension` lint が生数字を IDE と test で拒否)。参照は `EditorTheme.` 388 箇所、`EditorMetrics.` 681 箇所。生の `EdgeInsets(数字)`/`height: 数字` は 10 箇所(lint が効いている)。
- `ThemeExtension` 無し、`Theme.of(context)` 0 件。色は static const なので context 非依存 = light/高コントラスト切替や test 差替えが出来ない(dark 固定は視覚言語の決定なので今は代償のみ)。
- **生 `Color(0x…)` は theme 外に 37 箇所**: timeline.dart 20(1473・1501-1504・1523・1533・1541・1548・1753-1770・1884-1983 の painter 内)、filters.dart 7、ease_desk 3、depth_desk 2、stage 2、color_field/panel_controls/workspace_view 各 1。寸法には lint があり色には無い、の差がそのまま出ている。

## 橋

- **送り**: Dart Map → `jsonEncode({'op':…})` で文字列(protocol.dart:108-112)→ MethodChannel `request` の引数に文字列として載せ(native_bridge.dart:12-18、StandardMethodCodec で再符号化)→ Swift が `JSONSerialization` で **読み戻して** op を見て(MainFlutterWindow.swift:561-575)→ C 文字列で Rust `motolii_probe_request`(lib.rs:221)→ serde_json parse。**op 1 回に符号化 3 回・復号 3 回**。
- **返し**: Rust が status を serde_json 文字列 → Swift `JSONSerialization` → Dictionary → StandardCodec → Dart Map → `sameValue` 深比較。
- **同期 block は無い**: Swift `worker` DispatchQueue で実行、main で返す(`perform` 621-632)。Dart 側は `_serial`/`_tail`(367-390)で 1 本に直列化、`rendering` flag で render は 1 つ飛行。
- **polling timer は無い**: `Timer` は debounce のみ(save 300ms editor_window:159、hover 90ms blend_panel:185、export_controls 2 件)。再生は `Ticker`(vsync 駆動)。
- **毎コマ**: Ticker → `render` → Swift が `tick` を request → `render` FFI → **軽い status でも `liveLayers` に全層の properties/effects を JSON で同梱**(snapshot.rs:284-288、snapshot_cache.rs:124)。60fps × 層数 × property 数の JSON が Rust→Swift→Dart を毎コマ通り、Dart 側で上記の全コピーを起こす。dart:ffi は使っていない(0 file)。

## 表

| file | 行 | widget class | 最長 build/helper(行) | 最大 indent | setState | 生 Color | 直近 30 日の commit |
|---|---|---|---|---|---|---|---|
| panels/inspector.dart | 2140 | 3 | `_effect` 169 / build 139 | 19 | 6 | 0 | 35 |
| panels/timeline.dart | 2110 | 1 | build 241 / `_bar` 129 | 27 | 26 | 20 | 17 |
| foundation/panel_controls.dart | 1867 | 16 | build 201(EditorNumericField 全体 ≈520) | 32 | 22 | 1 | 30 |
| panels/stage.dart | 1689 | 1 | build ≈340(1022-1360) | 28 | 10 | 2 | 20 |
| panels/ease_desk.dart | 1539 | 2 | **build 679** | 26 | 12 | 3 | 12 |
| panels/browser.dart | 1118 | 1 | **build 386** | 28 | 15 | 0 | 53 |
| panels/browser/colors_shelf.dart | 1006 | 2 | build 203 | 21 | 5 | 0 | — |
| panels/browser/filters.dart | 919 | 7 | build 161 | 19 | 5 | 7 | — |
| session/editor_session.dart | 758 | 0 | — | — | 0 | 0 | 19 |
| panels/notes_desk.dart | 713 | 2 | build 256 | 17 | 10 | 0 | — |
| app/editor_window.dart | 611 | 1 | build 199 | 25 | 22 | 0 | 10 |
| foundation/theme.dart | 437 | 6 | — | — | 0 | 39(正本) | 18 |

全体: 21,950 行 / widget class 66 / `RepaintBoundary` 0 / `ListView.builder` 1 / `CustomPainter` 8 / `Map<String,dynamic>` 375 / test 60 file 9,419 行。

`flutter analyze`(.tools/flutter、4.2 s): **30 件 = error 29 + info 1**。error 29 は全て `lib/panels/browser/colors_shelf.dart:71-111` — `classification()` の本文の途中に `groups()`/`tagsOf()`/`valueOf()` が貼り込まれている(commit `bdb68b5e7`、9/13)。HEAD の UI は compile が通らない状態。info 1 は `media_shelf.dart:2` の不要 import。

## 判定 5 つ(効き × 直しやすさ の順)

**1. 毎コマの JSON と Dart 側の全コピー**(効き: 再生の滑らかさ・層が増えた時の天井、直し: 小)
- 証拠: editor_session.dart:607-623(Ticker → render)、325-328(`map`/`maps` がコピー)、268-277(`liveLayers`)、inspector.dart:116-160(毎コマ `_read` + 全 pulse 評価)、timeline.dart:155-181(毎コマ `controller.layers`)、snapshot.rs:284-288(`liveLayers` 全層同梱)。
- 反する定規: Platform channels「messages は小さく、必要な時だけ」/ Performance best practices「build と listener の中で割り当てを増やさない」。
- 最小の直し: (a) 再生の返信は `frame` と `renderMs` だけにし、`liveLayers` は「今 Inspector に見えている層 id」を request に添えた時だけ返す(Rust 側 1 分岐)。(b) `maps()` を `List<Map>` の**読み取り view**に変えコピーをやめる(`v is Map ? v as Map<String,dynamic>` — 返信は既に `Map<String,dynamic>`)。(c) timeline `frameMoved` は `durationFrames`+`layers` の slice が動いた時に端を 1 回計算して持つ。

**2. 1 つの Listenable に束ねた丸ごと rebuild と RepaintBoundary 0**(効き: 毎コマの build/raster 幅、直し: 小〜中)
- 証拠: stage.dart:1073-1095(5 本 merge → Stage 全木、毎 build に postFrameCallback)、timeline.dart:1014-1051(frame を含む merge → 目盛り全体、`maps(markers)` 毎 build)、editor_window.dart:181-292(panel の show/close が root setState)。`RepaintBoundary` 0/48。
- 反する定規: Performance best practices「rebuild は木の小さい部分に閉じ込める、`child` 引数で不変部分を渡す、RepaintBoundary で描き直す部分を隔てる」/ StatefulWidget「Performance considerations」。
- 最小の直し: Stage は `Texture` と `_StageOverlay` の CustomPaint だけを `rendered`/`frame` の `ValueListenableBuilder` に入れ、外側の Stack・LayoutBuilder は `child:` で 1 回建てる。Texture と各 CustomPaint を `RepaintBoundary` で包む(3 行 × 4 箇所)。`_syncWindow` は build ではなく viewport の変化時に呼ぶ。

**3. 400〜700 行の build と helper method の木**(効き: 変更コスト・hot reload・const の効き、直し: 中)
- 証拠: ease_desk.dart:526-1205(679 行、`jsonEncode` 544-545 毎 build)、browser.dart:497-883(386 行)、inspector.dart の `_effect`/`_controlBody`/`_world`/`_transform`(各 120-170 行)。widget class 66 に対し helper `Widget _x()` が 100 超。
- 反する定規: StatefulWidget docs「大きな build は小さい widget に割る(method でなく widget)— 割ると変わらない部分は rebuild されず const になる」/ "Build methods should be pure"。
- 最小の直し: まず ease_desk と inspector の `_effect` を `const` 付き StatelessWidget に切り出す(引数は今の helper の引数そのまま)。`jsonEncode` の鍵は preset が変わった時だけ State で計算。

**4. drag→preview→commit の状態機械 4 写し**(効き: 直しの伝播・数値欄の挙動の揺れ、直し: 中)
- 証拠: panel_controls.dart:655-668 / 1144-1155 / 1466-1477、gradient_inspector.dart:40-62(`_end`・`_ending`・`_queue.finish`・`WidgetsBindingObserver` 2 override が同型)。同 file 内重複 46 窓。
- 反する定規: AGENTS「1 つ直せば同族全部が直る component にする」/ Flutter「composition over inheritance — 共通の振る舞いは 1 widget(または mixin)に」。
- 最小の直し: `EditorDragSession`(mixin on State + WidgetsBindingObserver、`begin/move/end(cancel)` と `EditorPreviewQueue` を持つ)1 つを panel_controls に置き、4 箇所を `with EditorDragSession` に置換。

**5. 色の生値と context 非依存の theme**(効き: 見た目の一貫性・切替の余地、直し: 小)
- 証拠: theme 外の `Color(0x…)` 37 箇所(timeline.dart:1473-1983 に 20)。`ThemeExtension` 0、`Theme.of(context)` 0。
- 反する定規: Use themes「色・文字は ThemeData/ThemeExtension から取り、widget に直書きしない」。
- 最小の直し: `EditorTheme` に `lane`/`laneAlt`/`ruler`/`keyInk` 等 8 名を足して 37 箇所を置換、`raw_dimension` と同じ lint を `Color(0x` にも 1 本(tool/motolii_lints に規則追加)。ThemeExtension 化は light を要求された時で良い。

番外(効きは小さいが今すぐ): `colors_shelf.dart:71-111` の貼り直し(29 error)。

## 残す物(良い部分)

- `DocumentSlice` の鍵指定購読と `take` の「変わった鍵だけ差し替え」(editor_session.dart:13-30, 154-166)— Flutter の ValueListenable 型そのもの。型付き model を足すなら**この上に**乗せる。
- Inspector の行ごとの `pulse`(inspector.dart:77, 186, `_Live` 2096)— 行単位の scoped rebuild。
- Timeline/Stage の `CustomPainter` + `shouldRepaint`(timeline 2003-2012、stage 1540)、行を widget で積まない設計。
- IOSurface → Metal → `FlutterTexture` のコピー無し経路(lib.rs:152-180、MainFlutterWindow.swift:199-235)。
- `EditorMetrics` + `raw_dimension` lint(metrics.dart、tool/motolii_lints)— 寸法の生値が 10 箇所まで抑えられている実績。色にも同じ型で。
- `ThemeData.dark().copyWith` 1 本(theme.dart:124-250)と `EditorTooltip`/`EditorButton`/`EditorSection`/`EditorChoice`/`EditorFold`/`EditorSwitch` の共通部品群。
- `_serial`/generation で命令を 1 本に直列化し、再生を Ticker で回す運転(editor_session.dart:367-390, 585-646)— polling 無し、同期 block 無し。
- `DocumentOperation` enum で op 名を 1 表に(protocol.dart)。
- widget test 60 file 9,419 行。

## Sources

- Flutter Performance best practices — https://docs.flutter.dev/perf/best-practices (build を軽く・rebuild を局所に・RepaintBoundary・`child` 引数・不透明度/クリップの節約)
- Flutter rendering performance / UI performance profiling — https://docs.flutter.dev/perf/rendering-performance 、https://docs.flutter.dev/perf/ui-performance
- StatefulWidget「Performance considerations」(木は widget に割る、const、child を渡す)— https://api.flutter.dev/flutter/widgets/StatefulWidget-class.html
- Architecture guide(MVVM、UI 層は ViewModel の値を読む)— https://docs.flutter.dev/app-architecture/guide 、recommendations https://docs.flutter.dev/app-architecture/recommendations
- State management(ChangeNotifier / ValueListenable、ListenableBuilder)— https://docs.flutter.dev/data-and-backend/state-mgmt/simple 、https://docs.flutter.dev/get-started/fundamentals/state-management
- Long lists: ListView.builder — https://docs.flutter.dev/cookbook/lists/long-lists
- RepaintBoundary — https://api.flutter.dev/flutter/widgets/RepaintBoundary-class.html
- CustomPainter(shouldRepaint / isComplex)— https://api.flutter.dev/flutter/rendering/CustomPainter-class.html
- Texture widget — https://api.flutter.dev/flutter/widgets/Texture-class.html
- Use themes / ThemeExtension — https://docs.flutter.dev/cookbook/design/themes 、https://api.flutter.dev/flutter/material/ThemeExtension-class.html
- Platform channels(codec、message size、非同期)— https://docs.flutter.dev/platform-integration/platform-channels
- macOS C interop(dart:ffi)— https://docs.flutter.dev/platform-integration/macos/c-interop
- 計測: `.tools/flutter/bin/flutter analyze --no-pub`(2026-09-19、30 issues)、grep/python による行・窓の集計(本文の数値)。

## Flutter の非効率の直し(裏)(2026-09-19、同日)

見た目と操作は変えない効率だけの lane。行番号は直した後の file。gate は各段で `flutter analyze` 0 件 + `flutter test` に**新しい失敗が無いこと**。着手時点(貼り間違いを直しただけの HEAD)で 8 件が既に落ちていて(下の「触っていない物」)、その集合は最後まで同じ。

### 0. 貼り間違い
- `lib/panels/browser/colors_shelf.dart:69-111` — `classification()` の三項の途中に貼り込まれていた `groups()`/`tagsOf()`/`valueOf()` を三項の後ろへ。`media_shelf.dart:2` の不要 import も削除。`flutter analyze`: 30 件 → **0 件**。

### 1. 毎コマの JSON と Dart 側の全コピー(Top 1)
- **軽い status の `liveLayers` は選択中の層だけ行(値・効果)を持つ**。他の層は今の姿(枠・位置)だけ: `native/src/snapshot.rs:262-270`(`wanted` = `!live || selected || selected_ids`、外れた層は `{"id"} + overlay_geometry`)。Inspector が読むのは選択層、Stage の当たり判定は枠だけで足りるので、Swift・request の形は触っていない(「要求」は選択そのもの)。全層ぶんの行数は変えない(`port.rs:420` の試験もそのまま)。
  - 計測(`snapshot.rs` の試験 `playback_values_match_full_status_without_keyframe_metadata` が出力): 2 層・効果 1 つの軽い status、全層行 **8,607 bytes → 選択 1 層 4,790 bytes**(1 コマあたり)。層が増えるほど差は広がる(選ばれていない層は 1 行 ≈ 0.3 KB、選ばれた層は ≈ 4 KB)。
- **`EditorSession.map/maps` はコピーしない view**(`lib/session/editor_session.dart:374-383`)。返信は `_accept` で 1 回だけ型を揃え(`typed`、338-372、`map(typed(reply))` 391)、以後 `map()`/`maps()` は渡された物をそのまま返す。型が狭い literal(test の `List<Map<String,Object>>`)は今まで通りコピーするので、読み手の `orElse` は壊れない。API の形は同じ、呼び出し 130 箇所は無変更。
  - mutate していた 2 箇所だけ `.toList()` を足した: `lib/panels/timeline_layout.dart:107`、`lib/panels/ease_desk.dart:196`。
- **`liveLayers()` は (document, frame) につき 1 回**(`editor_session.dart:268-286`、identity で memo)。Stage(`stage.dart:38`)と Inspector(`inspector.dart:139`)が同じ list を読む。
- Inspector: 「他の選択層」を status ごとに 1 回だけ集め(`inspector.dart:76,152`)、行ごとの `_stampRow`(181)は全層でなくその list を回す。
- Timeline `frameMoved`: 端の計算は `_timeline` slice が動いた時だけ(`timeline.dart:172-173`、`_relane` で無効化 974)。再生中の毎コマは cache を読むだけ。
- 証拠: `test/live_layers_test.dart` に 1 本追加 — `maps()`/`map()`/`typed()` が identity を返す、`liveLayers()` が同じ (document, frame) で同じ list、触られなかった層は document の object そのもの、overlay が document へ書かない。
- 残した物: `DocumentSlice` の鍵購読、`take`、`_serial`/generation(監査「残す物」)。
- `cargo build -p motolii-ui`: 完走(3m00s、warning 38 は既存)。`cargo test -p motolii-ui --lib playback_values_match`: 1 passed。

### 2. rebuild の範囲(Top 2)
- Stage `stage.dart:1083-1145`: `LayoutBuilder → Focus → Listener → ClipRect → ColoredBox` は layout ごとに 1 回、5 本 merge の `AnimatedBuilder`(1133)の中は `_picture()`(1221)= 絵と重ね描きの `Stack` だけ。`Texture`・`_StageOverlay` の `CustomPaint`・市松は `RepaintBoundary`(1259, 1280, 1293)。`_syncWindow` の postFrameCallback は build ごとでなく、読む物(origin・scale・viewport・shown・capability)が変わった時だけ(`_queueWindowSync` 94)。
- Timeline `timeline.dart:1020-1040`: 目盛りの `GestureDetector`/`SizedBox` は listenable の外、`frame`/`_timeline`/`scrolled` の merge が建て直すのは `RepaintBoundary(CustomPaint)` だけ(1037)。レーン(`timeline-lanes`)と縮図の `CustomPaint` も `RepaintBoundary`。`markers` は §1 で view になったので build ごとの List 生成は消えた。
- Window: `WorkspaceLayout` を `ChangeNotifier` に(`lib/workspace/layout.dart:157`)、show/close/move が notify。`editor_window.dart:483` の `ListenableBuilder(listenable: workspace)` が `WorkspaceView` だけを建て直す。panel の show/close/move の root `setState` 9 箇所は消えた(残る `setState` は `ready`・`sheet`・Reset layout)。
- 計測(`test/panel_layout_cost_test.dart` が出力する LAYOUT CALLS、15 層、直す前 → 後): Window `update 23 builds 214` → `22 / 205`、Stage `builds 32` → `26`、Timeline `builds 115` → `112`。Inspector は同じ(93、この段の対象外)。選択変更 1 回の layout `258` → `257`(予算 40 は依然超過、既存の失敗)。

### 3. drag→preview→commit の状態機械 4 写し(項目 4)
- `mixin EditorDragSession<T, W>`(`lib/foundation/panel_controls.dart:1097-1165`、`EditorPreviewQueue` の隣)が 1 本。`beginDrag`/`queue.add`/`endDrag(cancel)`、`WidgetsBindingObserver` の 2 override、dispose 時の cancel を持つ。control が書くのは `sendPreview`/`commitDrag`/`cancelDrag` と hook(`dragStopped`/`dragSettled`/`discardsPreview`/`watchesWindow`)だけ。
- 置換: `_EditorNumericFieldState`(444、`watchesWindow => false`、settle timer と pointer の後始末は `dragStopped`)、`_EditorDialState`(1199)、`_EditorPadState`(1498)、`_GradientInspectorState`(`gradient_inspector.dart:31`、drop は `discardsPreview` + `cancelDrag`)。4 つの `_end` と 3 組の observer override は削除。
- 証拠: `test/drag_session_test.dart`(新規 5 本)— preview→commit で飛行中の最後の値が勝つ、cancel は待ち値を捨てる、discard は cancel 経路、窓を失うと cancel(opt-out は登録しない)、unmount 中の cancel。既存の数値欄・ダイヤル・パッド・グラデーションの test はそのまま通る。

### 4. 色の生値(項目 5)
- `EditorInk extends ThemeExtension<EditorInk>`(`lib/foundation/theme.dart:14-140`、`static const dark` 54、`EditorInk.of(context)`)を `ThemeData` に登録(268)。token 16: `laneGround lane laneAlt grid gridMinor tick tickMinor headerInk`(timeline)、`camera`(Stage/Depth)、`focusRing`(pane)、`easePaper easeInk easeTime`、`checkerLight checkerDark`、`collectionColors`(7 色)。値は全て元の literal のまま。
- painter は `ink` を受け取る(既定 `EditorInk.dark`、建てる widget が `EditorInk.of(context)` を渡す、`shouldRepaint` に `ink` を足した): `_TimelinePainter`(`timeline.dart:1415`)、`_StageOverlay`(`stage.dart:1376`)、`_DepthGrid`(`depth_desk.dart:314`)、`CheckerPainter`(`panel_controls.dart:1881`)。timeline の `0xff242424`(6)は既存 `EditorTheme.line`、`0xffaedce8`(2)は既存 `EditorTheme.select` と同値だったのでそちらへ。Stage の `Color(0x00000000)` は `Colors.transparent`。
- theme 外の `Color(0x…)`: **37 → 1**(`color_field.dart:45` の `Color(0xff000000 | n)` は 16 進の解析で token ではない、lint も通す)。
- lint `raw_color`(`tool/motolii_lints/lib/src/raw_color.dart`、`raw_dimension` と同型、`main.dart` に登録、`bin/check.dart` にも同じ収集を足して `raw_color: clean/N`)。試験 `test/raw_color_test.dart` 3 本。`dart test`: 9 passed、`dart run bin/check.dart lib`: `raw_dimension: clean` / `raw_color: clean`。
- 監査の「≈8 token」より多い 16 になったのは値を変えないため(tick 2 種・lane 2 種を丸めない)。`BrowserLibrary.collectionColors` は getter で `EditorInk.dark` を返す(build 3 箇所の context 有無を確かめずに済ませた、context 版へ寄せるのは次)。

### 5. 長い build(項目 3)— 最小だけ
- `ease_desk.dart:525,549`: preset の `jsonEncode` を `Expando` で preset object ごとに 1 回(§1 で `maps()` が同じ object を返すようになったので効く)。`shapeKey` の 1 回は残る。
- **やっていない**: ease_desk build(679 行)・browser build(386 行)・inspector `_effect`/`_controlBody`/`_world`/`_transform` の `const` StatelessWidget への切り出し。見た目の合否と同じ file を触る他 lane に掛かるので、別便。

### 触っていない物・注意
- 着手時点で落ちていた 8 件(HEAD は compile 不能だったので走っていなかった): `panel_layout_cost_test`(layout cost 3/15 層 = 予算超過、選択変更 258 > 40、棚 500 の click 208 > 100)、`ease_interaction_test`(legacy handles)、`desk_workspace_test`(desk drafts)、`visual_selection_test`(gradient modes: Colors placement)、`protocol` の「document operations agree with native」。集合は 0〜5 の全段で同一。直すのは別便(UX と予算の裁定が要る)。
- 触っていない: `ui/native/src/editor/script/*`、`crates/**`、Swift。`raw_dimension.dart`/`use_metric.dart` は `dart format` が触ったので戻した。
- 利用者が目で見る物: Stage(再生・zoom・pan・Stage tab の窓送り)、Timeline 目盛りと縮図の追従、panel の show/close/move、数値欄・ダイヤル・パッド・グラデーション stop の drag と ESC/窓外し、Ease desk の preset 選択色、Browser の collection 色、Depth desk のカメラ線。値は同じはずだが合否は利用者。

### 数字(まとめ)
| 物 | 前 | 後 |
|---|---|---|
| `flutter analyze` | 30 件(error 29) | 0 件 |
| `flutter test` | 走らない(compile 不能)→ 貼り直し後 `+145 -8` | `+151 -8`(新規 6 本、失敗集合は同じ) |
| 軽い status(2 層) | 8,607 bytes | 4,790 bytes(選択 1 層) |
| `maps()` の Map コピー / 呼び出し | 毎回(N 層 × 行) | 0(型付き view) |
| `liveLayers()` / コマ | Stage + Inspector で 2 回、毎回全層の overlay | 1 回、選択層だけ overlay |
| LAYOUT CALLS 15 層 Window/Stage/Timeline builds | 214 / 32 / 115 | 205 / 26 / 112 |
| drag 状態機械 | 4 写し | mixin 1 |
| theme 外 `Color(0x…)` | 37 | 1(解析、lint 通過) |
| `RepaintBoundary` | 0 | 6 |

## build の切り出し(裏)(2026-09-19、同日、前段 §5 の続き)

見た目と操作は変えない。定規は Performance best practices「rebuild は木の小さい部分に閉じ込める」「const constructor」「小さい widget に割る(method でなく widget)」。規則: 不変の入力だけの helper は `const` 付き StatelessWidget、listenable を読む物はその listenable だけを購読する widget(`DocumentSlice` の鍵購読はそのまま)。test が key/type で探す物は key の位置を変えていない。行番号は直した後の file。gate は各 file で `flutter analyze` 0 件 + `flutter test` の失敗集合が前段の 8 件と同一(6 回とも `+151 -8`、集合同一)。

### ease_desk.dart(build 684 → 418 行)
- `jsonEncode` は build から出た: `_keyOf`(195-196、`Expando` で shape object ごとに 1 回、`_shape` は差し替えのみで in-place 変更が無い)を preset と `_shape` の両方に使い、`mixed`(区間の shape が 2 種以上か)は `_segments` の派生時に 1 回(`_mixedCache` 168-190)。build 内の `jsonEncode` は 0(前: 区間ごと + `shapeKey` 1 回 / build)。
- 切り出し 7 つ(全て `const` constructor): `_PresetTile`(968、preset・選択・hover・focus・幅高さ・`free` — 触った tile だけが変わる)、`_EaseParamField`(1064、key `'$kind:$name'` は中の `EditorNumericField` に残した — `ease_interaction_test` が `Random:seed` を hitTest する)、`_EaseInfo`(1124、名前・意味・motion 帯、motion だけ `AnimatedBuilder`)、`_EaseTargetRow`(1227、`frame` の `ValueListenable` だけ購読)、`_EaseRail`(1287、同)、`_OvershootToggle`(1328)、`_ApplyButton`(1371)。
- 残る 418 行: 曲線の `Listener`(pointer の状態機械が State に居る)と preset grid の `Focus.onKeyEvent`(`_focused` を setState)。次は grid を `_PresetGrid` StatefulWidget にして label の高さ測定(`_labelKey`)ごと移す。

### browser.dart(build 386 → 252 行)
- `_BrowserSearchBar`(987、検索欄・道具・filter 印・view 切替。key `browser:filters-toggle` は中の `InkWell` のまま)、`_BrowserRail`(1080、`RailCollections` は親が組んで `collections:` で渡す)、`_BrowserRailTab`(1130、key `browser:rail-tab` のまま)、`_NoMatches`(1176、`const _NoMatches()`)。
- 残る 252 行: grid の `LayoutBuilder` が `columns`/`tileWidth` を State に書く(card の名前幅がそれを読む)ので、そこは動かしていない。`FilterView` の 6 個の callback は shelf/library を跨ぐので次。

### inspector.dart(`_effect` 169 → 136、`_controlBody` 150 → 141 行)
- `_HeadGlyph`(2070、`_headGlyph` はこれへの 1 行委譲)、`_CellLabel`(2094、`_controlBody`・`_pointBody` の 2 箇所)、`_EffectGrip`(2114、中身は丸ごと `const`)、`_AdvancedFold`(2139、`_advancedOpen` だけを `Picked` で購読、中身は開いた時だけ `builder()` で建てる — 前と同じ遅延)、`_SeedRoll`(2169)、`_SpaceChoice`(2188、`_world` の 2D/2.5D/3D)。
- **やっていない**: `_transform`(1043-)と `_world`(1168-)の丸ごと。`_well`/`_slot`/`_line` が `_wellWidth`/`_wordWidth`(panel 幅から `_fit` が決める)と `c` を読むので、切るなら先に `_InspectorGrid`(幅 3 つ)を `InheritedWidget` にして row helper を top-level に出す。`_controlBody` の残り 141 行は `_Kind` ごとの switch で、`_write`/`_begin`/`_finish` の drag 状態機械が State に居る。

### stage.dart(`_picture` 129 → 110 行)
- `_StageTexture`(1338、`textureIds` の `ValueListenable` だけ購読、「No rendered texture」は `const`)。`_picture` の残りは overlay painter の引数(全部 State の幾何)。

### timeline.dart(build 246 → 207 行)
- `_TimelineScrollStrip`(1357、`_timeline` だけ購読、幅は `measure()` で今の値を読む)、`_NameColumnGrip`(1402、drag の 4 callback)。目盛りとレーンの merge 購読は前段 §2 の形のまま。

### panel_controls.dart(`EditorSection.build` 70 → 36 行)
- `_SectionHead`(1437、`const`、名前・畳み印・leading/trailing)。

### 数字
| file | 最長 build/helper(行) 前 → 後 | `const` widget 生成 前 → 後 |
|---|---|---|
| ease_desk.dart | 684 → 418 | 38 → 45 |
| browser.dart | 386 → 252 | 25 → 30 |
| inspector.dart | 169 (`_effect`) → 141 (`_controlBody`) | 36 → 43 |
| stage.dart | 129 (`_picture`) → 110 | 16 → 17 |
| timeline.dart | 246 → 207 | 11 → 13 |
| panel_controls.dart | 70 → 61(`EditorSection` は 36) | 43 → 44 |

計測は `Widget x(` から括弧が閉じるまでの行数、`const X(` の個数(scratchpad の measure.py)。

`test/panel_layout_cost_test.dart` LAYOUT CALLS 15 層(前 → 後): Window `update 22 builds 205` → `22 / 207`、Stage `builds 26` → `27`、Inspector `93` → `93`、`whole` は全て同じ(641 / 68 / 424)。+1 ずつは `_StageTexture`・`_SectionHead` の 1 段(widget が 1 つ増えれば build は 1 回増える、layout は増えない)。選択変更 1 回の layout `257`(前後同じ、予算 40 は依然超過)、棚 500 の click `208`(同)。失敗 8 件の集合は前段と同一。

### 触っていない物
- test は 1 本も変えていない(type 名を探す test が無かった)。見た目の差は無いはずだが、合否は利用者: Ease desk の preset tile(hover/focus 枠)、数値欄の hit、Browser の検索欄と rail、Inspector の効果の頭(掴み・目・…)と advanced の畳み、Stage の絵、Timeline の横 scrollbar と名前列の掴み、section の頭。
- `flutter analyze` は 6 回とも `No issues found!`。

## Material からの離脱(裏)(2026-09-19、同日、desktop-kits の作業指示「kit は入れず flutter/widgets + 自前の葉、Lumit の型」)

見た目と操作は変えない、構造と速さだけの lane。`lib/` の `package:flutter/material.dart` は **37 file → 0**。root は `WidgetsApp`、葉は `foundation/leaves.dart` の自前 widget で、Material が theme の下で描いていた寸法・色・段差をそのまま写した(数値は着手前に `MaterialApp(theme: EditorTheme.data)` の下で実測: TextButton 19×12・padding 4、FilledButton(compact)min 44×16、IconButton = iconSize(無指定は IconTheme の 14)、TextField の文字 11 px w500、caret 幅 2 / macOS は角 2 の accent、Divider 0 = hairline、Scrollbar 6 px 角無し 白 30/65/75%、InkWell hover 白 3.9% / pressed `hover` / focus 白 12%、TextButton hover fg 8% / pressed 10%、tooltip 白 90% 黒 12 px 角 4 高 24)。行番号は直した後の file。gate は各段 `flutter analyze` 0 件 + `flutter test` の失敗集合が前段の 8 件と同一。

### 1. root — `MaterialApp` → `WidgetsApp`
- `lib/app/editor_app.dart:28-56`: `ScrollConfiguration(EditorScrollBehavior)` → `WidgetsApp(color: app, textStyle: EditorTheme.text, pageRouteBuilder: PageRouteBuilder)`、`builder` で `IconTheme(EditorTheme.icon)` → `DefaultSelectionStyle(caret, selection)` → `EditorScale` → `EditorScaledViewport` → `EditorLook(tooltips: false)`(= 旧 `TooltipVisibility(false)`)。Navigator/Overlay/Localizations/Shortcuts/Actions/MediaQuery は WidgetsApp が持つ。
- `EditorScrollBehavior extends ScrollBehavior`(`:62-91`): drag 全種・Clamping・overscroll 無し は前と同じ。`buildScrollbar` は Material と同じ条件(縦・desktop だけ)で `EditorScrollbar` を付ける。
- theme の器: `lib/foundation/theme.dart:152` `EditorLook`(InheritedWidget、`ink` と `tooltips`)。`EditorInk` は `ThemeExtension` をやめて素の class(`:14`)、`EditorInk.of(context)` は `EditorLook.of(context).ink`(無ければ `dark`)。`ThemeData`・`MenuStyle`・`ButtonStyle` の 100 行は消え、値だけ token に: `white/black/clear`(`:242`)、`scrim/scrimLight`、`hoverWash/focusWash`(`:253`)、`inkDisabled/washDisabled`(`:257`)、`scrollThumb*`(`:262`)、`tickActive/tickInactive`(`:267`)、`tooltip`(`:270`)、`caret/selection`(`:273`)、`text`(旧 bodyMedium: inherit false・11・w400・ink・alphabetic、`:278`)、`icon`(`:285`)、`menuPadding/menuMinWidth/menuRowPadding`(`:290`)。値は全て実測の写し。
- `Colors.*` 64 箇所 → `EditorTheme.white/black/clear/scrim/scrimLight`(`raw_color` lint は theme.dart を通す)。`Icons.*` 125 箇所(95 名)→ `lib/foundation/glyphs.dart` `Glyph.*`(SDK の `material/icons.dart` から codepoint と `matchTextDirection` をそのまま写した 95 行、font は pubspec の `uses-material-design: true` がこれまで通り同梱)。

### 2. 葉 — `lib/foundation/leaves.dart`(1,400 行、`flutter/widgets` のみ)
| 旧(件数) | 新 | 場所 | 写した物 |
|---|---|---|---|
| `InkWell` 25 | `EditorPress` | leaves.dart:21 | hover 白 3.9% / pressed `hover` / focus 白 12% を **子の下**に塗る(Material の ink と同じ層。不透明な子が覆えば見えないのも同じ)、`click` cursor、`borderRadius`、Enter/Space の `ActivateIntent`、`onHover`/`onFocusChange`/`onDoubleTap`/`onSecondaryTap`、tap semantics |
| `IconButton` 16 | `EditorIconButton` | :169 | ちょうど `iconSize`(無指定は IconTheme の 14)、hover/pressed に `EditorTheme.hover`、無効は `disabledInk`、`isSelected`/`tooltip` |
| `TextButton` 14 / `FilledButton` 2 | `EditorTextButton` | :226 | min 10×12・pad h4・`line` 枠・fg accent(compact density が min から 8 引いた後の実測)。`background/foreground/border/radius/padding/minimumSize/textStyle` で `styleFrom` の 5 箇所(EditorButton・notes_desk 3・ease_desk 2・web_panel の `.icon`)を写した |
| `Divider` 12 | `EditorRule` | :349 | `Border(bottom: width: thickness)` を `height` の中央に(0 = hairline) |
| `Scrollbar` 3 + `MaterialScrollBehavior` の自動付与 | `EditorScrollbar`(RawScrollbar) | :377 | 6 px・角無し・crossAxisMargin 2・最短 48・白 30/65/75%・fade 300/600 ms |
| `Slider` 4 | `EditorSlider` | :425 | track 2 px 丸端(`muted`/`line`)、thumb 5 px ink + drawShadow 1(押下 6)、`divisions` の snap と目盛(3 px 以上空く時だけ)、discrete の drag 中だけ値の札 |
| `TextField` 9 | `EditorTextField`(EditableText + `TextSelectionGestureDetectorBuilder`) | :627 | 11 px w500 ink、caret 幅 2 角 2 accent、selection accent 40%、text cursor、`hint`(muted、空の間だけ)、`padding`/`prefix`(旧 contentPadding/prefixIcon)、`expands`/`onTap`/`onSubmitted`、右クリックの Cut/Copy/Paste/Select all は `EditorMenuSheet` |
| `Tooltip` 63(`EditorTooltip` 経由) | `EditorTooltip` → `RawTooltip`(3.47 の widgets) | theme.dart:301 / leaves.dart:841 | `EditorLook.tooltips` が off なら widget を立てない(前と同じ)。on なら白 90%・黒 12 px・角 4・高 24、`ignorePointer: true`(Material と同じ。無いと sheet が pointer を奪って hover が切れる — `browser_tile_test` が捕まえた) |
| `MenuAnchor` 5 / `MenuItemButton` / `DropdownButton` 1 | `EditorMenuSheet`(:875)+ `EditorMenuRow`(:1018)+ `EditorMenuAnchor`(OverlayPortal、:1088)+ `EditorChoice`(:1340) | | sheet `menu`/`menuEdge`/pad v2/min 112(context menu は 0)、行 20・pad h8・hover/focus は `select`+`selectInk`・無効 `disabledInk`、anchor の下・入らなければ上・窓内に押し込む、Esc / 外 click(`consumeOutsideTap`)/ ↑↓ / Enter、閉じた後は前の focus へ。`showEditorMenu`(theme.dart:389)は OverlayEntry のまま中身だけ差し替え |
| `showDialog` + `AlertDialog` 1 | `showEditorDialog` + `EditorDialog` | :1179 / :1195 | scrim 黒 54%、fade 150 ms easeOut、`panel` に `border` 枠、title 13 / body 11、pad 24、min 280、actions 右寄せ 8 間隔 |
| `CircularProgressIndicator` 1 | `EditorSpinner` | :1287 | 36 px・4 px・accent の弧。**回り方は M3 の式を写していない**(読み込み中だけ出る) |
| `Material(` 8 / `Scaffold` 1 | `ColoredBox` / `DecoratedBox` / `PhysicalModel(elevation: 4)`(sheet、editor_window.dart:535)/ drag feedback は `DefaultTextStyle(EditorTheme.text)` | | Material が配っていた DefaultTextStyle は root の `textStyle` が同値で配る |

数値欄の drag→preview→commit、`EditorDragSession`、`TrackStyle`、dial/pad/lamp/switch/anchor grid、`DocumentSlice`/`Picked`、painter 8 本は無傷(gui-existing-devices の「守る順 1〜5」)。

### 3. import の入れ替え
- 33 file を機械的に(`material.dart` → `widgets.dart`、`Icons.` → `Glyph.`、`InkWell(`→`EditorPress(` …)、手で写したのは editor_window(dialog・Scaffold・sheet)、notes_desk(TextButton 3・TextField・Material)、ease_desk(`_OvershootToggle`・`_ApplyButton`)、web_panel(`TextButton.icon`)、browser/filters/rich_text_editor の `InputDecoration` 7 箇所、blend_panel/desk/workspace_view/media_shelf/tile の `Material`。
- `lib/` の Material 語彙の残り: **0**(`grep -E "InkWell|IconButton|TextButton|MenuAnchor|ThemeData|Icons\.|Colors\."` は glyphs.dart のコメント以外に無し)。
- test は触らない範囲を最小に: `MaterialApp`/`Scaffold` の器はそのまま、`theme: EditorTheme.data` 37 箇所 → `test/support/editor_test_theme.dart` の `editorTestTheme`(旧 ThemeData の text/icon/density だけの shim、24 file)。finder は `TextField`→`EditorTextField`、`MenuItemButton`→`EditorMenuRow`、`Divider`→`EditorRule`、`Tooltip`→`EditorTooltip`、`InkWell`→`EditorPress`、`TextButton`→`EditorTextButton`(14 file)。`hover_and_tap_target_test` は Material の theme を測る test だったので葉の同じ数値を測る形に書き直し(+1 本: tooltip が on なら hover で出る)。相対 import(`../lib/…`)の test に `package:` で葉を足すと型が二重になる(`EditorChoice` が両方から)ので、その file の流儀に合わせた。

### 4. split pane — 自前を残す
`workspace/workspace_view.dart:73-133` `_Split` は 60 行で、`DockNode.firstExtent/drag`(layout.dart:56-76)の「fixed は catalog の extent + offset、elastic だけ ratio」をそのまま読み、仕切り drag は node の State だけを建て直す(gui-existing-devices K-6「仕切り drag は node の状態(panel は build されない)」)。multi_split_view は controller 駆動で drag ごとに MultiSplitView 全体を build し直し、`Area(size/flex/min/max)` へ DockNode を写す層が要る。得るのは同じ 4 px の線だけなので入れない。pubspec は無変更。

### 5. 段ばしごの pill(Lumit value_field の型、値だけ写した)
- `EditorNumericField` は drag 中に `×rung` を tooltip の message に出していたが、tooltip は全 off なので誰にも見えていなかった。`lib/foundation/panel_controls.dart:392` `_showRung`、`:738` `OverlayPortal` + `CompositedTransformFollower`(well の上 4 px、pointer は通す)、`:858` `_RungPill`: 4 段 `×10 ×1 ×0.1 ×0.01` を tooltip の sheet(白 90%・黒 12 px・角 4)で、今の段だけ黒 w600・他は `disabledInk`。**擦っていて ×1 以外の段に居る間だけ**出る(Lumit は擦っている間ずっと 4 段を出す。×1 の時に出さないのは「hover で何も出ない」の裁定に寄せた)。
- decision-index に足す 1 行(未記入): `2026-09-19 | 数値欄の段ばしごは擦って段を変えた間だけ 4 段の pill を well の上に出す(Lumit value_field の型、tooltip 全 off でも段が読める) | panel_controls.dart _RungPill`

### 6. lint `material_import`
- `tool/motolii_lints/lib/src/material_import.dart:16`(`raw_color` と同型、`ImportDirective` の uri が `package:flutter/material.dart` / `cupertino.dart` なら warning、test dir は除外)、`main.dart:19` に登録、`bin/check.dart:96` に `material_import: clean/N`。試験 `test/material_import_test.dart` 2 本(stand-in の flutter package を `newPackage` で置く)。`dart test`: **11 passed**、`dart run bin/check.dart lib`: `raw_dimension: clean / raw_color: clean / material_import: clean`。

### 数字(前 → 後)
| 物 | 前 | 後 |
|---|---|---|
| `lib/` の `material.dart` import | 37 file | **0**(test は 55 file が `MaterialApp` の器で残る) |
| Material の葉の呼び出し(InkWell/IconButton/TextButton/FilledButton/Slider/Scrollbar/TextField/MenuAnchor/DropdownButton/showDialog/Divider/Material) | 25/16/14/2/4/3/9/5/1/1/12/8 | 0(EditorPress 27 / EditorIconButton 15 / EditorTextButton 11 / EditorSlider 3 / EditorScrollbar 4 / EditorTextField 9 / EditorRule 4) |
| `Icons.` / `Colors.` | 125 / 64 | 0 / 0(`Glyph.` 125、`EditorTheme.white/black/clear` 29) |
| `flutter analyze` | No issues found | No issues found(6 段とも) |
| `flutter test` | `+151 -8`、25.6 s | `+152 -8`(失敗集合同一、+1 は tooltip on の test)、23.2 s |
| LAYOUT CALLS 15 層 Window `builds` / `whole` | 207 / 641 | **191** / 642 |
| 同 棚 `whole` Create / Media / Effects / Colors / Fonts | 292 / 278 / 294 / 356 / 211 | **265 / 246 / 267 / 298 / 179** |
| 同 Inspector `whole` / Stage | 424 / 68 | 414 / 68 |
| 選択変更 1 回の layout(予算 40、既存の失敗) | 257 | **165** |
| 棚 500 の click の layout(予算 100、既存の失敗) | 208 | 199 |
| `flutter build macos --debug`(warm、build/ は消していない — 消すと利用者の窓 `motolii_stage5.app` が消える) | — | 38.2 s、`✓ Built`(warm。cold は測っていない) |

減った widget は `Material`/`Ink`/`InkResponse`/`_InputDecorator`/`ButtonStyleButton` の皮(Semantics 178 → 減、`RawGestureDetector` が代わりに上位に出る)。

### 見た目が変わり得る所(利用者が目で見る物)
- **押す面の段差**: InkWell の hover/pressed は 200 ms で滲んでいた。`EditorPress` は即時(値は同じ)。IconButton・TextButton・menu の行も同じ。
- **TextButton の文字色**: 無指定は accent(M3 primary)のまま。editor_window の Save changes? の 3 つ、notes の page tab / Import previous / reference block、web の Open in browser。
- **TextField**: 文字が w500(Material の実測どおり)、caret 角 2、selection accent 40%、右クリック menu は自前の sheet(文言 Cut/Copy/Paste/Select all)。手掛け(handle)は出さない(desktop と同じ)。IME 合成の下線・spell check は EditableText の既定。
- **Slider**(Browser の tile size、Colors の alpha): 値の札は M3 の drop 形でなく丸角の accent 札、tick の色 黒/白 38%。
- **Scrollbar**: hover の色変化が即時(M3 は 200 ms)。drag 色は thumb の上で押した時だけ。
- **Menu**: 位置の反転は「下に入らなければ上、上にも入らなければ窓内へ押す」。MenuAnchor の細かな offset(alignmentOffset 0)と同じはず。開いた menu は FocusScope を取り、閉じると前の focus へ戻す。
- **Dialog**: fade 150 ms、scrim 54%、min 幅 280、pad 24 — M3 AlertDialog の寸法を写したが、title/actions の細部(actions の OverflowBar)は Row。
- **Spinner**(読み込み中の 1 秒だけ): 弧の伸縮が M3 の式でない。
- **Tooltip**: 全 off なので見えない。Settings 等で on にした時は白 90% / 黒 12 px。
- **段ばしごの pill**(新): 数値欄を擦って上下に離すと well の上に `×10 ×1 ×0.1 ×0.01`。
- **sheet の影**: `PhysicalModel(elevation: 4)`。Material の elevation 4 と同じ影の式(kElevationToShadow)。

### 触っていない物
- `DockNode`/workspace.json、pubspec(依存は flutter のみ)、native、Swift、docs/decision-index.md(1 行は上に置いた)、`ui/build/`。
- test の器(`MaterialApp` + `Scaffold`)は Material のまま。器も widgets に寄せるなら `test/support` に 1 つ helper を置いて 86 箇所を差し替える便。
- 8 件の既存失敗(panel_layout_cost 4・ease_interaction・desk_workspace・visual_selection・module_contract)は前段と同じ集合。

## Stage の大きさを変えた瞬間に絵が潰れる(利用者、2026-09-19 昼)

現象: Stage(Front)の tab を伸縮すると、球などが一瞬潰れてすぐ戻る。
切り分け: native を隔離した実験 2 本(`snapshot.rs` の `a_sphere_keeps_its_roundness_in_a_tall_stage_window`、#[ignore]、GPU 10 分)で、窓と roi の縦横比が揃っていれば 3D の球も 2.5D の球も、縦長 400×900 でも横長 1200×400 でも 13 % でも出力と同じ丸さ(比 1.00)。投影は潰していない。
原因: 描く側は毎コマ、Rust が言う窓の大きさで IOSurface を新しく作る(MainFlutterWindow.swift の render)。Flutter の Texture は tab の矩形いっぱいに引き伸ばすので、tab を伸縮してから次のコマが届くまでの間、前の大きさの絵が新しい矩形に引き伸ばされる = 一瞬の潰れ。
直し(ui だけ、render は触らず): (1) host が envelope に `textureSizes`(view ごとの描いた px)を載せる (2) `EditorSession.textureSizes` (3) Stage の `_StageTexture` は `anchored`(Stage tab)の時、描かれた大きさで左上に置き、引き伸ばさない(Camera は出力の絵なので今まで通り一様に fit) (4) LayoutBuilder で大きさが変わった同じコマに `_syncWindow()` を送る(post-frame の 1〜2 コマの遅れを削る)。
確認: analyze 0、stage の test 通過(panel_layout_cost の 4 本は元からの予算超過)、実窓で divider を引いても cube は潰れず。
別件で残る: 環境(HDR)の空は roi を無視して窓の uv で描かれる(fork の `camera_ray_direction_from_screenuv`)。空も枠に従わせるか、Front では枠の中だけにするかは利用者の裁定待ち。

## FFI の橋(裏)(2026-09-19、同日、利用者「直せ」)

現象の骨: 毎コマの絵が Dart → JSON op → MethodChannel → Swift `worker` queue → Rust `render`(毎コマ新しい IOSurface)→ main → MethodChannel `documentChanged` → Dart rebuild → raster と **thread を 4 回跨いで**運ばれ、Fit / zoom / 伸縮の後 1〜3 コマは前の絵が見えていた。viewport を動画 player の texture のように運んでいたのが構造の欠陥。

**新しい毎コマの道(言葉で)**: Dart が `dart:ffi` で Rust を直接呼ぶ。`EditorSession.render()`(再生の Ticker と `needsRender` の全部)= `renderInfo`(FFI)→ view ごとに `motolii_probe_render`(FFI、Swift が作り置いた IOSurface へ)→ Rust が描き終えた直後に C callback `frame_ready(user, view, surface_id)` を**同じ thread で**呼ぶ → Swift がそれを `registry.textureFrameAvailable` に変える → `status`(FFI)→ notifier。全部が Dart の同じ関数呼び出しの中で終わるので、**そのコマの build が既に新しい窓と status を持ち、そのコマの raster が新しい絵を拾う**。Stage の `_syncWindow` は LayoutBuilder / build の中から `EditorSession.commandNow('stageWindow')` を同期で送り、返ってきた「描けた」で `_drawnWindow` を持って `_picture` がその矩形に絵を置く。build の中では notifier を動かせない(木の外の widget を dirty に出来ない)ので、status の取り込みは `_deferred` に積んで microtask(そのコマの終わり)で流す。

**Swift に残る物**: dylib の dlopen と `motolii_probe_open`(複数窓の `ProbeSession` はそのまま)、view ごとの texture 登録と IOSurface(1 view に 2 枚を交互、大きさが変わった時だけ Dart が `ensureSurfaces` を 1 回 channel で頼む)、Rust の callback → `textureFrameAvailable`、窓の管理・drag & drop・閉じる確認・pane state、そして **副窓への `documentChanged` 配信**(main の Dart が返信の生 JSON を `broadcast` で 1 回渡す。副窓が無ければ呼ばない)。`vism/` の見張りは Swift が `effectsChanged` を main の Dart へ送り、Dart が `reloadEffects` を FFI で打つ。`context`(生の pointer)と `library`(path)は envelope に毎回載り、`open` で runtime が入れ替わっても Dart が追う。副窓は今まで通り channel(FFI は main だけ、`windowInfo.main == false` では結ばない)。

**thread の型**: Flutter 3.35 から macOS は UI thread と platform thread が既定で 1 本(https://docs.flutter.dev/release/breaking-changes/macos-windows-merged-threads 「Flutter 3.35 merges the UI and platform threads by default on macOS and Windows」、3.33.0-0.0.pre で着地)。本 repo は 3.47.2。実走の証拠: Rust の callback を受けた Swift の log `PROBE room=bridge frameReady thread=main merged-ui-platform=true`。従って Dart(FFI)と Swift(open / close / 副窓の request)は同じ main thread で runtime を触り、競合しない。Swift の `worker` queue は設定 file の読み書きだけに残し、runtime を触る `perform` は main で同期に走る(Mutex は要らない — もし将来 thread が分かれたら C ABI の中で包む)。

**数字(前 → 後)**
- thread の跨ぎ: 4(Dart→Swift main→worker→main→Dart)→ **0**(Dart の呼び出しの中で完結。IOSurface を作り直す時だけ channel 1 往復、大きさが変わった時だけ)
- 窓を変えてから、その窓で描かれた絵が出るまでのコマ数(kDebugMode の `PROBE room=stage-window verdict=matched lag=N`、Stage が `stageWindow` を頼んだコマと、頼んだ窓に等しい絵を持って build したコマの差):
  - zoom 150 %: `verdict=matched lag=0 frames path=ffi-same-frame window=1692x952 roi=[678.07, 381.41, 563.87, 317.18]`
  - zoom 50 %: `verdict=matched lag=0 frames path=ffi-same-frame window=1692x952 roi=[114.20, 64.24, 1691.60, 951.53]`
  - 起動時の最初の窓(surface が無い = 伸縮と同じ道: ensureSurfaces 1 往復 → render): `verdict=matched lag=0 frames path=status window=2059x1158 roi=[-1517.11, -853.37, 4954.21, 2786.74]`
  - 前(同日昼の切り分け): 1〜3 コマ前の絵が新しい矩形へ引き伸ばされていた
- 符号化: op 1 回に JSON 符号化 3 回・復号 3 回 → 符号化 1 回・復号 1 回(Dart ↔ Rust の JSON 文字列だけ。Swift の `JSONSerialization` と StandardCodec は毎コマの道から消えた)

**変えた file**
- `motolii/ui/native/src/lib.rs` — `FrameReady` 型、`motolii_probe_set_frame_ready(ctx, cb, user)`、`motolii_probe_render` が 0 を返す直前に呼ぶ。test `frame_ready_tests::the_frame_ready_signal_fires_once_per_render`(本物の IOSurface へ 2 回描いて 2 回、失敗した描画は呼ばない、外せば呼ばない)。`Cargo.toml` に dev-dependency `objc2-core-foundation`(lock に既にある版)
- `motolii/ui/lib/bridge/native_frames.dart`(新規)— `NativeFrames` の口と `FfiFrames`(手書きの `dart:ffi`、`DynamicLibrary.open(library)`、`malloc/free/strlen` は process から。codegen も package も足していない)
- `motolii/ui/lib/session/editor_session.dart` — `_frames` の結び(envelope の `context`/`library`)、`_requestNow`/`_renderNow`/`_ensureSurfaces`/`_broadcast`/`commandNow`、`_deferred`、Ticker は FFI なら同期に `_renderNow`、`effectsChanged`、`documentClosed` で frames を捨てる。channel の道は test 用にそのまま(`NativeBridge` は変えていない)
- `motolii/ui/lib/panels/stage.dart` — `_queueWindowSync` は `c.sameFrame` なら build の中で `_syncWindow`、`_syncWindow` は `commandNow`、`_drawnWindow`(f32 で戻る roi と 0.01 px で比べる `_sameWindow`)、kDebugMode の lag 計測 `_noteWindowLag`
- `motolii/ui/macos/Runner/MainFlutterWindow.swift` — `perform` は main で同期、`worker` は設定 file だけ、`surfaces`(view → 2 枚)、`frameReady`、`ensureSurfaces`、`broadcast`、`effectsChanged` の転送、envelope に `context`/`library`、`windowInfo` に `panelWindows`、`makeSurface` を切り出し。fallback の channel `render`(FFI が結べない時)は残る

**確認**: `flutter analyze` 0(`No issues found!`)、`flutter test` `+152 -8`(落ちる 8 本は元から: panel_layout_cost ×4・ease_interaction・desk_workspace・visual_selection・module_contract)、`cargo test -p motolii-ui --lib` は `every_example_builds_its_document` が元からの失敗、frame_ready の test は通過。実窓: 球を置いて Stage と Camera が両方描かれ、選択の枠も出る(利用者が触っている間の screenshot)。

**気付いた事**: GPU の test を並走させたまま app を動かすと、`re_renderer` の shader 見張り(`notify` の `FSEventStreamStart` → fseventsd への RPC)が **3 process 同時**で応答しなくなり、app の最初の描画が止まる(test process を止めた瞬間に動き出した)。これは main thread に載せたせいではなく machine の fseventsd の詰まり(app を止めた後の serial な test でも同じ RPC で止まった = この machine で断続的に起きる)。ただし今は最初の描画が Dart の thread なので、詰まると窓ごと固まる(前は worker が待つだけで窓は動いた)。`test` と `dev` を同時に回さない(記憶「レーンは 1 個ずつ」と同じ)。根は `re_renderer` の shader 見張り(dev の `load_shaders_from_disk` だけ)— 見張りを別 thread で遅らせて立てるのが筋。

**残る物**: (1) FFI の中身はまだ JSON 文字列(op も status も)。次は status を struct/バイナリで渡す(audit §1 の続き)。(2) IOSurface は Swift が作る(大きさが変わった時だけ channel 1 往復、計測は 0 コマ)。Rust が自分で作れば(`IOSurfaceRef::new`)その往復も消え、Swift は `IOSurfaceLookup` だけになる。(3) callback は仕様の `(user, view)` に `surface_id` を足した 3 引数(2 枚交互のどちらを出すか Swift が知る必要があった)。(4) 副窓(panel window)は絵を status の配信と一緒に受ける(main は callback で即)。(5) 環境(HDR)の空が roi を無視する件は別(前段の通り、裁定待ち)。(6) 実窓の divider drag は自分では動かせない(AppleScript は拒否)。伸縮の道の数字は「surface の無い最初の窓」で測った物。

## Timeline のガクつき(裏)(2026-09-19、同日、利用者「ガクガクする、何度も戻ってくる」)

**先に測った**。`kDebugMode` の計測を `_renderNow` の各段と Flutter の `addTimingsCallback` に入れ、`light-in-form.rrd`(利用者の保存 layout = Stage と Camera が**並んで両方見えている**)で 12 秒の再生を自動で回した。log は scratchpad の `jank/`。

### 計測(前)— 再生 12 秒、drawn tick 468 回、ms

| 段 | 中央 | p90 | 最大 |
|---|---|---|---|
| `_renderNow` 全体 | **10.16** | **49.31** | 59.23 |
| `tick`(FFI) | 0.05 | 0.06 | 0.14 |
| `renderInfo`(FFI) | 0.04 | 0.05 | 0.10 |
| `render` Camera(GPU) | **4.36** | **26.97** | 35.78 |
| `render` User = Stage(GPU) | **3.53** | **17.94** | 22.12 |
| `status`(FFI、軽い status 5.9 KB) | 1.62 | 3.01 | 4.67 |
| `jsonDecode` | 0.08 | 0.14 | 4.04 |
| `_accept`(notifier) | 0.34 | 0.50 | 0.93 |
| `_broadcast`(副窓 0) | 0.00 | 0.00 | 0.00 |

Flutter 自身のコマ(16.7 ms 超だけ記録、184 本): build 中央 **42.77**、raster 中央 **1.34**、全体 中央 **49.84**、最大 140.46。33 ms 超 108 本。
UI thread が `_renderNow` の中に居た時間 = **8,645 ms / 12 秒(≈ 72 %)**。

### 原因(1 行)

**再生の Ticker が UI thread の上で view 2 枚を同期に GPU 描画し(`motolii_probe_render` → `render_into` の `device.poll(wait_indefinitely)`、`lib.rs:203`)、1 コマあたり中央 10 ms・p90 49 ms を UI thread から奪うので、Timeline は塗る番が回って来ない — Timeline の paint ではない(raster 中央 1.3 ms)。**

潰した仮説 2 つ:
- 「Camera tab は隠れているのに描いている」— **違う**。利用者の保存 layout(`~/Library/Application Support/MotoliiStage5/layout.json`)は Stage と Camera を別々の pane に並べており、`Visibility.of` も両方 true。既定の dock(`layout.dart:142`、Camera は Stage の裏 tab)ならこの仮説は生きる。
- 「status の JSON が重い」— **違う**。再生中の軽い status は 5.9 KB・1.62 ms、decode 0.08 ms。重いのは停止後の全部入り(140 KB、status 7.1 ms + decode 1.3 ms)だけで、再生の道には乗っていない。

「Timeline を閉じた時の数字」は**測れていない**。dock の tab は実窓の手でしか閉じられず(記憶「実窓操作は拒否される」)、代わりに raster の中央 1.3–2.3 ms を Timeline の塗り代として置く。

### 直した所(一番重い所だけ)

時計は**コマ単位に丸めて**進むので、表示コマ(60〜120 Hz)の多くは既に画面に出ている絵を頼んでいた。実測: 474 回の再生描画のうち **179 回(38 %)が直前と同じ document コマ**。Rust には既にその審判(`image_key`)があるのに、静かな `tick` は `{"ok":true}` としか答えていなかった。

- `motolii/ui/native/src/lib.rs:296` — 静かな `tick`/`seek` の返信に `needsRender`(= `image_key` が動いたか)を載せる。
- `motolii/ui/lib/session/editor_session.dart:691-695` — `needsRender:false` なら renderInfo も render も status も踏まずに即 return。
- `motolii/ui/lib/session/editor_session.dart:791,858` — 返信が「状態を持たない」判定を `response['ok'] == true` へ(鍵が 2 つになった静かな返信が document へ流れ込まないように)。副産物として**静かな seek(スクラブ)も絵が動かない時は描かない**。
- `motolii/ui/lib/session/editor_session.dart:700-706, 807-816` + `panels/stage.dart:86,1099` — `_shownViews`: 見えていない view は描かない(`attachView`/`detachView`)。今回の利用者 layout では両方見えているので**効いていない**が、既定 dock(Camera が裏 tab)では GPU の仕事が半分になる。空なら今まで通り全部描く。

### 計測(後)— 同じ 12 秒、tick 430 回 = 描いた 252 + **飛ばした 178**

| 物 | 前 | 後 |
|---|---|---|
| 描かずに済んだ tick | 0 | **178 / 430(41 %)** |
| Flutter コマ build 中央(16.7 ms 超の本) | 42.77 ms | **17.39 ms** |
| Flutter コマ全体 中央(同) | 49.84 ms | **20.19 ms** |
| 33 ms 超のコマ | 108 | **72** |
| UI thread が `_renderNow` に居た時間 / 12 秒 | 8,645 ms(72 %) | **7,274 ms(61 %)** |
| raster 中央 | 1.34 ms | 2.34 ms |
| `PROBE room=stage-window` | `verdict=matched lag=0` | `verdict=matched lag=0`(0 以外は 0 本) |

### まだ遅い物(次の便)

1. **本丸は残っている** — 描く 1 回はまだ中央 12 ms・p90 60 ms で、16.7 ms の予算を 1 回で食い切る。`device.poll(wait_indefinitely)` を UI thread から外す(submit して返り、`on_submitted_work_done` か別 thread の poll で `frame_ready` を鳴らす)のが次。roi は submit の前に決まるので 0 コマの約束は保てるはずだが、約束を壊す危険があるので今回は触っていない。後の最大値が伸びた(140 → 566 ms)のは、飛ばした分だけ毎回「本当に新しいコマ」を描くようになったため(素材の復号・shader の初回)で、run ごとのばらつきも入る。
2. **`_TimelinePainter.paint()`(`timeline.dart:1532` 以降)は 1 回の paint で `Paint()` を約 30 個・`Path()` を数個作る** — `custom-canvas-and-gestures` の「paint() の中で Paint / Path を作らない」に真っ向から反する。ただし今回のデータでは raster 中央 1.3–2.3 ms で**一番重い所ではない**ので、規則「一番重い所だけ直す」に従って触っていない。GPU 待ちを外した後に測り直す番。
3. 停止直後の全部入り status(140 KB、status 7.1 ms + decode 1.3 ms + `_accept` 1.2 ms)は 1 回きりだが、スクラブを止めた瞬間に 1 コマ落ちる。

### gate

`flutter analyze`: 1 件(`test/inspector_layout_section_test.dart:6` の `unnecessary_import`、着手前から。lib は 0)。`flutter test`: 失敗 **10 本**、同じ tree で本 lane の Dart を stash した基準は **12 本** — 新しい失敗は無い(基準の 12 = 既知 8 + `panel_placement` + `panel_layout_cost` の pane ring + `live_layers` + `stage_spatial_gizmo`、後 2 本は本 lane で通る)。`dart test`(lints): 11 passed。`dart run bin/check.dart lib`: `raw_dimension: clean` / `raw_color: 1` / `material_import: 1` — どちらも `workspace/layout` 側の既存で、本 lane は触っていない。`cargo build -p motolii-ui`: 完走(7m36s、warning 38 は既存)。`cargo test -p motolii-ui --lib`: `89 passed; 5 failed` — 既知は `every_example_builds_its_document` 1 本で、残る 4 本(`every_effect_on_the_shelf_has_a_snapshot`・`freeze_bakes_the_layer_in_the_background_and_unfreeze_forgets`・`creating_particles_shows_the_particle_rows`・`the_cage_follows_the_drawn_text_under_an_orbited_camera`)は**本 lane の物ではない**: 本 lane の Rust の変更は静かな `tick`/`seek` の返信 1 行だけで、`quiet` を使う Rust の試験は 1 本も無い(`grep '"quiet"'` の当たりは `lib.rs:274` と Dart の 1 行のみ)。この dylib は 10:00 以降に置かれた別 lane の `block_program.rs`・`engine/blocks.rs`・`vism/*.wgsl` を**初めて**取り込んだ build で、4 本はそちら側。裁定は利用者へ。

### 当てた規則(`.claude/skills/flutter-skills/`)

- `flutter-performance` 規則 5「重い CPU は UI isolate の外へ、UI thread を塞いだら必ずガクつく」→ `editor_session.dart:685` の `_renderNow`。塞いでいるのは Dart の計算ではなく FFI の先の GPU 待ち。
- `flutter-performance`「build 重 / raster 重 を分けてから直す」→ build 中央 42.8 ms 対 raster 中央 1.3 ms = **build 重**。だから Timeline の painter ではなく Ticker の中身を直した。
- `custom-canvas-and-gestures`「paint() の中で Paint / Path を作らない」→ `timeline.dart:1532` に当てはまるが、データが「一番重い所ではない」と言うので見送り(上の「まだ遅い物 2」)。

## Timeline の painter(裏)(2026-09-19、同日、上の「まだ遅い物 2」の番)

**やった事**: `timeline.dart` の 2 つの painter(`_TimelinePainter` = 定規と行の両方、`_ArrangementOverview` = 下の俯瞰)の `paint()` から `Paint` / `Path` の生成を全部外した。`custom-canvas-and-gestures` 規則 5「paint() の中で Paint / Path を作らない — 場に持って安い属性だけ書き換える」の写し。

**数(paint() の本体の中の生成)**

| painter | `Paint()` 前 → 後 | `Path()` 前 → 後 |
|---|---|---|
| `_TimelinePainter.paint` | **37 → 0** | **4 → 0** |
| `_ArrangementOverview.paint` | **6 → 0** | 0 → 0 |

1 コマに定規 1 枚 + 行 1 枚 + 俯瞰 1 枚が塗られるので、**再生 1 コマあたり 80 個の `Paint` と 8 本の `Path`** が消えた(行の painter は行数 × key 数だけ更に増えるので、実際はこれより多い — 上の 37 は生成の書き口の数で、loop の中の物は行数分だけ掛かる)。

**どう置いたか**(`timeline.dart:1451-1466`、file の頂点の 4 つ + 3 つの助け):
- `_fillPaint`(塗り)・`_linePaint`(線、`drawLine` は style を見ず `strokeWidth` だけを見る)・`_strokePaint`(縁取り)・`_scratchPath`(菱形・切り欠き・再生頭の三角)。
- `_fill(color)` / `_line(color, [width])` / `_stroke(color, [width])` が色と太さだけを書き換えて同じ物を返す。`Canvas.draw*` は呼んだ瞬間の値を読むので、続けて使い回しても絵は変わらない(`_scratchPath` も `..reset()` してから積み直す)。
- `_ArrangementOverview` は `const` の構築子を持つので、場ではなく **file の頂点**に置いた(両方の painter が読む。`paint()` は UI thread の上で 1 本ずつしか走らない)。

**絵は変えていない**。色は全部 `EditorTheme` / `EditorInk` の名前のまま(`raw_color` は増えていない)、幾何も順番もそのまま。書き口が省略していた既定値(`strokeWidth` 0 = 髪の毛線、marquee の縁取り、marker の線、定規の目盛)は助けの既定引数 0 で同じに保った。棚卸しの装置は全部残っている: ease の帯(実線 / 破線・両端が選ばれたら太く)、ゴーストのはみ出しと frame 0 の切り欠き、marker の線と丸、key の菱形(掴んだ分だけずれる 2 個)、lamp 4 つ、名前の欄、「尺は壁ではない」印、選択の面。

**やらなかった事と理由**
- **`shouldRepaint` を 1 個の値物に畳む** — 見送り。`_TimelinePainter` は 21 個の場を持ち、今の `shouldRepaint` は既にその 21 個を 1 個ずつ比べる = 値物の `==` と**同じ判定**なので、畳んでも repaint の回数は 1 回も変わらない(振る舞いが変わらない改修は入れない、という条件に従った)。畳むなら painter の構築子と 3 箇所の呼び出しを全部書き換える大きな手で、絵の危険に見合わない。
- **`repaint:` の `Listenable` へ移す** — 見送り。この 3 つの `CustomPaint` は既に `ticker の setState` ではなく `ListenableBuilder(Listenable.merge([controller.frame, _timeline, scrolled]))` の中に居て、更に `RepaintBoundary` で包まれている(規則 6 の趣旨は満たしている)。`repaint:` へ本当に移すには painter が `frame` を場ではなく `ValueListenable` から読む必要があり、それは「painter は馬鹿で、Scene を受け取るだけ」(規則 1)と `shouldRepaint`(規則 2)を壊す。`setState` で毎コマ塗り直している painter は**無かった**。
- **paint の数を数える kDebugMode の計測** — 入れていない。今回の数は「生成が 0 個である」という**構造の事実**で、走らせて数える物ではない(数えられるのは draw の回数で、それは減っていない)。

**gate**: `flutter analyze` は `lib/` 0 件(着手前からの `test/inspector_layout_section_test.dart:6` の `unnecessary_import` は test 側なので直した → 全体 `No issues found!`)。`flutter test` は着手前に同じ tree で取った基準と同じ失敗 10 本で、新しい失敗は無い。`dart run bin/check.dart lib`: `raw_dimension: clean` / `raw_color: 1` / `material_import: 1`(どちらも `workspace/workspace_view.dart` の既存)。`panel_layout_cost_test.dart` の Timeline の行は前後とも `Timeline   update     2   builds    97`(painter は layout を通らないので当然だが、増えていない事の証)。

**次**: 上の「まだ遅い物 1」(`device.poll(wait_indefinitely)` を UI thread から外す)が本丸で、そこを外してから raster を測り直すと、この painter の効きが初めて数字に出る。

## GPU の待ちを UI thread から外す(裏)(2026-09-19、同日、前段「まだ遅い物 1」の続き)

前段が残した本丸 —— `motolii_probe_render` が `render_into` の中で `device.poll(wait_indefinitely)` を踏み、
Dart の Ticker がその GPU の描き終わりを UI thread の上で待っていた —— を外した。**原因は前段の診断の通りだった**。

### 新しい毎コマの道(言葉で)

Dart は `motolii_probe_render_async`(新しい口)を呼ぶ。Rust は **submit した所で返る**。
GPU が終わると `Queue::on_submitted_work_done` の callback が旗を立て、`frame_ready(user, view, surface_id)` を鳴らす。
callback を鳴らすのは **poll した thread** なので、鳴らす係を 2 つ置いた:

1. **次の tick の頭**(`lib.rs:258`)— 塞がらない `poll(PollType::Poll)` を 1 回。終わっていなければ即座に返る。毎コマ必ず通る道なので、これが主役。
2. **待ち係の thread**(`GpuWaiter`、`lib.rs:104-146`)— 積んだ仕事がある間だけ 4 ms 毎に `poll(Poll)`。Dart が tick を止めた後の最後の 1 枚を取りこぼさないための受け皿。仕事が 0 なら condvar で寝る。

**塞がる `poll(Wait)` は使わない。** 最初これを待ち係の thread で回したら、`_renderNow` の中央が
**8.5 → 26.6 ms に悪化した**(計測: `scratchpad/gpu-wait/after.log`)。`poll(Wait)` は wgpu の device の錠を
握ったまま GPU を待つので、UI thread の `queue.submit` がその錠で止まる —— 待ちの置き場所が変わっただけだった。
500 µs 毎の `poll(Poll)` でも錠の取り合いで 19.1 ms(`after2.log`)。4 ms + 次の tick の頭、で 7.45 ms に落ちた。

### 裂けない・古くない ための 3 つ

- **二重書きの番人**(`lib.rs:243` `surface_is_free`)— 行き先の surface をまだ GPU が書いていれば、再生中は**描かずに飛ばし**(`AsyncSkip`、数える)、それ以外は空くまで待つ。**実測では 1 回も飛ばなかった**(`skips=0`)。view ごとに 2 枚を交互に使うので、自分の番が回る頃には 1 コマ前の仕事はもう終わっている。
- **窓が動いたコマは待つ**(`lib.rs:264`)— Fit・zoom・伸縮で窓や roi が変わった描画は `RenderWait::Sync` へ落とす。Flutter は「頼んだ窓」の矩形へ絵を置くので、そのコマだけ絵が遅れると前の窓の絵がその矩形へ伸びる = 昼に直した潰れが戻る。窓は時刻の関数ではない(`snapshot.rs:68` の `window()` は `stageWindow` op でしか動かない)ので、再生中は毎コマ同じ = 待たない道のまま。
- **選択の籠は待たない**(`selection_bounds.rs:158` に `wait`、`engine.rs:291`、`snapshot.rs:212`)— 籠は GPU の読み戻し(map)なので、待つと元の木阿弥。届いていなければ **前の籠をそのまま持つ**。既存の呼び手(書き出し・試験)は全部 `wait = true` で今まで通り。

### 計測(前 → 後)

同じ機械・同じ build・同じ `light-in-form.rrd`・同じ自動走行(12 秒放置 → 12 秒再生 → Fit/In/Out/Actual/Fit)。
差は dart-define `MOTOLII_SYNC_GPU` だけ(`editor_session.dart:420`、同期の道を 1 本残してある)。
log は scratchpad の `gpu-wait/before2.log` と `gpu-wait/after4.log`。

**再生の 12 秒、描いた tick(前 293 回 / 後 358 回)、ms**

| 段 | 前 中央 | 前 p90 | 前 最大 | 後 中央 | 後 p90 | 後 最大 |
|---|---|---|---|---|---|---|
| `_renderNow` 全体 | 9.28 | **48.22** | 148.99 | **7.45** | **18.74** | **47.81** |
| `tick`(FFI) | 0.06 | 0.08 | 0.12 | 0.06 | 0.08 | 0.18 |
| `renderInfo`(FFI) | 0.03 | 0.04 | 0.09 | 0.03 | 0.03 | 0.07 |
| `render` Camera | 3.66 | **26.55** | 114.99 | **2.80** | **8.07** | **17.33** |
| `render` User = Stage | 3.70 | **18.46** | 27.75 | **2.53** | **7.28** | **22.62** |
| `status`(FFI) | 1.61 | 2.91 | 3.71 | 1.65 | 2.96 | 4.49 |
| `jsonDecode` | 0.05 | 0.11 | 0.23 | 0.07 | 0.13 | 1.24 |
| `_accept`(notifier) | 0.39 | 0.51 | 2.18 | 0.41 | 0.49 | 5.95 |
| `_broadcast`(副窓 0) | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

**Flutter 自身のコマ(16.7 ms 超だけ記録)と全体**

| 物 | 前 | 後 |
|---|---|---|
| 16.7 ms 超のコマ | 128 本 | 198 本 |
| その build 中央 | 45.56 ms | **21.41 ms** |
| その raster 中央 | 1.54 ms | 1.51 ms |
| その全体 中央 | 59.39 ms | **25.27 ms** |
| その全体 最大 | 1815.98 ms | **1334.05 ms** |
| **33 ms 超のコマ** | **122 本** | **12 本** |
| 再生 12 秒に描けた tick | 293 | **358** |
| **UI thread が `_renderNow` に居た時間 / 再生 12 秒** | **6,583 ms(55 %)** | **4,539 ms(38 %)** |
| 飛ばした tick(`skips`) | — | **0** |
| `PROBE room=stage-window` | `verdict=matched lag=0` ×5、0 以外 **0 本** | `verdict=matched lag=0` ×5、0 以外 **0 本** |

目標「20 % well under」には届いていない。残る 38 % は**待ちではなく本当の CPU** —— view 2 枚の層の組み立てと符号化
(各 2.5〜2.8 ms)+ `status` 1.65 ms。待ちは消えた(p90 が 48.2 → 18.7、Camera の p90 が 26.6 → 8.1)。
33 ms 超のコマが 122 → 12 本、janky なコマの build 中央が 45.6 → 21.4 ms、これが利用者の「ガクガク」に一番近い数字。

### gate

- `flutter analyze lib`: `No issues found!`(lib は 0。test の `inspector_layout_section_test.dart:6` の `unnecessary_import` 1 件は着手前から)
- `flutter test`: `+160 -10` —— 前段が記録した基準 **10 本と同じ集合**(desk_workspace・ease_interaction・module_contract・panel_layout_cost ×4・他 3)。着手前に自分で数えても 10 本。**新しい失敗は無い**
- `cargo test -p motolii-ui --lib`: `test result: FAILED. 90 passed; 5 failed; 10 ignored` —— 落ちる 5 本は**前段が記録した 5 本と同じ**(`every_effect_on_the_shelf_has_a_snapshot`・`every_example_builds_its_document`・`freeze_bakes_the_layer_in_the_background_and_unfreeze_forgets`・`creating_particles_shows_the_particle_rows`・`the_cage_follows_the_drawn_text_under_an_orbited_camera`。全部別 lane の物、今回は触っていない)。**通った本数が 89 → 90** = 今回足した `the_async_render_signals_after_the_gpu_is_done` が通っている
- `cargo test -p motolii-render --lib`: GATE_RENDER_TEST
- 書き出し(`export_range_with_progress`)と `zz_watch`: **どちらも `Engine::render_frame`**(読み戻しの道、`engine.rs:1091` の poll)を通る。今回触ったのは窓へ描く道(`render_into_window`)だけで、読み戻しの道の poll は 1 行も変えていない。書き出しは更に別 thread で自前の `Engine::new()` を建てる(`export_job.rs:25`)ので、待ち係とも無縁。同期の口 `motolii_probe_render` もそのまま残してある(channel の fallback と Rust の試験が使う)

### 変えた file

- `motolii/ui/native/src/lib.rs` —— `RenderWait`(`:97`)、`GpuWaiter`(`:104`)、`Signal`(`:158`)、`surface_is_free`(`:243`)、`render` に待ち方と `last_window`(`:256-265`)、`render_into_mode`(`:307`、合図は両方の道でここから鳴る)、`motolii_probe_render_async`(`:464`)、`motolii_probe_gpu_waits`(`:479`)。試験 `frame_ready_tests::the_async_render_signals_after_the_gpu_is_done`
- `motolii/crates/motolii-render/src/compositor/selection_bounds.rs:158`・`compositor/presentable.rs:63`・`engine.rs:291`・`engine/texture.rs:1871` —— 籠の読み戻しに `wait`。既存の呼び手は全部 `true`
- `motolii/ui/native/src/snapshot.rs:212` —— `take_selection_bounds(..., wait)`
- `motolii/ui/lib/bridge/native_frames.dart:19,98` —— `renderAsync`
- `motolii/ui/lib/session/editor_session.dart:415`(計測の switch)・`:420`(`MOTOLII_SYNC_GPU`)・`:423`(飛ばした数)・`:777`(呼び分け)・`:789`(全部飛んだら status も踏まない)

### 残る物

1. **UI thread の 38 %** は層の組み立てと符号化。次は前段 §1 の続き(status を struct/バイナリで渡す)と、view 2 枚の層の解決を 1 回で済ませる事(同じ時刻・同じ document を 2 回解いている)。
2. `_TimelinePainter.paint()` の `Paint()` 約 30 個(`timeline.dart:1532`)は**まだ一番重い所ではない**(raster 中央 1.5 ms、前段から動かず)。順番はまだ来ない。
3. **飛ばす道は 1 回も通っていない**(`skips=0`)。番人としては正しく立っているが、効いているかは測れていない。もっと重い作品(4K・層 300)で初めて出るはず。
4. **裂け・古い絵の目視は出来ていない** —— 実窓の screenshot は利用者の担当(記憶「実窓操作は拒否される」)。出せた証拠は間接的な物だけ: 走行 6 本で `PROBE room=stage-window` が 48 行、**0 以外の lag は 1 本も無い**(zoom の途中経過まで拾った `after2.log` の 28 行を含む)。`skips=0`(surface が書かれている最中に二度と入っていない)。log に `Rust render failed` も `nativeError` も無く、例外は前後で同じ 12 件(別 lane の `No Material widget found`)。**絵そのものの合否は利用者の裁定を待つ**。
5. 実窓の divider を引く「伸縮」は自分では動かせない(前段と同じ)。窓の約束は Fit・In・Out・Actual を code から打って確かめた(`editor_session.dart:545` の自動走行)。
6. GPU の試験と app を同時に回すと `re_renderer` の shader 見張り(`FSEventStreamStart` → fseventsd)が固まる件は**今日も 2 回出た**(`sample` で確認)。前段の指摘の通り、見張りを遅らせて別 thread で立てるのが筋。

## 非同期を戻す(裏)

上の走行の後、Rust の**中で**もう一度測った。1 回の描画の CPU は **0.18〜0.19 ms(中央、静止した書類、view 2 枚とも)**。
Dart から見えていた 4〜8 ms は**初回**(shader と pipeline の compile)と GPU の poll であって、定常の値ではない。
待ちを外して買える物は 0.18 ms しか無く、代わりに危ない所が増える —— status と絵が食い違う、
籠の読み戻し(`selection_bounds`)が何も返さない、最後のコマが落ちる。**利用者の裁定: 描画は同期のまま**。

### 消した物(非同期のためだけに在った物)

- `motolii/ui/native/src/lib.rs` —— `motolii_probe_render_async`、`motolii_probe_gpu_waits`、`RenderWait{Sync,Async,AsyncSkip}`、
  `GpuWaiter`(裏の poll thread と condvar)、`inflight` の旗の表、`surface_is_free`、`last_window`、
  `completed_bounds`/`CompletedBounds`/`accept_completed_bounds`、`poll(PollType::Poll)` の前置き、
  試験 `frame_ready_tests::the_async_render_signals_after_the_gpu_is_done`。
  `render_into_mode` は 1 本に戻した(`lib.rs:220` `render_into_surface`、`lib.rs:177` `render`)
- `motolii/crates/motolii-render/src/compositor/selection_bounds.rs`・`compositor/presentable.rs:62`・`engine.rs:290`・
  `engine/texture.rs:1871` —— 読み戻しの `wait: bool` と `SelectionReadback`/籠の池/`detach`。**常に待つ**に戻した(実測 0 ms)
- `motolii/ui/native/src/snapshot.rs:211` —— `take_selection_bounds(..., wait)` と `accept_selection_bounds` の分割
- `motolii/ui/lib/bridge/native_frames.dart` —— `renderAsync` と その binding
- `motolii/ui/lib/session/editor_session.dart:685` `_renderNow` —— 非同期の呼び分け、surface の使用中の番人、
  `_gpuSkips`/`_renderPending`、`MOTOLII_JANK` の計測一式(段ごとの Stopwatch、`addTimingsCallback`、
  12 秒再生 + Fit/In/Out/Actual の自動走行)、`MOTOLII_SYNC_GPU` の switch
- `motolii/ui/macos/Runner/MainFlutterWindow.swift:281` —— `frameCompleted` の呼び返し、完了の serial と
  `publishedSerial`、main thread への飛ばし直し。`frameReady` は**呼んだ thread のまま**(= Dart の render の中)

### 残した物(今日の keeper)

- **Dart の FFI 橋**: `native_frames.dart`、`EditorSession._renderNow`、Swift の surface の池・`ensureSurfaces`・
  `frame_ready` → `textureFrameAvailable`。`motolii_probe_set_frame_ready`(`lib.rs:361`)は同期の描画の**後で**鳴る
- **絵が動いていなければ描かない**: 静かな `tick`/`seek` が Rust の `image_key` から `needsRender` を返し
  (`lib.rs:325`)、`_renderNow` が早く返る(`editor_session.dart:693`)

### gate

- `cargo build -p motolii-ui`: `Finished \`dev\` profile [optimized + debuginfo] target(s)` —— error 0
- `cargo test -p motolii-ui --lib`: GATE_UI_TEST
- `cargo test -p motolii-render --lib`: GATE_RENDER_TEST
- `flutter analyze lib`: `No issues found!`
- `flutter test`: `00:26 +160 -10: Some tests failed.` —— 落ちる 10 本は着手前に自分で数えた 10 本と**同じ集合**
  (desk_workspace・ease_interaction・module_contract・panel_layout_cost ×4・panel_placement・visual_selection)。新しい失敗は無い
- lints: `raw_dimension: clean` / `raw_color: 1 raw colours` / `material_import: 1 Material/Cupertino imports` —— 着手前と同じ
- 実窓: GATE_APP

### 覚えておく事

着手前の `cargo test -p motolii-ui --lib` は**終わらなかった**。`frame_ready_tests` の 2 本
(`the_async_render_signals_after_the_gpu_is_done` と `the_frame_ready_signal_fires_once_per_render`)が
GPU の借り(`GpuLease`)を握ったまま固まり、走行全体が止まる。`sample` で確認した。
非同期を消した今、この 2 本のうち残った 1 本は普通に通る。
