# 繋げるだけの口 — 残りの一覧(2026-08-28)

**この表は現在地であって歴史ではない。** 繋いだら消す。次のレーンはここから取る。

判定語: **観察**(測定結果)。想像ではなく `grep` で数えた。

## 数字

| 層 | 公開されている口 | front が呼んでいる | 残り |
|---|---|---|---|
| `Intent`(store へ書く) | 24 | **9** | 15 |
| `Engine`(絵と素材) | 18 | **3** | 15 |
| `motolii-export`(書き出し) | 10 | **0** | 10 |
| `motolii-audio`(音) | 73 | **0** | 73 |
| `motolii-vector`(ベクタ演算) | 34 | **0** | 34 |
| `motolii-shell-state`(Session) | 41 | 3 | 38 |

**バックはほぼ完成していて、front が呼んでいないだけ。** 2026-08-28 朝の時点で
`Intent` の接続はゼロだった — 9本は昨夜から今朝で繋がった分である。

## タスク表 — hero への距離順

hero = 実測の点群と、外で描いたベクタと、音が、同じタイムラインで出会う MV。
**「自分が1本作るのに要る物が先」**(裁定273)。

### 第1波 — これが無いと作業が始まらない

| # | やること | 使う口 | なぜ今か |
|---|---|---|---|
| S1 | **棚の資産をレイヤーへ割り当てる** | `Intent::SetSource` | `AdmitAsset` は繋がったのに `SetSource` が未接続 = **棚に入れた物をレイヤーにできない**。インポートの切れている線そのもの |
| S2 | **音が鳴る・波形が出る** | `motolii-audio`(`AudioProgram` / `MixProducer` / `PlaybackSession`)、`Engine::media_duration` | BPM グリッドへ**手で合わせる**とは聞きながら置くこと。鳴らなければ何も置けない。**73本の口が丸ごと未接続** |
| S3 | **素材が読めない時に理由が出る** | `Engine::layer_failures` | engine は失敗を溜めているのに front が読まない。**失敗が見えないのは失敗するより悪い**(窓を叩いても見えない嘘) |
| S4 | **レイヤーを消せる** | `Intent::RemoveLayer` | 今できない。編集の基本動詞 |
| S5 | **マーカー / ロケータ** | `Intent::SetMarkers` | BPM グリッドに印を置く。MV では必須 |

### 第2波 — 1本を仕上げるのに要る

| # | やること | 使う口 |
|---|---|---|
| S6 | **書き出し**(範囲・進捗・取り消し) | `motolii_export::{export_range_with_progress, export_with_cancel, export_still}` — **10本すべて未接続**。`export_surface.rs` の進捗バーは今どこにも繋がっていない |
| S7 | **カメラのキーフレーム** | `Intent::SetCameraTrack` |
| S8 | **comp 設定**(尺・fps・解像度) | `Intent::SetComposition` |
| S9 | **テキスト** | `Intent::SetTextDocument` |
| S10 | **シェイプの編集** | `Intent::SetShapes` + `motolii-vector`(34本未接続。trim-path / repeater / rounded-corners / pucker-bloat / zig-zag / offset-path / twist) |

### 第3波 — あると良い

| # | やること | 使う口 |
|---|---|---|
| S11 | マスク | `Intent::SetMasks` / `AddMask` |
| S12 | プロパティの参照・リンク | `Intent::SetPropertySlot` / `SetPropertyLink` / `SetSlots` |
| S13 | 棚の管理(消す・繋ぎ直す) | `Intent::RemoveAsset` / `RelinkAsset` |
| S14 | 凍結 | `Intent::Freeze` / `Unfreeze` |
| S15 | 市松(透過の可視化) | `Engine::render_frame_without_background` |
| S16 | マット合成 | `Engine::apply_matte` |
| S17 | キャッシュの状態を見る | `Engine::cached_frame_count` ほか2本 |
| S18 | Session の残り | `motolii-shell-state` 38本 |

## 走行中(2026-08-28 朝、workflow `wf_6462b31d-d22`)

統合1本 + 4レーン: **棚**(S1 を含む)/ **カメラ**(`render_frame_into_with_view_camera`)/
**区間イージング**(AM 式)/ **音**(S2 の入口)。

返ったらこの表を数え直す。**繋いだ行は消す。**

## この表の使い方

- **1行 = 1レーン**。write-set が互いに素になるよう組み合わせる
- **審判は3つ**(裁定271/272/274): AE に追いつく → Lottie、AE から離れる → 「仕方ない」試験、
  UI の分岐 → 意図論。**利用者に聞く前にこれを引く**
- **繋げるだけを自作に膨らませない** — 新しい型・trait を定義する瞬間に
  「これを既にやっている物は何か」を答える。**この表の存在自体がその答えである**:
  ほとんどの機能は既に在って、呼ばれていないだけ
