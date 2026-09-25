# BB劇場を異常に高品質な光で — renderer 探索の結論を実画で(2026-09-25)

renderer 探しを閉じた結論(Surface / Lighting / Image formation は交換可能、Motolii semantics へ renderer の型を漏らさない)を、scratch の Lighting Pack 1 枚で実画にした。renderer 選定は再開していない。

## 画

同じ構図・同じ作品(`2026-09-25-bb-theater/bb.js`、素材は `gen_assets.py` で生成)。1920×1080、M4 / Metal。

| | 画 | 何が違うか |
|---|---|---|
| 1 | `1_current.png` | 今の Motolii: 環境の最も明るい方向を太陽にした照明、512² の型紙の影、`saturate` |
| 2 | `2_surface_lighting.png` | Surface と Lighting だけ Pack に替えた。画の形成は今の host のまま(`saturate`、露出無し)→ 窓の光が白く飛ぶ |
| 3 | `3_beauty.png` | 2 + 画の形成(露出 1.55、AgX、弱い look) |
| 4 | `4_off.png` / `4_off_frame60.png` と `3_beauty_frame60.png` | Lighting OFF / ON(Pack は描く、光だけ無し。画の形成は 3 と同じ) |

一覧は `contact_four.png`、OFF/ON は `contact_off_on.png`、カメラ移動中の beauty は `contact_motion_beauty.png`(frame 0 / 30 / 60 / 90 / 119)。

## 2D 素材がどう光に参加したか(3D asset に焼き直していない)

- **立ち絵 PNG 2 枚**: coverage(alpha)がそのまま遮蔽。窓光の柔らかい接地影、床・壁への影、間接光の遮り。
- **左壁 PNG(窓の穴が透明)**: 穴だけ光が通る = gobo。窓の十字の影、床と女の子に落ちる窓光、空気中の光の筋はこの PNG の alpha から出ている。
- **赤い字幕(文字、Glow 付きの Group)**: 面光源。少年と女の子の髪の上だけに赤が乗る。壁全体は赤くならない。
- **ラグ(shape)**: 赤い面として間接光に参加。
- **窓の外 PNG(Glow)**: 窓越しの空の光として部屋へ入る。
- **キャプション帯(2D、flat)**: 光に参加しない。上に合成するだけ。

## どの pass が画のどの特徴を作ったか

| pass | 作った特徴 |
|---|---|
| atlas(絵を 128² の slot へ、mip で平均まで) | 光の問い合わせ用の coverage と色。字幕の平均色 = 面光源の色 |
| 形の絵(平面 mesh → 絵、同じ mesh・同じ大きさの間は使い回し) | 床・壁・天井・ラグ・文字が Renderable になる |
| 光の場 1(22×11×22 の probe、48 cone、L1 SH、毎フレーム) | 窓光が床・壁で跳ねた 1 回目の光。部屋の奥・隅の暗さ |
| 光の場 2(1 を読んで同じことをもう一度) | 2 回跳ねた光。影の中の明るさ・色の回り込み |
| front(alpha > 0.5 の手前の面の normal と picture) | 半分の大きさの pass の入力 |
| near(半分の大きさ: 6 cone の接触 AO + 空気) | 足元・壁際の接触の暗さ、窓からの光の筋(光を遮る絵の穴を通った光だけ空気が光る)、字幕の周りの空気の赤み |
| color(絵を遠い順に、premultiplied で重ねる) | 直接光: 窓光(遮る絵を cone で抜ける柔らかい影、遠いほど半影が広い)、字幕の面光源(形状係数 + 遮蔽)。光の場は背面の probe を外して読む |
| formation | 1〜2: 今の host と同じ `saturate`、3〜4: 露出 + AgX + look |

## 時間(M4 / Metal、1080p、`script_frames` の `MOTOLII_TIMING=60`: カメラが動く 60 コマの平均、CPU 準備と毎コマの読み戻しを含む wall)

| | ms / コマ(3 回) |
|---|---|
| 1 今の Motolii | 27.1 / 27.9 / 28.5 |
| 2 Surface + Lighting | 31.3 / 31.3 / 32.3 |
| 3 beauty | 31.5 / 31.4 / 31.5 |
| 4 Lighting OFF | 22.9 / 22.9 / 24.2 |

作品は 30 fps(33.3 ms)。Pack の上乗せは約 4 ms。最初の版は 126 ms で、次の手直しで下げた: 前面描画で重なった面の全部を陰影していた計算を半分の大きさの deferred pass へ移した / probe を 40×20×40×96 → 22×11×22×48 / 形の絵の使い回し / OFF の時は光の pass を走らせない。GPU 単独の時間は取っていない。

## 常時リアルタイムと決定性

- 履歴を使わない。毎コマ、そのコマの絵だけから光の場を作り直す(停止後に収束する画ではない)。カメラ移動中のコマ(`contact_motion_beauty.png`)に履歴由来の破綻は無い。
- wall clock を新しく読んでいない。空気のサンプルの揺らぎは画素位置で固定(IGN)。seek / export の byte 一致は検査していない。

## scratch としての読み替え(決定ではない)

- **Cast Shadow** = 窓光を遮る。空気が光るのは遮る絵の穴を通った光だけ。
- **Glow** = 光を出す。強さは Glow の Intensity。host の `Layer` / `SequentialInput` / `LayerWork` に `emission` を 1 つ足し、`shadow` と同じ経路で運んだ。
- 3D の絵はすべて間接光と AO に参加する。spill(blend が Normal 以外)は面ではないので外す。
- Pack が照明を持つ時は host の型紙の影を取らない。
- Pack の入口は view の run(`SequentialInput` の列)だけ。Pack の型は Motolii semantics へ出ていない。切り替えは `MOTOLII_LIGHT_PACK=on|off` / `MOTOLII_IMAGE_FORMATION=film|clamp`。WGSL は `MOTOLII_LIGHT_PACK_WGSL` で実行時に読むので、look の調整に Rust の build は要らない。

## 途中で見つけた host の穴(今回は直していない)

- 3D に置いた文字・形にベタ塗りが届かない(既知)。文字は Group に入れ、形は `linear-gradient` で避けた。
- 今の Motolii では、2D のキャプション帯に世界の型紙の影が落ちる(`1_current` の帯の右)。
- 文字・形は絵ではなく平面 mesh として run に来る。Pack 側で絵にした(Surface realization の役目)。
- Group は効果が乗った時だけ一枚の絵(plate)になる。

## 残り(画の質)

- 鏡面反射が無い(床の艶、窓の映り込み)。
- 光の場の格子が全部の絵の AABB を覆う。床が大きいと無駄が多い。
- 半分の大きさの AO を bilinear で戻すので、輪郭に薄い滲みが出る。
- キーライトは環境の太陽 1 本だけ。

コード: `motolii/crates/motolii-render/src/compositor/light_pack.rs` と `light_pack.wgsl`。host 側は `view.rs` の run 分岐と `emission` の配管だけ。
