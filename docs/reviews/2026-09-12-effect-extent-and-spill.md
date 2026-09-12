# 効果の広がり・溢れ・下を読む — 合成の 3 つの法

2026-09-12 利用者報告(swiss.rrd の楕円に Blur): 「切れてるんですよ、画面外にも効果は出るべきなのに、それにグローも影響を出さない、あくまで自分の中だけに完結してしまう。発光してもしていないように見える」。
Motolii に、効果が層の外・他の層へ及ぶ時の法が無かった。

## 1. 広がりの法(退行修正、本日)

**効果は素材全体に、素材座標で評価する。層の広がり = 素材の範囲 + 効果が宣言した reach(`PADDING`)。view・comp・カメラは評価の入力を切らない。切るのは最終合成だけ。**

### 何が起きていたか

2026-09-11 の「輪郭を最終投影まで保つ」で、図形・文字は `path_model`(planar な網)になった。その層に絵を読む効果(Blur・Glow)が付くと、`render.rs` の `planar_image_pass` が `flatten_if_asked` で **comp と同じ大きさ・カメラを通した画面座標**に焼き、その後に効果を掛けていた。comp からはみ出した部分は焼く前に失われ、Blur は comp の縁で断ち切られる。PADDING は焼いた comp 大の絵に足されるだけで、失った外側は戻らない。画像の層は局所 texture + 余白の道なので切れない。図形・文字だけの退行。

「投影密度で描く」と「出力座標で描く」を混ぜたのが誤り。

### 直し

- `texture_for_resolved`: 絵を読む効果を持つ層は `vector = false`。素材座標・投影密度・内容範囲(`content_canvas`)で描き、`ImageFrame`(論理の大きさ・原点・画素数)を持ち帰る。図形も枠を返す。
- `Layer.frame: Option<ImageFrame>` を追加。`apply_material_domains` が warp 後の枠を載せ、`effective_layer_textures_in_frame` は呼び手の枠が無ければ層の枠で評価する(余白は密度倍、shader の `render_size` は論理の大きさ)。
- `planar_image_pass` の comp 大 flatten を撤去。flatten は利用者の `flatten` だけ。
- 論理 px の欄は **host が密度で画素へ写す**(`VismProgram::params_at_density`: `SUBTYPE` が DISTANCE / TRANSLATION の欄だけ)。shader は ISF の作法どおり `render_size` = 画素のまま書く。Blur・Glow の `radius` に `SUBTYPE: DISTANCE` を宣言(宣言 > 解析)。warp 段は uv 正規化で最初から密度に依らないので触らない。描画密度を上げてもぼけ幅・reach が変わらない。
- 余白(`PADDING`)は絵の画素なので、置く時は密度で割って論理 px にする(`sequential_inputs`)。
- 絵に描く時の密度は投影の密度そのもの(輪郭の細分だけ 2 の冪の段)。浮動小数の 20.000002 が 361 画素を生んで縁が甘くなるので 1/1024 に丸める。
- 副作用の修正: Blur がカメラの後(画面座標)で掛かっていたのが、板の面内で掛かるようになる(2.5D の傾いた板)。
- Codex 停止地点の取りこぼし: `reflection_cache.rs` の `SequentialContent::LinearRect` の match 漏れ(lib test が compile しなかった)。

### 実装中に出た 3 つの穴と塞ぎ方

- **上限越えの texture が pool を汚す**: 素材の絵 + reach が device の `max_texture_dimension_2d` を越えると wgpu は無効な texture を返し、それが effect scratch pool に入って**以後の全フレームが真っ黒**になる(実窓で Stage が何も描かなくなった原因)。密度の側で先に収め(`texture_for_resolved`: 密度 ≤ 上限 / (素材 + 2·reach))、pool の入口でも余白を削って収める。
- **費用**: 素材座標で密度どおりに描くと、comp より大きい層(Scale 818% の楕円)は数千万画素になり、Blur 1 段が 0.5 秒かかった。`raster_pixel_budget` = comp の 4 倍を絵の画素の上限にし、それを越える密度は落とす(効果は素材全体に掛かるが、費用は comp の定数倍で止める)。
- **Blur の算法**: 半径が大きい時の 193 tap の等倍ガウスは無駄。`blur.wgsl` を 3 段(等倍 / 1/4 / 1/16)にし、半径(画素)で段を選んで双一次で戻す(`FILTER: linear`)。swiss.rrd の 1 フレームは 547 ms → 20 ms(修正前の comp 大 flatten は 58 ms)。等倍との差は 20 倍拡大の円で最大 2 階調。

### 見た目が変わる所(利用者の裁定待ち)

1. **Blur の radius は層の px**。Scale 818% の楕円に radius r を掛けると、画面では 8r の幅でぼける(AE の「効果は変形の前」と同じ)。旧経路は comp 大に焼いてから掛けていたので画面 px だった。swiss.rrd の楕円は旧世界で調整された値なので、同じ見た目には radius を 1/8 にする。
2. **効果を掛けた図形・文字は 2.5D の置き場に残る**。旧経路は comp 大 2D に焼いていたので z を失い、常に積み順で重なった。今は Title(z=609)より楕円(z=0)が手前になり、赤いぼけが文字に被る。z=0 の世界を 2D とする裁定には沿うが、見た目は変わる。

### 審判

- `vector_projection_contract::blur_reaches_past_the_composition_edge`: 直径 48 の円を comp(64×64)の右縁に跨がせ Blur 8。comp を 128 に広げた絵と、重なる 64 列を比較。修正前 **528 画素**が違う(縁で切れる)、修正後 **20 未満**。縁の列がぼけていることも同時に確認(試験が空振りしない)。
- `blur_radius_is_measured_in_logical_pixels_at_any_raster_density`: 直径 16 の円を 20 倍に置いて Blur 4 と、直径 320 の円に Blur 80。shader 側で密度を掛けない状態では 20 倍側がほぼぼけない(soft 7339 画素 vs 116400)。host 側の写しで一致。
- 既存 `enlarging_a_path_matches_drawing_the_large_contour_even_at_an_image_effect_boundary`(絵の効果あり)は、絵の道へ移っても 300 画素未満を保つ。
- render lib 95 成功・8 除外。native 70 成功・1 失敗(`the_cage_follows_the_drawn_text_under_an_orbited_camera`、wip commit 時点で既に失敗 — Codex の未完、別件)。
- 実窓: swiss.rrd を開き直し、Camera View・99 フレームで楕円の Blur が comp の上縁で切れない。切り分け用に `examples/render_doc.rs`(作品を 1 フレーム描いて PNG、present 経路の再現、層の絞り込み)を足した。

## 2. 溢れの法(次)

**効果の出力のうち素材の coverage の外へ出た分(光・影)は、層の Blend と独立に、下の絵へ光として合成する。**

先例: Photoshop の layer style は Outer Glow が既定 Screen、Drop Shadow が既定 Multiply を**効果自身が**持ち、層の blend とは別。Figma の effects も同じ。AE は層の blend を Add にしないと光らない(「そういうものだ」なので引き継がない)。

形(実装済み、2026-09-12): manifest に `"SPILL": "screen" | "add" | "multiply"`。効果の鎖の出口 1 箇所(`effective_layer_textures_in_frame`)で、SPILL を持つ層だけ出力を素材の coverage で内と外に分け(`matte_by_coverage`、生成器の閉じ込めと同じ matte program の mode 0/1)、外は同じ置き場の 2 枚目の入力として宣言の混ぜ方で積む(`sequential_inputs`)。内は層の Blend のまま。効果作者は分岐しない。Glow が `SPILL: screen` を宣言。

ついでに、焼く経路(mix 系 blend の矩形)が `LinearRect` で `unreachable!` に落ちる穴を `SequentialContent::image()` で塞いだ。

審判 `domain_contract::a_glow_halo_spills_as_light_independent_of_the_layer_blend`: Normal の青い矩形に Glow。白地では素材の外が白のまま(screen は白を変えない — SPILL 無しだと 1968 画素が濁る)、黒地では halo が 200 画素以上光る。

## Glow の作り直し(2026-09-12、実装済み)

利用者「Deep Glow の画は素晴らしいが実装は過剰」。取説を読んで、Bevy の bloom(Call of Duty: Advanced Warfare の作法、MIT)を 1 つの Vism に畳んだ。ID `motolii.glow` と `threshold / intensity / radius` は据え置き(保存済みの作品はそのまま開く)。

- **中身**: 余白込みの絵を 13 tap で半分ずつ 6 段に落とし(初段だけ Catlike Coding の soft knee と Karis 平均)、3×3 tent で戻しながら段ごとの重みで足す。ISF の `PASSES` の `$WIDTH/2 … /64` と `FILTER: linear` だけで書けた。runtime の変更なし。
- **札**: Threshold・Softness(knee)・Intensity(HDR、1 を越えて足す)・Radius(届く段 = log2 radius)・Spread(遠い段の持ち上げ、Bevy の low_frequency_boost)・Anamorphic(標本間隔を x に伸ばす、Bevy の scale)・Chromatic(深い段ほど YIQ で色相を回す、外周ほど色が変わる)。Tint(光の色)は ISF の color 欄が pass の params に乗らないので宿題。
- **嘘**: 段の和は正規化しない。近くは全段が重なって白く飛び、遠くは深い段だけで淡い — bloom の形。正規化すると光が消える(最初の版で確認)。
- **溢れ**: `SPILL: screen` で halo は Normal の層でも周りを照らす。
- 見本: `domain_contract::glow_gallery_for_the_eye`(`MOTOLII_GLOW_EVIDENCE=dir`)で既定・Radius 16・Radius 256+Spread・Chromatic・Anamorphic の 5 枚。

## 3. 下を読む法(後)

Figma の Background blur、ガラスの屈折のように、下に描かれた絵を入力にする効果。glass 用の「run を切って backdrop の mip を渡す」口を画像 pass にも開く。

## 3 者の対応

| | 形を動かす | 絵として扱う | 溢れ | 下を読む |
|---|---|---|---|---|
| vgpu | draw | effect | host の pass 構成 | host の pass 構成 |
| Blender | Modifier | Visual Effects / Compositor | Compositor | Compositor |
| Photoshop / Figma | — | filter | layer style の blend | Background blur |
| Motolii | field / surface | pass | `SPILL` | backdrop 口 |
