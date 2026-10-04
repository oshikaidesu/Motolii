# design-sense-lab

Motolii とは別の、隔離した作業場。デザインの知識を LLM が読める形にして、実画面で試すための場所。

> **これは Motolii の製品ではない。** 見た目の見本。製品の窓は Motolii の `motolii/ui`(`scripts/motolii-ui.sh dev`)だけ。見本の Stage は代用の絵で、作品は保存されない。
> 見本どうしで今の形は Widgetbook の `Workspace › Window`([`book/lib/workspace/README.md`](book/lib/workspace/README.md))。製品の窓とは呼ばない。

## 中身
- `DESIGN.md` — Motolii の Timeline から始めたデザインシステム(awesome-design-md の 9 節書式)。値に出所の印(`[code]` `[concept]` `[decided]` `[open]`)。選択状態の決まり(部品の種類ごと)も入っている。
- `preview-dark.html` — DESIGN.md の値だけで描いた Timeline の静的な見本。
- `book/` — Flutter の Widgetbook。126 個の use case(16 コンポーネント)。
- `research/precedents.md` — Adobe Spectrum、Blender、Radix、Linear などの先例調査。
- `research/open-decisions.md` — 先例との差 10 項目と、あなたが決める項目 A〜I。

## 起動(macOS)
```bash
cd book
/Users/member_ottoto/rust_ae/Motolii/.tools/flutter/bin/flutter run -d macos --pid-file /tmp/lab-book.pid
# 反映: kill -USR1 $(cat /tmp/lab-book.pid)   (const のクラスを変えた時は -USR2)
```
- ウィンドウは「Window > Fill」で広げる。
- 各 use case の右パネルに knob。`Look`(quiet / concept / glow)、`Timeline > Six layers` は行の高さ・縞・キーの縁・格子。

## 作り方のルール
- 新しく作る前に、リポジトリとネットで既存を探す。見つからない時だけ作る。
- 値は `book/lib/tokens.dart`(DESIGN.md から)。部品は数値を直書きしない。
- 見た目の判断は、先例と数値の差で並べる。決めるのは利用者。
