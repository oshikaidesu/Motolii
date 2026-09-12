# Shadertoy の取り込み — 貼れば棚に出る

作成日: 2026-09-12

状態: **Shadertoy(`mainImage`)は実装済み。層の絵を別の時刻で読む口(`TIME_OFFSET`)は実装済み(§8)。Buffer 複数枚・音・`iChannelResolution` / `iChannelTime` は未対応。貼る窓(editor)は作らない(各自の editor で書く)。**

関連正本: [場(Field)の取説](vism-field-model.md)、[Vism コンセプト](vism-package-concept.md)、[プラグイン作者向け規約](plugin-authoring.md)

## 1. 一文で

> **変換器は書いていない。** GLSL → WGSL は wgpu 本体の [naga](https://github.com/gfx-rs/wgpu/tree/trunk/naga)（`glsl-in` / `wgsl-out`）が決定的に行う。Motolii が足したのは**方言の前口上**だけ — Shadertoy の名前を ISF の名前へ結び直す 100 行。

## 2. 使い方

`vism/` に Shadertoy の file を `.frag`(または `.glsl` / `.fs`)で置く。manifest は要らない。

```glsl
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    fragColor = texture(iChannel0, uv) * vec4(1.0, 0.8, 1.2, 1.0);
}
```

file 名が効果の名前になり、ID は `import.<file 名>`。棚には同梱の効果と並ぶ。

## 3. 名前の結び直し

| Shadertoy | Motolii での姿 |
|---|---|
| `mainImage(out vec4, in vec2)` | ISF の `main()` から呼ばれる |
| `fragCoord` | 画素の座標。**上下を裏返して**渡す(§5) |
| `iResolution` | `vec3(RENDERSIZE, 1.0)` |
| `iTime` | `SUBTYPE: TIME` の欄。host が時刻を流す |
| `iChannel0..3` | **使われている物だけ** image 入力の欄になる |
| `iMouse` | 使われていれば point2D の欄 |
| `iFrame` / `iTimeDelta` / `iFrameRate` / `iDate` / `iSampleRate` | `iTime` と 60fps から組む定数 |

写せない名前(`iChannelResolution`・`iChannelTime`)は、黙って壊れずに**名前を挙げて断る**。manifest の無い素の GLSL(`void main()` だけ)も、理由を添えて断る。

実装は [`effects/isf/shadertoy.rs`](../motolii/crates/motolii-render/src/compositor/effects/isf/shadertoy.rs)、入口は [`effects/catalog.rs`](../motolii/crates/motolii-render/src/compositor/effects/catalog.rs) の `prepare`。

## 4. 乗る先

取り込んだ効果は `STAGE: pass` になる。[場の取説 §8](vism-field-model.md) の通り、**Pass は素材を選ばない** — 板は層の絵へ焼かれ、網・点群は描いた後の窓で効く。つまり貼った Shadertoy は 3D にも乗る。

## 5. 上下の向き(落とし穴)

Shadertoy も ISF も**座標は下端が 0**(GL の作法)、wgpu の texture は**上端が 0**。座標をそのまま
サンプリングに使うと、上下逆の位置を読む。**実写に当てて初めて分かる**(文字が裏返る。対称な
効果では絶対に気づけない)。

直し方は「座標を裏返す」ではなく「**読む時に裏返す**」。座標は各方言の作法のまま渡すので、
手続き的な絵の向きも Shadertoy と揃う。

- Shadertoy: `#define texture(smp, coord) texture(smp, vec2((coord).x, 1.0 - (coord).y))`
  (macro は自分自身へ展開し直されないので、中の `texture` は組み込みのまま)
- ISF: `IMG_THIS_PIXEL` / `IMG_NORM_PIXEL` が裏返す

正しさの証明は**恒等**で取った — `fragColor = texture(iChannel0, uv)` の結果が元の絵と
**完全一致**(PSNR ∞)。見張りは `shadertoy.rs` と `isf/mod.rs` の
`the_picture_is_read_right_side_up`。

## 6. 表現の天井(2026-09-12 実測)

実写に当てて、書けるものと書けないものの線を引いた。

| 書きたい物 | 状態 |
|---|---|
| 手続きの絵(`iTime` だけ) | **書ける** |
| 画像フィルタ(`iChannel0`) | **書ける** |
| 近傍を舐める(Sobel・ぼかし) | **書ける** |
| **複数パス**(抽出 → 横 → 縦 → 合成) | **書ける**(ISF の `PASSES`。`PASSINDEX` と中間 buffer が shader から見える) |
| 中間 buffer の寸法を変える(`$WIDTH/2`) | **書ける** |
| **フレーム跨ぎの持ち越し**(`PERSISTENT`・Shadertoy の Buffer 帰還) | **この口からは書けない**(下記) |

### 持ち越しの扱いは 2026-07-10 に決着している

恒久に禁じられているのは**効果が自分で前フレームを覚えること**(`StatefulFilter`、再生ヘッド依存の
隠しバッファ)だけである。理由は能力ではなく**追跡できないこと** — 純関数契約・フレーム並列・
スクラブが壊れ、同じ時刻を 2 回描くと違う絵になる。ISF の `PERSISTENT` を断るのはこれに当たる。

時間を使う表現そのものは禁じられていない。解は「プラグインの賢さではなく、**ホストが渡す
時間参照**」で、口の形まで予約済み — [`plugin-resources.md` §6](plugin-resources.md)(F-11):

| 形 | 何ができる | 状態 |
|---|---|---|
| **lookbehind**(非再帰。`exclude` で自己参照を切る) | 残像、時間差、フレーム間の比較 | **層の絵の別時刻は実装済み(§8、2026-09-12)**。合体後(Group / CompRoot)の別時刻は予約のまま |
| **フィードバック**(再帰。クリップ先頭を初期条件とする漸化式 + チェックポイント/リプレイ) | 軌跡、蓄積、反応拡散 | **口の予約のみ。未実装** |

後者は「スクラブすると変わる」TD / AviUtl 型ではなく、**コーデックの GOP と同型**(チェックポイント
から再生)に定義することで決定性を保つ、と §6-3 が定めている。先人は Nuke の別時刻入力、
反面教師は AE Echo(キャッシュ規律なしの素朴な再評価)。

datamosh はさらに別トラックで、codec 領域の台帳が
[decision-index.md](decision-index.md)(`M5-DATAMOSH-P0` = `DONE / PRIVATE PROBE`・`BUILD FORBIDDEN`)にある。

つまり**この取り込み口の天井**は「時間を持ち越せない」ではなく、「**ホストの時間参照のうち、
層の絵の別時刻だけが繋がっていて、合体後の別時刻と再帰はまだ**」である。

## 7. まだ無い物

- **合体後(Group / CompRoot)の別時刻**(`CompLookbehind` の本来の対象)と、**再帰のフィードバック**。
  §6 の通り設計は 2026-07-10 に済んでいる。層の絵の別時刻(§8)は、その入口の最初の 1 本。
- **貼る窓**は作らない。各自の editor で書き、file を置く。
- **Shadertoy の Buffer A..D をそのまま貼る**(1 file に複数 tab を書く取り決め)。ISF の `PASSES`
  に写せるので、器はもう在る。
- **音**(`iChannel` に音を入れる型)。
- naga の GLSL frontend が読めない書き方。通らなければ理由が出る。

## 8. 別の時刻の絵を読む — `TIME_OFFSET`

image の欄に `TIME_OFFSET` を書くと、**ホストがその時刻の層の絵を作って渡す**。効果は何も覚えない。
値は**数値**(秒。負が過去。作者が固定)か、**float 欄の名前**(その欄が普段の仕組みで Inspector に出て、
利用者が回す)。同梱の実例は [`vism/time_difference.fs`](../motolii/crates/motolii-render/vism/time_difference.fs)。

```glsl
/*{ "ID": "motolii.time_difference", "STAGE": "pass",
    "INPUTS": [
      { "NAME": "inputImage", "TYPE": "image" },
      { "NAME": "past",   "TYPE": "image", "TIME_OFFSET": "offset" },
      { "NAME": "offset", "TYPE": "float", "DEFAULT": -0.2, "MIN": -5.0, "MAX": 5.0 } ] }*/
void main() {
    gl_FragColor = abs(IMG_THIS_PIXEL(inputImage) - IMG_THIS_PIXEL(past));
}
```

### 作法(ホスト側)

- **「時刻 t′ の層の姿」を作るのは Document の resolve 1 箇所だけ。** ホストは `view.resolved_layers(t′)` で
  層を引き直し、その姿で絵を作る(`engine/render.rs` の `sources_at_other_times`)。`source_time` だけを
  手でずらすと、mask やキーフレームが t のままの継ぎ接ぎになる — やらない。
- 別の時刻の読みは**別の流れ**(層の番号を変えて復号器の流れを分ける)。同じ流れで読むと、今の絵まで
  巻き添えで上書きされる。
- 読んだ絵は**写しを取る**。素材の texture は時刻ごとに同じ 1 枚へ上書きされるので、写した物だけが
  「あの時刻の絵」でいられる。写しの命令は**即 submit しない** — 復号したコマの転送は frame 共通の
  encoder に積まれ、流れるのは `before_submit` の中。先に打つと、届いていない texture を写す
  (冷えていれば零、暖まっていれば前のコマ。**同じ時刻の絵が辿り方で変わる** — 2026-09-12 にこれで
  1 度落ちた)。
- 届かなかった時に**今の絵で代用しない**。理由を挙げて層の失敗にする。

### 審判

`engine/render.rs` の `time_reference_is_deterministic` — いきなり飛んだ時と、頭から辿った時で、同じ時刻の
絵が**完全一致**すること(実 GPU、毎コマ変わる素材)。素の動画で同じ事を先に確かめる兄弟の test が並ぶ。

### 限界

- 読めるのは**自分の層の絵**だけ。合体後の別時刻(下の層ごと)は予約のまま。
- 名指した欄が無い(または float でない)場合は、黙って 0 にせず名前を挙げて断る。
- 費用は、ずれ 1 つにつき復号 1 回 + 写し 1 枚。cache はまだ効かない。
- 速度を変えた層(time stretch)は、素材側のずれが comp の秒とは一致しない。
