# 残作業の割り振り(機械導出)

`scripts/plan_backlog.py` が生成。**手で編集しない。**

**「普通の動画ソフト」の定義は `normal-map.tsv` が持っている。** 残っているのは
`verdict` が `採用予定`/`結線待ち` の行で、どの pane が持つかは `bundle` →
`intent-bundles.tsv` の `home` から引ける。順番は `freq`(4製品中いくつに在るか)降順。

- 残 51件 / crate 5本 / 家が未決 6件

## crate ごと(重い順)— **crate が違えば同時に走れる**

| crate | 残 | freq≥2 | 束 |
|---|---|---|---|
| `ui/motolii-timeline-pane` | 26 | 1 | B15 キーフレーム束, B19 マーカー/フラグ束, B20 再生ヘッド移動束 ほか3 |
| `ui/motolii-stage-pane` | 6 | 0 | B17 カメラ/3Dビュー束 |
| `ui/motolii-menubar` | 6 | 0 | B11 File束(MB-1延長), B33 クリップボード/履歴束(MB-0延長) |
| `ui/motolii-browser-pane` | 5 | 1 | B08 素材取り込み束 |
| `ui/motolii-inspector-pane` | 2 | 1 | B37 速度/リタイム束 |

## 最優先(freq≥2 — 4製品中2つ以上が持つ = 普通度が高い)

| crate | id | freq | 項目 | 意味 |
|---|---|---|---|---|
| `ui/motolii-timeline-pane` | 1315 | 2 | New Sequence / Timeline / Composition | 新規シーケンス・タイムライン・コンポジションを作成 |
| `ui/motolii-browser-pane` | 167 | 2 | Replace (footage/clip) | 素材・クリップを置換 |
| `ui/motolii-inspector-pane` | 169 | 2 | Speed / Duration | クリップの速度・長さを変更 |

## 家が未決 6件(束が `home` を持たない/語で引けない)

**推測で割り当てていない。** 束の `home` を決めるのが先。

- B32 ロック/可視性/リンク束: 3件
- B36 新規コンテンツ作成束: 2件
- B29 ツール切替束: 1件
