# Icebook panel design drafts

Motolii の6領域を、Icebookで閲覧できるstoryの入力候補として比較するための草案台帳。
各領域30案、合計180案を置く。IDは各エージェントが書いた実在形
（`B01` / `I01` / `ST-01` / `T01` / `E01` / `S01`）を保ち、生成索引でも同じIDを使う。

## この台帳の位置づけ

これは実装仕様でも実窓合否でもなく、**パネルの視線・密度・主役・操作文法を先に比較するための設計候補**である。
候補を選んだ後に、選択案だけをIcedのstoryへ落とす。180案を製品へ実装する意味ではない。

現在公開されている `icebook` 1.2.0 は `StoryRegistry` と `ThemeProvider` を実装してstoryを登録し、
`run` / `run_with_settings` でstorybookを起動する形を取る。一方、Motoliiの`next/`はIced forkの
0.15-dev系を使っているため、Icebook 1.2.0のIced 0.14依存を本体workspaceへ直接混ぜない。
まずデザイン案を依存なしで確定し、採用案を隔離したIcebookギャラリーへ送る。

## 共通フォーマット

各案は次の欄を持つ。Markdown上の太字・コード表記、英語の大小や
`Problem solved` / `解決する問題` のような表記差は静的検査器が同じ欄として扱う。

| 欄 | 判定すること |
|---|---|
| `ID / name` | storyとして選べる安定名 |
| `problem` | そのパネルが解決する利用者の停止・不安・表現不足 |
| `hero` | hero creationへどう寄与するか |
| `layout` | 視線の順序、主役、補助、状態帯 |
| `interaction` | 入口、ドラッグ、選択、確定、キャンセル |
| `density` | sparse / balanced / dense と、成立させる対象量 |
| `reuse` | 既存pane・tokens・Document/Sessionへ預ける部分と、自前が必要な継ぎ目 |

## 生成された索引

6ファイルを結合した `next/reference/generated/icebook-panel-stories.tsv` は、Icebookへ渡す
story候補の検索用索引である。これは草案Markdownから
`python3 scripts/derive_icebook_panel_stories.py ...` で再生成し、手で編集しない。

## 領域

| ファイル | 領域 | 現在の正本 |
|---|---|---|
| `browser.md` | Browser / Media・Effects・Create・Panels | `next/ui/motolii-browser-pane` |
| `inspector.md` | Inspector / property・keyframe・effect・text | `next/ui/motolii-inspector-pane` |
| `stage.md` | Stage / hero・camera・gizmo・overlay | `next/ui/motolii-stage-pane` |
| `timeline.md` | Timeline / transport・timing・key・marker・audio | `next/ui/motolii-timeline-pane` |
| `export.md` | Export / range・format・audio・progress・recovery | `next/ui/motolii-export-pane` |
| `settings.md` | Settings / project・session・appearance・input・chrome | `next/ui/motolii-settings-pane`, `next/ui/motolii-menubar` |

## 見る順番

最初に各ファイルの30案を流し読みし、次に同じ問題を解く案を領域横断で比較する。
「普通の動画ソフトにあるか」ではなく、次の順で候補を残す。

1. P0/P1の制作停止・喪失・真実不一致を解くか
2. heroの主役・動機・空間表現を直接強くするか
3. 既存構造と先例へ預けられ、スクラッチを薄くできるか
4. 量が増えても視線と操作が崩れないか

採用候補はあとで`map_id`、`technical_route`、`scratch_policy`へ接続する。草案の数は完成機能数ではない。
まず各領域から、heroへの直接効果が高く、既存の意味・評価経路へ戻せる案を少数選び、
Icebookの実storyへ落とす。180案をそのまま実装しない。
