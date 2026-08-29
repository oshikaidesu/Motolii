# vgpu(vercel-labs)と Vism 構想 — 同じ形の3つ目が出た

- 日付: 2026-08-29(利用者が vgpu.sh を持ち込み、その場で調べた)
- 位置づけ: **観察**。Vism の設計判断ではない。事実と対応関係だけを置く
- 発端: 利用者「点群データと svg データが同時に同居するグラフィックス作品を作りたい。
  こういうものが web だけに閉じてしまうのは勿体無い、持って来れませんか」

## vgpu が何であるか(実測)

- TypeScript の WebGPU ライブラリ、**MIT**、`vercel-labs/vgpu`。npm 配布
- ブラウザ canvas / headless Node(Dawn 裏打ち)/ serverless を1つの API で通す
- API: `init()` / `effect(gpu, shader, options)` / `surface` / `draw` / `frame` / `target` /
  `clock` / `frameLoop`
- **uniform は WGSL の変数名で addressing**される(README: "effect compiles the shader into a
  fullscreen effect whose uniforms are addressed by their WGSL names through `set()`")
- **時刻は組み込みではなくホストが毎フレーム入れる**: `wave.set({ time: time.time })`
- `npx vgpu docs` / `examples` / `check`、`agents.md` / `llms.txt` を公開 —
  **書き手がエージェントである前提で道具が設計されている**
- `import { hash2 } from "@vgpu/wgsl-std/hash"` を使う。**WGSL に import は無い**ので
  vgpu 側に前処理がある。素の WGSL とは限らない

## 同じ形の3つ目

**シェーダ + 宣言された型付き入力**という形は、既に Motolii の中にも外にもある:

| | 宣言の場所 | 渡し方 | 時刻 | 現在地 |
|---|---|---|---|---|
| ISF | `.fs` の `INPUTS` JSON | name | ホスト | `motolii/engine/motolii-compositor/src/effects/isf/`。`bloom.fs` が1本通済 |
| vgpu | WGSL の uniform 変数名 | `set({name})` | ホストが毎フレーム | 外部・MIT・未検証 |
| Vism | 型付き input を宣言 | 型付き link | ホスト(keyframe 評価) | 設計のみ(`docs/vism-package-concept.md`) |

**駆動側(マウス / キーフレーム / DataTrack)はホストの都合であって、単位の側の違いではない**
(利用者の言葉: 「単なる拡張という、いや、拡張でもないかもしれません、ただの言い換えで
済む可能性が十二分にあります」)。Motolii はこの言い換えを `ParamDriver` / `DataTrack` /
型付き link として `concept.md` に既に持っている。

つまり **vgpu は Vism 構想が成立することの外部証拠**であって、追い越しではない。
vgpu に無いのは時刻・keyframe・配布単位・ホスト契約 — Motolii が足そうとしている所。

## naga が両方の入口になる

`naga` は `motolii/engine/motolii-compositor/Cargo.toml:15` で既に直接依存
(`glsl-in`, `wgsl-out`)。**`wgsl-in` を足せば WGSL も同じ解析器から入る。**

停止線(`vism-package-concept.md` §11)1番「複数の実 plugin で公開境界をコード実証する」
に対して、ISF(GLSL・人が書いた資産)と vgpu 系(WGSL・エージェントが書ける)の2系統を
同じ `EffectDescriptor` へ着地させれば、**境界が ISF に癒着していないことを実証できる**。

## 停止線9本の軸は2本(利用者裁定)

利用者「停止線自体は、具体性がない時にふんわり決めていた。妥当性がなければ実装時に
踏み倒してよい。なにかしら軸があるはずです」。9本を読み直すと守っていたものは2つ:

- **軸A 境界の実在性**(停止線1・2・3・7・8) — 外から来た本物が、内部の特権なしに、
  同じ口で動く。first-party だけ通る抜け道がない。UI/Document への隠れ依存がない
- **軸B 作品の持続性**(停止線4・5) — Vism が欠けても・更新されても・知らない payload が
  来ても作品が壊れない

残り(6=配布形式の比較、9=レビュー手続き)は物が動いてから決めればよい。
**捨てエフェクトは軸Aの不合格判定そのもの** — 口を証明するためだけに存在するものは、
まだ本物を運んでいない。

## 現在地の実測 — 境界はまだ実在していない

`motolii/engine/motolii-engine/src/translate.rs`:

- `KNOWN_EFFECTS` が `&'static [EffectDescriptor]` の**コンパイル時定数**。
  外から来た物が載る余地が構造的に無い
- **ISF の param が手書き定数**(`ISF_BLOOM_PARAMS`)。コメント自身が「手書き(timebox の
  fallback)、front カタログを manifest から都度生成する所まではやらず」と書いている。
  **マニフェストが在るのに読んでいない**
- `translate_glow_params` 等、**plugin ごとの手書き分岐**

## そして描画の居場所が憲法の外にある

**これが最大の発見。** エフェクトの描画は `re_renderer` の中ではなく隣に建っている:

- `motolii/engine/motolii-compositor/src/effects/isf/mod.rs:436` `pipeline: wgpu::RenderPipeline`、
  `:535` `device.create_render_pipeline`、`:669` `encoder.begin_render_pass`
- `effects/glow.rs:43-44` も自前 `wgpu::RenderPipeline` を複数持つ

一方 `re_renderer` は拡張点を持っている(pin 済み checkout
`~/.cargo/git/checkouts/rerun-bdb1f1ac6277bf7e/7cca401/crates/viewer/re_renderer/src/`):

- `renderer/mod.rs:165` `pub trait Renderer` — `create_renderer(ctx: &RenderContext)` と
  `draw(&self, render_pipelines: &GpuRenderPipelinePoolAccessor, phase: DrawPhase,
  pass: &mut wgpu::RenderPass, ...)`。**パイプラインはプールから貰い、パスは渡される**

決定索引に既にある「`wait_indefinitely()` 3箇所・FFmpeg 同期IO 2箇所・**独自テクスチャ
プール1箇所**が export 用コードのライブ経路への誤流用」と**同型の4つ目**。

**マニフェスト駆動化を先にやると、憲法違反の経路を綺麗にするだけになる。**
居場所を直すのが先。調査中(`re_renderer::Renderer` を外部クレートが実装できるか、
実行時 WGSL 文字列からパイプラインを作れるか、中間 texture の器が在るか)。
