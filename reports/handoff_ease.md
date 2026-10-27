# 引き継ぎ：区間イーズ（10/25 夜）

## 解く問題
**Motolii の区間イーズ（キーとキーの間の進み方）で、「今の Motolii と Alight Motion に無い線の具合」をどこまで出せるかを増やし、窓の Ease の机から触れるようにする。**

- 土台は Alight Motion の 9 種（直線・止める・ベジェ・Bounce・Elastic・Cyclic・Random・Steps・Elastic Steps）。Motolii は `filament/interp.h` に同じ 9 種を持つ。**ただし既定の数字と取っ手の働きが Alight Motion と違う**（Elastic・Random・Steps は大きく、Bounce・Elastic Steps は少し）。比べた絵：`reports/alight_ui/137_ease_vs_motolii.jpg`。Alight Motion の既定の曲線と取っ手の位置の数字：`reports/alight_ease/default_curves.csv`、`default_handles.json`。
- Alight Motion は「区間ごとに型を選ぶ」入口までで止まっている売り物。Motolii はその先へ伸ばす。伸ばす方向とお手本：`reports/ease_segments.md`。
- **足してはいけない物**：
  - 始まりと終わりの 2 つの取っ手のベジェで描ける物（Sine・Quad・Cubic・Quart・Quint・Expo・Circ・Back と、その in/out/inOut/outIn）。easings.net が全部に cubic-bezier を載せている。机の見本 8 枚とベジェの取っ手で足りる。
  - Alight Motion から写した 9 種と重なる物（Penner の Elastic・Bounce、anime.js の irregular＝Random、spring＝Elastic に近い）。
  - **何かを足す前に、既にある物（型・見本・お手本から写した物）と重なっていないか数える。**
- 欲しいのは「性格のある線」：溜めてから飛ぶ（anticipation）、行き過ぎて揺れて止まる、震える、段々にカクつく、ゴムのように伸びる、区間をまたいで速さが続く（ばね・ロービング）など。
- 借りる先はライセンスを確かめてから。決まっていること：anime.js・Motion・bezier-easing・bodymovin は MIT で借りられる、GSAP は独自ライセンスで借りない。

## 今の状態
**コミット済み**
- `8fa40d2`：SQLite の関数 `anime(式, u)`。区間の式（`define` や `filament/ways/*.way`）から anime.js（`filament/third_party/anime/`、MIT、v4.5.0 をそのまま）のイーズを呼べる。QuickJS で動く（`filament/script.cpp` の `anime_ease`、`filament/doc_match.h`）。検査 `filament/check/anime_ways.sh`（yes）。これは「点で好きな形を描く `linear()`」などの道具として残す。
- `f8d60ad`：anime.js の見本 46 本（`filament/ways/anime_*.way`）。**これは足してはいけない物だった**。

**まだコミットしていない変更**（作業の途中。見直してから扱う）
- `filament/ways/anime_*.way` 46 本を消した（上の誤りを戻す）。
- `skin/book/lib/workspace/desk_ease_library.dart`（新規）：机の下の段。表（All / ★）、横スクロールの札、並びは固定、最近使った 4 本を頭に区切って出す、星は ★ の表に集める、星はカーソルを乗せた時だけ見せる。星と最近は `~/.motolii/ease_stars.json`。
- `skin/book/lib/workspace/desk_ease_presets.dart`：下の段を `EaseLibrary` に替えた。`PresetTile` を公開し、`label` を足した。
- `skin/book/test/workspace_ease_library_test.dart`（新規）：**anime.js の表がある前の形のまま。今の作りに合わせて書き直す必要がある**。
- `ASSETS.md` の anime の行は「見本 46 本」のまま。直す。

## 確かめ方
- エンジン：`cd filament && ./edit.sh out/x.motolii < check/anime_ways.txt`、`./check/anime_ways.sh`。窓が使うライブラリは `./build_lib.sh`。
- 窓：`cd skin/book && flutter test test/workspace_ease_library_test.dart test/workspace_desk_test.dart`（flutter は `/Users/motolii_maker/Documents/Motolii-probes/toolchains/flutter/bin/flutter`）。窓は `flutter build macos --debug`、`open -n build/macos/Build/Products/Debug/lab_book.app`（15:52 から動いている古い窓があるので `-n` で別に開く。古い窓は閉じない）。
- 前からのずれ（今回の変更と無関係）：窓のテスト 9 本（Trim Paths ほか）、`filament/check/doc.sh` の 2 つの NO（値の一覧・動画の速さ）。

## 進め方の決まり（partner と記憶より）
- 自分で書かず、外の実装を借りる。借りられなければ中身の出典を示す。
- 区切りで止まらない。止まるのは次が明らかでない時だけ。いちいち方針を聞かない。
- 言葉は短く、やさしい日本語で。報告は数字と結果で、悪い結果を先に。
