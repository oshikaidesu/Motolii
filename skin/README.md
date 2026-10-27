# skin — Motolii の窓

Motolii の窓（Flutter、macOS）と、その見た目の決まり。窓は操作だけで、絵は中身（`../filament/`）が描く。

## 中身
- `book/` — 窓そのもの。Flutter の Widgetbook として組んであり、`Workspace › Workspace › Live` が本物の窓（[`book/lib/workspace/README.md`](book/lib/workspace/README.md)）。試験は `book/test/`・`book/integration_test/`。
- `book/skins/` — 窓の着せ替え（色・部品の絵）。`preview.png` は `book/skins/shoot.sh` で撮った窓の絵。
- `BRIDGE.md` — 窓と中身の境目の決まり（窓が中身を呼ぶ所）。
- `DESIGN.md` — 見た目の決まり（値に出所の印つき）。
- `OVERVIEW.md`・`PROCESS.md` — 窓を別の作業場（design-sense-lab）で作っていた 10 月初めの記録。今の形は上の `book/`。

## 開く（macOS）
```bash
book/live.sh
```
- `.dart` を保存すると開いている窓にすぐ出る（Flutter の hot reload）。窓を閉じると終わる。
- 中身の場所は `live.sh` が自分の置き場所から求める（別の所なら `MOTOLII_DIR=/path/to/filament`）。

## 作り方の決まり
- 新しく作る前に、リポジトリとネットで既存を探す。見つからない時だけ作る。
- 値は `book/lib/tokens.dart`（`DESIGN.md` から）。部品は数値を直書きしない。
