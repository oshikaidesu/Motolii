# speed 編集の操作面調査(δ)— 先例意味論+正典整合(RESEARCH_RETURN 保全)

日付: 2026-08-21 / 発注: 後任セッション(SP1 発注の前段)/ レーン: read-only 調査(sonnet)
supervisor 採択: **候補3(2段構成)を採用** — SP1 第一波= Inspector 数値欄(map 963+269 消化)、第二波= Shift+bar端 rate stretch(Resolve の Shift 先例は質フラグ低につき、正典へ書く前に**利用者の実機確認**([[am-hands-on-verification]] の型)を挟む)。**speed 変更でキーフレーム時刻は動かさない**(Motolii の LayerTiming.speed は素材フレーム参照のみを動かす設計 — AE の「キーも比例スケール」は拘束7(a) の理由つき非借用)。

## 先例台帳(要点)

| ソフト | 操作面 | 従属関係 | キーフレーム相互作用 | 質 |
|---|---|---|---|---|
| AE Time Stretch | **ダイアログのみ**(端drag=trim のまま、speed 専用ジェスチャ無し) | %か新尺の一方を入力・他方自動。Hold in Place 3択 | **In/Out もキーも比例スケール** | 二次(Adobe 不達を再実測、複数一致) |
| Premiere Rate Stretch / Speed・Duration | 専用ツール(R)で端 plain-drag+数値ダイアログ併存 | 反対端固定・drag端=duration 直接、speed 従属 | effect keyframe は既定で動かない(優勢記述) | 二次(複数独立一致) |
| Resolve Retime Controls / Change Clip Speed | on-clip overlay の端drag+%入力+数値ダイアログ | 端drag は Premiere 同型 | speed point 追加時に Maintain Timing / Stretch Keyframes の明示トグル | 二次。**「Shift+drag端=speed」は単一ソース・質フラグ低(要実機確認)** |
| CapCut Speed パネル | パネルのみ(Normal スライダ+Curve) | 単一値スライダ/速度点 | 汎用 property keyframe 自体が無い | 低 |

## 正典整合(要点)

- modifier は hit-target ごとにスコープ(`write.rs:302,454` — bar 文脈の Cmd は snap-toggle で専有済み、コード確認)。**bar端 drag 文脈で Cmd は使えない。空き枠は Shift のみ**(Resolve 先例と一致)
- T4 の Cmd リタイムはキー菱形の hit-target — 元々衝突しない
- **speed はトリムに近い操作**(素材の窓を変えるだけ)— 正典§2 の「トリムでキーは動かない」の対に整合。AE の「キーも動く」を輸入すると Motolii の型が壊れる(拘束7(a))
- Inspector の既存 drag-to-scrub(`FieldDragState`)は Document property track 専用 — `LayerTiming.speed` は pane-local preview パターンの新設が要る(§5.5 の move/trim と同型)

## 候補案

- **候補1(第一波・採択)**: Inspector 数値欄(speed %、reset-to-100% 付き)。4ソース全会一致の先例。map 963(Time Stretch…)+269(Reset Clip speed)消化。write-set: `clip_gesture.rs`(speed⇄duration 純関数)・`inspector-pane`(Speed 欄+pane-local preview)・`write.rs`(SetTiming{speed} 発行腕)
- **候補2(第二波・実機確認後)**: Shift+bar端 drag = rate stretch(反対端固定・duration 直接・speed 従属、カーソル予告則へ新形状)。map 272/273 部分消化。候補1と同じ純関数・同じ Intent を共有(作り直しなし)
- 候補2b(不採用・記録): 専用 tool-mode 新設 — Motolii に tool-mode 概念が無く、1機能のためのサブシステム発明(拘束7 違反)。modifier で同じ結果が出る以上不要
- map 270(Reset Retime)はどちらも消化しない(可変速カーブ= §7 未決5 の管轄)

## EVIDENCE_GAP

1. Resolve「Shift+drag端=speed」は単一 WebSearch 要約由来・未 corroborated — **候補2 を正典へ書く前に実機確認必須**
2. Adobe ドメイン直接 WebFetch は今回も timeout 再現(二次要約経由のみ)
3. Premiere「keyframe は既定で動かない」は公式本文未確認(確信度中)
4. Inspector の Speed 欄の配置 section は未特定(SP1 発注書で EXACT TARGET として詰める)

## FINDING

1. `snap_enabled = !modifiers.command()` が `write.rs:302,454` で重複(3箇所目になる前に SP1 発注書で共通化可否を一言渡す)
2. Inspector の `FieldDragState` は将来 speed が animatable になった時(Time Remapping 採用後)はそのまま使える設計
