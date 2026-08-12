# 刻み込まれた編集ソフト文法とのギャップ地図(inherited grammar gap)

- 制定: 2026-08-12(利用者裁定「ユーザーは動画編集ソフトの文脈を脳に刻み込まれている。その部分のUXがMotoliiに足りない」)
- 位置づけ: [「普通に使える」品質バー](ui-quality-bar.md) Q1(一貫した操作文法)の**対象リスト**。ここにある反射は「機能追加」ではなく**負債**として扱う — 無いこと自体が違和感を生む
- 優先順位の原理: **反射の深さ(無意識に手が出る度合い) × 現在の実装可能性**

## Tier 1 — 既存の器で即撃てる反射(wire/UI配線のみ。最初に消化)

| 反射 | 期待挙動 | 現状と接続先 |
|---|---|---|
| ←/→ frame step | 矢印でplayheadが1フレーム動く(Shiftで10) | `set_time`既存。keymap追加のみ |
| Esc | 選択解除/進行中gestureのcancel | `clear_selection`既存・cancel文法既存。ObjC keymapにEsc未処理(実測) |
| Cmd+D 複製 | 選択layerの複製 | **`duplicate_track_item`がmotolii-docに既存**(duplicate.rs:44)。wire口+keymapのみ |
| キーフレームnavigator | Inspector/行の ◀ ◆ ▶ (前のキーへ・キー追加/削除・次のキーへ) | prev/next=`set_time`(キー時刻はwire済み)、◆=`add/remove_position_key`既存。UI席のみ |
| Stage矢印nudge | 選択物を矢印で1px(Shiftで10px)動かす | `move_layer_by`既存。keymapのみ |
| Shift+Z / fit | 作品全体をfit表示、`Cmd+=`/`Cmd+-` zoom | view機構既存。keymap+Fitボタン接続(Q0 inventoryと同件) |
| 数値のdrag-scrub | InspectorのX/Y等をラベルdragで増減 | DialParameter既存(fixtureで実証済み)。実データ行へ流用 |
| Home/End | playheadを先頭/末尾へ | `set_time`既存 |

## Tier 0 — 最深の反射(campaign級。名前を付けて計画する)

| 反射 | 中身 | 依存 |
|---|---|---|
| **Space = 再生/停止** | 編集ソフトの反射の頂点。JKLシャトル・timecode実表示・playhead追従スクロール・再生中のStage連続評価が一族 | playback spine(PlaybackSession接続、audio clock)。transport一式のQ0違反もここで解消 |
| **右クリック menu** | 文脈操作の普遍的な逃げ道(削除・複製・rename…) | context menu基盤(ObjC)。中身はTier 1/2の再掲で良い |

## Tier 2 — D2/モデルのgrainが要る反射(仕様粒→実装の二段)

| 反射 | 欠け | 備考 |
|---|---|---|
| **Split at playhead**(Cmd+K/razor) | D2にSplitコマンド不在(実測) | SetPositionKeyTime/Remove対で確立した鏡映playbookで新設可 |
| Rename(名前double-click) | renameコマンド不在(R2地図でも既知) | layer_names台帳は存在 |
| 複数選択(Shift+click/marquee) | selection modelがsingle primary | U2h系譜と整合させる仕様粒 |
| Copy/Paste | コマンド不在 | duplicate既存が下敷き |
| I/O点・work area | range概念が未設計 | 台帳の「Preview Range/Loop/Trim比較中」row参照 |

## Tier 3 — 台帳決定と交差する反射(覆すのは利用者のみ)

| 反射 | 交差する決定 |
|---|---|
| M/S(mute/solo)を行で直クリック | 「object barは読み取り専用、状態変更はInspectorとkeymap」(2026-08-08 Timeline設計)。刻まれた反射との衝突を**反証材料として記録** — 本書は覆さない |

## 検収

Tier 1は一括で1〜2 orderに束ねられる(全て既存intentへの配線)。oracleは品質バーQ0/Q1/Q9を引用。Tier 0は独立campaign、Tier 2は各仕様粒から。
