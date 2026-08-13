# 刻み込まれた編集ソフト文法とのギャップ地図(inherited grammar gap)

- 制定: 2026-08-12(利用者裁定「ユーザーは動画編集ソフトの文脈を脳に刻み込まれている。その部分のUXがMotoliiに足りない」)
- 位置づけ: [「普通に使える」品質バー](ui-quality-bar.md) Q1(一貫した操作文法)の**対象リスト**。ここにある反射は「機能追加」ではなく**負債**として扱う — 無いこと自体が違和感を生む
- **基準枠(2026-08-12利用者裁定)**: 刻まれた文脈とは**一般的な動画編集ソフトの共通文法**(Premiere / DaVinci / Final Cut / CapCut / AviUtl等の交差集合)である。モーションデザイン特化の独自文法(AE式stopwatch等)は補助参照に留める
- 優先順位の原理: **反射の深さ(無意識に手が出る度合い) × 現在の実装可能性**

## 核の一周 — 一般編集ソフトの最深文脈

一般的な動画編集ソフトのユーザーが無意識に期待する骨格は次の一周であり、これ自体が最優先の負債地図である:

| 段 | 反射 | Motolii現状 |
|---|---|---|
| **入れる** | Finderから動画/画像/音声をdrag&dropで投入、Browserに実サムネで並ぶ | **無**(media campaign。Browser MEDIAはダミー) |
| **並べる** | Timelineへdrag、移動/trim/複製 | **有**(本PRで成立。複製はwire口のみ残) |
| **切る** | playhead位置でsplit(razor/Cmd+K)、要らない断片をDelete | **無**(Splitコマンド不在=Tier 2筆頭) |
| **見る** | Space再生/停止、JKL、音が出る、timecode | **無**(playback campaign=Tier 0) |
| **書き出す** | Exportボタン→形式選んで書き出し | **無**(CLI exportは存在、UIなし) |

キーフレーム編集(本PRで厚く作った部分)はこの一周の**上に乗る装飾**であり、一周が欠けたままでは「一般的な編集ソフト」の体感にならない。

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
| M/S(mute/solo)直クリック | layer mute/soloのD2コマンド不在(実測) | 旧「object bar read-only」は2026-08-12裁定で撤回済み |

## (旧Tier 3は消滅 — 2026-08-12利用者裁定)

旧台帳のUX決定は[実機以前UX決定の降格](reviews/2026-08-12-pre-handson-ux-decision-demotion.md)により仮説へ降格済み。M/S直クリックは正当な反射として**Tier 2へ移動**(layer mute/soloのD2コマンドが不在のため仕様粒→実装の二段)。

## App全域の反射(2026-08-12追補 — 利用者裁定「刻まれた文脈は今回の実装外にも及ぶ。UIが見えた今こそ全てのUXを正しくする責任がある」)

編集面(Timeline/Stage/Inspector)に限らず、appの全表面が同じ反射の対象である。

### カーソル言語(最深級・全面共通)

pointerの形が変わること自体がaffordanceの発見手段として刻まれている。trim端=左右resize、clip/Stage物体=move/grab、テキスト=I-beam、splitter=行列resize、無効領域=標準。**現状Timelineはカーソル変化ゼロ**。hit判定は既存(gesture zone)なので配線のみ = Tier 1。

### Window / タイトル

- タイトルの**実project名**表示+未保存dot(●)。現状は `night_drive.mtl` の**ハードコード=嘘**(Q0級)。実名はhostが知っている = Tier 1
- Cmd+S=保存の手応え(自動永続でも反射は残る。保存済み表示で応える)、Cmd+N/O/Save As/最近使った = file dialog席のTier 2

### Stage

- **選択物のscale/rotateハンドル**(bounding boxの角と辺、回転ハンドル)。`SetProperty{Scale/Rotation/Anchor/Opacity}` は**D2に既存** → wire+UI配線のみ = Tier 1.5
- Shift+drag=軸拘束/等比、Alt+drag=中心基準 = ハンドルと同粒
- Stage自体のzoom/pan(pinch・Space+drag・Fit/100%実装) = Tier 1.5(view変換のみ、hit系は逆変換併走)

### Browser

- 項目のdragでStage/Timelineへ配置(CREATE sourceは現状double-clickのみ) = Tier 1
- Cmd+F=検索focus、リストの矢印キー移動+Enter確定 = Tier 1
- hover preview、サムネの実内容化 = media campaign従属

### Inspector / text field作法

- **Enter=確定してblur、Esc=打ち消してblur**、Tab=次のfield(X→Y)、focus時全選択(部分実装) = Tier 1
- 数値fieldの単位表示・矢印キー増減(Shiftで大股) = Tier 1

### 発見可能性

- iconボタンのhover **tooltip**(macOS標準toolTip) = Tier 1
- 空状態の一行ガイド(空Timelineに「Browserから配置」等、Q7と同件) = Tier 1
- 右クリックcontext menu(Tier 0再掲) — 全面に同じ献立を出す

## 検収

Tier 1系は面ごとに1 orderへ束ねる(oracleは品質バーQ0/Q1/Q7/Q9引用)。Tier 0は独立campaign、Tier 2は各仕様粒から。**「実装外の面」を後回しの理由にしない** — 全表面が同一の品質バーの下にある。
