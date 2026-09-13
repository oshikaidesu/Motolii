# Freeze と Flatten の法 — 固めるのは投影の前か、後か

制定: 2026-09-13。利用者裁定「意味を先行させる。ユーザはなぜフリーズを使うか」から。
先例: DAW の Freeze / Flatten(Ableton)、Bounce in Place(Logic)、Blender の bake、Houdini の file cache、
C4D の「現在の状態をオブジェクトに」、AE の pre-render / proxy。
関連: [時間の法 §6](plugin-resources.md)(feedback は host が持つ)、[Hold = `TIME_AT`](vism-shadertoy-import.md)(静止フレームは効果の 1 枚で、ここには入らない)。

## 0. 一文で

**Freeze は「投影の前」の結果を固めて 3D のまま軽くする。Flatten は「投影の後」の絵を α 付きの素材にして流用する。**
DAW は信号が 2 次元(時間 × 値)なので両方とも同じ音声に落ちるが、こちらは投影があるので 2 つは別の札。

## 1. 意図(なぜ固めるか)

| 意図 | DAW で | MV 制作で | 札 |
|---|---|---|---|
| 再生が引っかかる。作業中の他の層のために計算を返す | 重い楽器・effect の track を Freeze | 残像・別時刻・重い shader の層があると、他を触っている間もスクラブが重い | **Freeze** |
| 結果を素材にして流用する。1 回し目を焼いて 2 回し目で切り刻む・逆再生・time remap・datamosh(絵にしか掛からない物) | Flatten / Bounce in Place、凍らせたクリップを別 track へ | 群の合成をカメラ込みの動画にして、別の層として置く | **Flatten** |
| もう決めた。事故で触らない | Freeze の灰色 | 利用者はあまりしない。Freeze の副作用として付いて来るだけ | Freeze の拒み |
| 別の機械・後日でも出る | plugin が無い機械で凍った音声が鳴る | shader を消した後・他人の機械でも絵が出る | Freeze の cache を**書類の隣に置く** |
| 聞いた物がそのまま出る | freeze = 書き出しと同じ計算 | プレビュー = 書き出しの 1 本道 | 既に成立。Freeze / Flatten はその道の結果を保存するだけ |

## 2. Freeze — 投影の前を固める

### 2-1. 何を固めるか

層(または群 = Whole の境界)の**素材としての結果**を、層の入点〜出点のコマごとに:

| 素材 | 固める物 | 今それが出来る所 |
|---|---|---|
| 板(動画・画像・文字・形の絵) | 効果の列を通った層の絵(乗算済み線形、余白込み) | `effective_layer_textures`(compositor/render_effects.rs)が層の絵を出す所 |
| 網・点群・線(場を通った物) | 場を通った後の頂点・点(コマごと) | 場の hook を通った mesh / 点群が run の view へ渡る所 |
| 群(Whole) | 板と同じ(1 枚に焼かれた絵) | `bake_isolated_layers` |

固めるのは**投影の前**なので、3D は 3D のまま。凍った網はカメラを動かせば違って見え、他の層と深度で刺さり合う。
これが Flatten との境。

### 2-2. 生きている物

DAW の mixer に当たる物は生きたまま: 配置(位置・回転・拡縮・Z)、不透明度、blend、マット、クリッピング、重ね順、時間上の移動、カメラ。
**時間上の移動**は cache が層の時刻で引かれる(入点からの番号)ので、層を動かしても cache はそのまま使える(DAW でクリップを動かすのと同じ)。

### 2-3. 拒む物

素材・効果・欄・キーフレーム・中の子・マスク・場の欄。cache と食い違う編集は拒む(DAW の灰色)。
拒みは既にある(`Intent::Freeze { group }`、`check_not_frozen`)。層 1 枚へ広げる。
拒んだ時の文は「凍っている。Unfreeze で戻す」の 1 行で、黙って無視しない。

### 2-4. 可逆

Unfreeze = cache を捨て、拒みを解く。書類は Freeze の前と同じ(Freeze は書類に `frozen` の旗と cache の在処を持つだけで、素材も効果も変えない)。

### 2-5. cache の置き場と形

- **書類の隣**(`<name>.rrd` の隣の `<name>.motolii-cache/<layer>/<frame>.exr` など)。VRAM だけだと再起動で消えて「決めた」が崩れ、他の機械でも出ない。
- 板の絵は**乗算済み線形の half float**(今の効果の列の出口と同じ空間。EXR half か raw)。sRGB 8 bit に落とさない — 効果の列の出口を保存するのであって、書き出しではない。
- 網・点群は頂点・点の列(層ローカル、場を通った後)。
- cache は**使い捨て**で書類の真実ではない(concept の「cache は document truth にならない」)。無ければ Freeze を解いて描き直せる。

### 2-6. 不変条件(審判)

- **凍っても絵は変わらない**: 任意の t で `render(frozen)` == `render(unfrozen)`(誤差は保存形式の丸めだけ)。
- **飛んでも辿っても同じ**: 凍った層は cache から返すだけなので、feedback の辿り直しは走らない(時間の法 §6-3 の答えを cache が先に出している)。
- **書類は同じ**: Freeze → Unfreeze の前後で書類の指紋(`revision_key`)が同じ。
- **拒みは名指し**: 凍った物への編集は理由付きで断る。

### 2-7. やらない事

- **暗黙の cache**(AE の緑のバー、触った所から無効)は別の話。この法の Freeze は明示・全区間・捨てない。暗黙の cache を後で足すなら、同じ置き場・同じ形を使う。
- 凍った層の中を「一部だけ」触れる仕組み(部分的な無効化)。触るなら Unfreeze。

## 3. Flatten — 投影の後を素材にする

### 3-1. 何をするか

層(または群)の入点〜出点を、**カメラを通した絵**として書き出し、Browser の Media に**素材として登録**する。
元の層はそのまま(触れる)。増えるのは素材 1 つ。差し替えたければ元の層を消すか隠す — 普通の操作で、特別な状態を作らない。

- 書き出しの道はプレビューと同じ 1 本(`render_frame` → export)。Flatten の絵 = そこで見た絵。
- **α を持つ**: 素材として上に重ねるため。形式は **ProRes 4444**(利用者裁定 2026-09-13、mac の普通、α あり)。
- 素材の 0 秒 = 層の入点。層の外には出ない(DAW のリージョン)。
- カメラ: 作中カメラ(出力)。Stage の窓ではない。

### 3-2. 不可逆

Flatten は素材を作る動詞で、書類の層は変えない。取り消しは「素材を消す」。

### 3-3. 名前

書類の `flatten`(3D を板に収める、2026-08-30「平面に収めるのは選択肢」)と**名前が衝突する**。
UI に出す時は、この法の Flatten を **Bounce**(Logic)か **Render to Media** と呼び、書類の `flatten` はそのまま。
どちらの語にするかは窓に出す時の利用者裁定。この文書では DAW の語に合わせて Flatten と書く。

## 4. 2 つを混ぜない

| | Freeze | Flatten |
|---|---|---|
| 固める段 | 投影の前 | 投影の後 |
| 3D | 残る | 消える(それが目的) |
| 書類 | 旗 1 つ + cache の在処。素材も効果もそのまま | 素材が 1 つ増える。層は変わらない |
| 可逆 | Unfreeze | 素材を消す |
| 重さ | 返す | 返す(素材を置いた側は動画 1 本) |
| 形式 | half float(効果の列の出口)/ 頂点の列 | ProRes 4444 |

Hold(`TIME_AT`)は「ある瞬間の絵を読む効果」で、固めるのではなく読む。3 つは別の札。

## 5. 順番

1. Freeze の板の道(絵の列の cache、拒みの層への拡張、Unfreeze、審判 2-6)
2. Freeze の立体の道(頂点・点の列)
3. Flatten(ProRes 4444 の書き出し → Media 登録)

## 6. 現在地(2026-09-13)

**板の Freeze を実装**(§5 の 1)。層にも群にも `Intent::Freeze`。裏の thread(`ui/native/src/freeze_job.rs`、export と同じ型)が
入点〜出点を順に焼き(`Engine::freeze_bake_frame`)、書類の隣 `<name>.motolii-cache/<layer>/<frame>.rgba16f` + `.json`
(乗算済み線形 half float、余白・枠込み)へ置く。本番の engine は焼けたコマから cache の絵で層を組み、素材の復号も
効果の列も走らない(`engine/frozen.rs`、`frozen_layer`)。場・面の hook と配置・不透明度・blend・マット・時間は生きたまま。
凍った層の中(効果の欄・効果の列・マスク・形・文字・素材)は名指しで断り、位置・不透明度・重ね順・時間は通る
(`check_not_frozen_inside`、`property_is_inside`)。Unfreeze は cache の dir を消す。
窓: Timeline の右クリック(Freeze / Unfreeze)、Inspector の ❄ switch(全部の層)、凍った層の効果は灰色で触れず 1 行の注意、
下の行に「Freezing *Title* 34/120」。審判 `freeze_keeps_the_picture`(凍っても絵は同じ・再起動後も disk から同じ・拒み・Unfreeze で戻る)、
`freeze_op`(口)。
未実装: 立体の Freeze(頂点・点の列)、Flatten(Bounce to Media)、未保存の書類の cache は temp(保存しても引っ越さない — Freeze し直す)。
