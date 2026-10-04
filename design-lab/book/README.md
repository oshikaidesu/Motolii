# lab_book

design-sense-lab の Widgetbook。

- **見本は Workspace の窓**(製品ではない): [`lib/workspace/README.md`](lib/workspace/README.md)。製品の窓は Motolii の `scripts/motolii-ui.sh dev`。

## Widgetbook の棚(`lib/main.dart`)
| 棚 | 中身 |
|---|---|
| `Workspace` | 今の見本の窓(製品ではない) |
| `Parts` | Workspace が使っている席ごとの部品(Inspector は `Parts › Inspector › Inspector Pop`、Timeline / Stage / Media / Transport / Relation の部品) |
| `Foundations` | トークンと基礎部品(Controls / Inputs / Navigation / Feedback) |
| `Workflows` | 操作の通し(Workflow / Dose) |
| `Archive` | 見返す用。見本に置き換わった最初の草案(`First drafts`)と、検討の研究(`Studies`: OP / Pop / Browser / Inspector / Worlds / Phenomena)。棚は `lib/archive.dart` |

## ファイルの地図
| 場所 | 中身 |
|---|---|
| `lib/workspace/` | 見本の窓(地図は [`lib/workspace/README.md`](lib/workspace/README.md)) |
| `lib/sets/inspector/` | Inspector(`inspector_parts.dart` が入口、`InspHost` と Pop のトークン)と `param_infer.dart` |
| `lib/sets/panels/` | Timeline / Stage / Media / Transport / Relation の部品 |
| `lib/sets/foundations/` | Inputs / Navigation / Feedback |
| `lib/sets/workflows/` | Workflow / Dose / R1 の通し |
| `lib/sets/studies/` | Archive の研究(OP / Pop / Worlds / Phenomena / 旧 Browser・Inspector 案) |
| `lib/parts/` | 最初の草案の部品。`controls.dart` と `glyphs.dart` は今も Workspace と Foundations が使う |
| `lib/tokens.dart` / `kit.dart` / `session.dart` / `*_addon.dart` | トークンとテーマ、棚の小道具、操作の通しの状態、Widgetbook のアドオン |

- `lib/sets/inspector_gui_set.dart` は編集中なので `sets/` の直下に残してある。`lib/sets/inspector_parts.dart` はそのための中継(`inspector/inspector_parts.dart` を export するだけ)。
