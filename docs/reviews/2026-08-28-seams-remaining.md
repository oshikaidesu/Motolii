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

## 動線 = 製品の最小コア(2026-08-28 利用者裁定)

**「普通に使えるようになる」までが最小コア。** その動線は4駅で、責任は駅ごとに区分けする:

| 駅 | 持ち主(write-set の家) | 口 |
|---|---|---|
| 1. ブラウザで取り込む | `browser_surface.rs` | `AdmitAsset` → `SetSource` |
| 2. タイムラインに出る | `timeline_surface.rs` + projection | 投影(読み)+ `SetTiming`/`SetOrder` |
| 3. インスペクターでいじる | `inspector_surface.rs` | `SetAttrs` ほか property 系 |
| 4. 書き出す | `export_surface.rs` | `export_still`(動線分。全10口は S6) |

`main.rs` は**薄い配電盤**であって駅ではない。駅の処理を main.rs に書くと区分けが漏れる
(実測: 2026-08-28 の4レーン統合で3本が main.rs 衝突。区分けの漏れはマージ痛として請求される)。

**動線オラクル**: 素材 drop → レイヤーが出る → Inspector で1値変更が絵に映る →
`export_still` のファイルが実在する。`--remote` 窓で毎検収時に通す。
**「駅が在る」と「駅が繋がっている」は別**(実測: export は surface も進捗バーも在るのに
口10本が全部未接続だった)。在るかどうかではなく、この一本が通るかで数える。

## 背骨 = 最小経路の確定(2026-08-28 利用者裁定)

「普通に使える」の中身が確定した。**この5つで、それ以上を求めると最小経路ではない**:

1. **素材インポート**(WIRE-2 — 走行中)
2. **ギズモによる位置調整**(S20 を最小経路へ繰り上げ)
3. **キーフレームの設置・配置** — 区間キーフレームは **Flow / Alight Motion 様式の UI**
   (利用者裁定: プリセット/型の見える面。deltas の「薄いポップアップで足りる」を上書き)
4. **再生による音ハメ**(背骨は着地済み — 音声クロック再生・波形・ロケータ・クリップ移動)
5. **レイヤーの移動**(`SetClipTiming(edge: None)` で着地済み)

**書き出し(S6)は最小経路の外**(利用者の列挙に無い。第2波のまま)。

**仮設路の禁止**(利用者裁定「最小経路用の道を作らないこと。あくまで今後の拡張を設ける。
背骨です」): 最小経路のために使い捨ての近道を建てない。ギズモは最初から単一操作系
(transform-gizmo、S20)、イージング UI は `Interp` 正本の上の本物の面。後で剥がす物を作らない。

## タスク表 — hero への距離順

hero = 実測の点群と、外で描いたベクタと、音が、同じタイムラインで出会う MV。
**「自分が1本作るのに要る物が先」**(裁定273)。

### 第1波 — これが無いと作業が始まらない

| # | やること | 使う口 | なぜ今か |
|---|---|---|---|
| S4' | **Delete の実キー合否** | (結線済み) | `f7a17703` で `Intent::RemoveLayer` は Delete/Backspace に結線され check/test 緑。ただし**実窓合否が取れていない** — `--remote` の `/k` は app レベルの key handler に届かない(既存の Cmd+scale も同様に無反応。器具の限界、fork AGENTS.md に記録)。**実キーボードでの確認待ち**。効かなければ key focus 経路の欠陥 |

**消えた行**(2026-08-28 統合 `7856cb65` / 第1波 `f7a17703` で着地、実窓で確認):
- ~~S3 失敗の可視化~~ — `layer_failures` → Stage chrome の失敗帯(空なら沈黙、これが正常系)
- ~~S4 レイヤー削除~~ — 結線は完了、実キー合否のみ S4' として残す(上表)
- ~~S5 ロケータ~~ — 実窓で3動詞とも確認: 右クリックで置く(「marker placed at frame 758」)・
  左クリックで playhead が跳び Stage も追随・もう一度右クリックで消える。fixture の
  マーカー3枚も描かれる
- ~~S5b 選択→Inspector~~ — 実窓で確認: Credits 選択でヘッダが「クレジット · Solid layer」
- ~~S1 棚→レイヤー~~ — カード double-click(`PlaceAsset` → `AddLayer`+`SetMeta`+`SetAttrs`)で
  playhead/最前面に立つ。実窓で glow_default が立ち ● バッジ・status 行まで確認。
  **`SetSource` 自体は依然未接続**(既存レイヤーの繋ぎ直し = S13 の隣)
- ~~S2 の入口~~ — 音声 drop → frame 0 にレイヤー+波形形状(wf-5)
- ~~S2 再生の背骨~~ — Space/transport が `AudioProgram::from_view` → `PlaybackSession` を
  開き、**再生中の playhead は音声クロックから導出**(wall timer は再描画の口実のみ。
  P07-C1D の写し)。scrub は `session.seek`、comp 末尾で自動停止(実窓: frame 1799 で
  PAUSED)、デバイス不在は status に理由を出して無音継続。tick では seek しない
  (ring の再基底化は音を割る)。**残るのは実音の耳確認**(headless では検証不能)と
  meter/gain/pan UI 等の残り口

### 第2波 — 1本を仕上げるのに要る

| # | やること | 使う口 |
|---|---|---|
| S6 | **書き出し**(範囲・進捗・取り消し) | `motolii_export::{export_range_with_progress, export_with_cancel, export_still}` — **10本すべて未接続**。`export_surface.rs` の進捗バーは今どこにも繋がっていない |
| S7 | **カメラのキーフレーム** | `Intent::SetCameraTrack` |
| S8 | **comp 設定**(尺・fps・解像度) | `Intent::SetComposition` |
| S9 | **テキスト** | `Intent::SetTextDocument` |
| S10 | **シェイプの編集** | `Intent::SetShapes` + `motolii-vector`(34本未接続。trim-path / repeater / rounded-corners / pucker-bloat / zig-zag / offset-path / twist) |
| S10c | **点群の縦スライス**(小さい .ply 1個が場面に出る) | 棚は既に迎え入れ済み(`Asset.asset_type` は opaque、`pointcloud.octree.v1` が asset.rs:109 に例示済み)。**engine 側の読み・描きが0件**。委託先: 読み込み=rerun の loader 資産、描画=re_renderer の point cloud renderer(本業)。自作は繋ぎだけ。LOD/streaming(1億点)は縦スライスの後 |

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
| S20 | **Stage 直接操作**(掴む=ギズモ) | 利用者裁定(2026-08-28)「普通に使う分には必要」「3D は 2D の下位互換なのでそのまま持って来ていい」。**操作系は1つ**(憲法2: モードの切り替えを作らない — 平面用/空間用の2階建て案は総監督が出し利用者が却下)。借用先確定: `transform-gizmo` crate(MIT/Apache、renderer 非依存 — `Gizmo::draw` が viewport 座標の頂点を返す、mint/glam 互換)。z=0 のレイヤーは退化ケースとして同じギズモで掴む。土台=rerun の GPU picking(`re_view_spatial/picking.rs` 一式、instance 単位読み戻し)+ `re_renderer` line overlay(同一 ViewBuilder=第二表示経路を作らない)+ `Intent::SetAttrs`。rerun 本体に編集ギズモは無い(実測: "gizmo" は原点軸表示1箇所のみ)。未決はハンドルの皮(AE 箱型か軸矢印か)だけで、同一操作系の上の見た目 — 窓の UX 合否で決める。**前提 = 選択の真実が Stage へ流れること**(S5b の線の続き) |
| S19 | **BPM グリッド**(小節線・ビートスナップ) | **口が無い — 語彙ごと無い**(実測: store 全体で bpm/tempo/beat 0件)。利用者裁定(2026-08-28): **LFO 自動制御(ParamDriver)の拡張と同じ束**なのでモデル上の置き場はその時に決める。乗り物は Lottie の車体(marker/meta の慣習)で行ける見込み — 新 component を先に切らない。v1 の hero 動線は S5 ロケータ+耳(聞きながら置いて印を打つ)で成立する。先例=Ableton |

## 着地済み(2026-08-28、統合 `7856cb65`)

workflow `wf_6462b31d-d22` の4レーンを main へ cherry-pick で統合(統合レーンは中断されたため
supervisor が引き取り)。**棚**(→S1 消し)/ **カメラ** / **区間イージング**(モデル層のみ —
`Interp::Bounce/Elastic/Steps` と `Interp::ease`。**INTERVAL EASING の選択板は統合で落とした**、
main 側の統一 action 設計が新しいため。front 入口は次の波)/ **音**(→S2 の入口消し)。

門: check 緑・テスト42本全緑・実窓で駅1→2 通過(カード double-click → レーン成立)。
発見: 駅3未結線(→S5b)、import の WIRE-2 は畳まれたまま(`IMPORT_WIRED=false`)。

## この表の使い方

- **1行 = 1レーン**。write-set が互いに素になるよう組み合わせる
- **審判は3つ**(裁定271/272/274): AE に追いつく → Lottie、AE から離れる → 「仕方ない」試験、
  UI の分岐 → 意図論。**利用者に聞く前にこれを引く**
- **繋げるだけを自作に膨らませない** — 新しい型・trait を定義する瞬間に
  「これを既にやっている物は何か」を答える。**この表の存在自体がその答えである**:
  ほとんどの機能は既に在って、呼ばれていないだけ
