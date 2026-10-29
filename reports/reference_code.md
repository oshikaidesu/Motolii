# コードで読める手本（10/29）

夜のあいだも、ユーザーに聞かずに「どう操作するか」を決めるための手本。AbsentMotion（Alight Motion の写し）が床で、その上はここから取る。どれも PC のソフトで、操作の仕方がコードに書いてある。

置き場所は Motolii の外：`~/Documents/Motolii-probes/refs/<名前>`（浅く取り寄せた物。git には入れない）。取り寄せ直しは各行の `git clone`。

## 5 つの手本

| 名前 | 何の手本 | ライセンスと取り方 | 取り寄せ |
|---|---|---|---|
| **Godot**（ゲームエンジンの動きの編集） | 時間軸の印（右クリック：Insert Marker…・Rename Marker・Delete Marker(s)・Show All Marker Names）、キーの右クリック（Insert Key…・Delete Key(s)）、キーの間の曲がり方（Bezier の編集） | MIT：コードごと借りてよい | `godot` cafc17a（10/27）。`git clone --depth 1 --filter=blob:none --sparse https://github.com/godotengine/godot` → `sparse-checkout set editor/animation editor/inspector editor/gui scene/gui scene/animation` |
| **Theatre.js**（Web の動きの編集、モダン） | キーの右クリック（Copy・Paste Keyframes・Delete）、イーズの名前の並び（Quad・Cubic・Back… In / Out / In Out、Hold）、再生位置の Place marker、範囲（focus range）を消す | Apache-2.0：コードごと借りてよい | `theatre` 6ea82b9（2024/4 で止まっている）。`git clone --depth 1 https://github.com/theatre-js/theatre` |
| **Glaxnimate**（KDE のベクターの動きの編集。PC 版の Alight Motion に一番近い） | レイヤーの右クリック（Precompose・Mask・Parent・上へ下へ・Convert）、値の右クリック（Add Keyframe・Clear Animations・Copy/Paste Keyframe・Follow Path・Perfect Loop）、キーの間（Hold・Linear・Ease・Ease In・Ease Out・Overshoot・Anticipate・Fast・Custom）、**書類の色見本**（Link Shapes to Swatch：色見本を変えるとつないだ物が全部変わる）、塗りとグラデーション | GPL-3：振る舞いと名前だけ手本にする（行は写さない） | `glaxnimate` b359f99（10/25）。`git clone --depth 1 https://github.com/KDE/glaxnimate` |
| **Penpot**（オープンソースの Figma） | 舞台の右クリック（context_menu.cljs）、色（グラデーションの色の点、**harmony：補色などの組み合わせ**、ramp、ライブラリの色）、文字のスタイル（typographies）、レイヤー、**取り消しの一覧（history）**、**版の見比べと戻し（versions）** | MPL-2.0：振る舞いと名前だけ手本にする | `penpot` 2607ec8（10/29）。`git clone --depth 1 --filter=blob:none --sparse https://github.com/penpot/penpot` → `sparse-checkout set frontend/src/app/main/ui frontend/src/app/main/data common/src/app/common/types` |
| **Kdenlive**（KDE の動画編集） | 素材の棚（bin：Replace Clip…）、印（Add Marker・色の種類・Add Marker Quickly・範囲から印）、範囲（zone）、書き出し（render）、文字の場面（titler） | GPL-3：振る舞いと名前だけ手本にする | `kdenlive` 3d7c862（10/29）。`git clone --depth 1 https://github.com/KDE/kdenlive` |

決め方の順：まず Alight Motion（reports/alight_ui.md、AbsentMotion のコード）。そこに無い所、または PC で上を目指す所は、上の表の「何の手本」の列で受け持ちが決まる。受け持ちが 2 つ重なったら、Motolii の構成に近い方（時間とキーは Godot、物と色は Penpot）。どれにも無ければ「出典なし」と書いて止まる。

## シナリオ（reports/scenarios.md）ごとの手本

| シナリオ | 手本のファイル（refs から） |
|---|---|
| P03 キーカラー | Glaxnimate `src/gui/widgets/shape_style/document_swatch_widget.cpp`（Link Shapes to Swatch・Rename Swatch Color・Unlink）、Penpot `workspace/sidebar/assets/colors.cljs`（ライブラリの色） |
| P04 字体を役目ごとに | Penpot `workspace/sidebar/assets/typographies.cljs`（文字のスタイルをライブラリに） |
| P05 奥行き・P06 粒子 | Motolii 自身の物（2.5D・3D）。操作は Alight Motion |
| P07 押し込み・P08 印で揺らす | Godot `editor/animation/animation_track_editor.cpp`（キーと印）、Theatre.js `SequenceEditorPanel/RightOverlay/Playhead.tsx`（Place marker） |
| P09 行き過ぎて戻る | Glaxnimate `src/gui/widgets/timeline/keyframe_transition_data.hpp`（Overshoot・Anticipate）、Theatre.js `…/CurveEditorPopover/shared.ts`（Back Out など）、Godot `editor/animation/animation_bezier_editor.cpp` |
| P10〜P12 文字と HUD | Motolii 自身の文字の動き。右クリックは Glaxnimate `src/gui/widgets/menus/animated_property_menu.cpp` |
| P13 一語だけ色 | Penpot `workspace/sidebar/options/menus/text.cljs`（選んだ文字の範囲ごとの塗りと書体） |
| P14〜P16 光・グリッチ・つなぎ | Kdenlive の効果の範囲（zone）と場面転換（`src/transitions`）。0.5 秒だけ掛けるのは範囲（Kdenlive の zone、Theatre.js の focus range） |
| P17 絵の差し替え | Kdenlive `src/bin/bin.cpp`（Replace Clip…） |
| P18 尺を縮める | Godot `editor/animation/animation_track_editor.cpp` の Scale Selection（キーの間を縮める）、Kdenlive の範囲 |
| P19 前の版と比べる | Penpot `workspace/sidebar/versions.cljs` |
| P20 横長から縦長 | Penpot `workspace/sidebar/options/menus/constraints.cljs`（板の大きさを変えた時の中の物の付き方） |
| P21 何も選ばずに押した | Penpot `workspace/context_menu.cljs`（押せない時の出し方） |
| P22 戻したい | Penpot `workspace/sidebar/history.cljs`（取り消しの一覧：何が戻ったかが見える） |
| P23 どこを触っているか | Penpot `workspace/sidebar/layers.cljs`・`options`（選んだ物と値がいつも同じ場所） |
| P24 次の日に続きから | Kdenlive・Penpot の開き直し（最後の場所） |
| 右クリック全般（㉒） | 時間軸は Godot、レイヤーと値は Glaxnimate `menus/node_menu.cpp`・`animated_property_menu.cpp`、舞台は Penpot `context_menu.cljs`、棚は Kdenlive `src/bin` |
| 色の棚（⑬） | Penpot `workspace/colorpicker/`（gradients・harmony・ramp・libraries）、Glaxnimate `shape_style/`（fill_style・gradient_list） |
