# FrameDesc／plugin-facing共有型lineageの価値回収（Unit 3C、2026-07-23）

状態: **縮小採用**（M1仕様28 blobの処分、意味契約と歴史的signatureの分離、現行gapの再発見）

対象: `docs/specs/M1-vertical-slice.md`のcutoff全28版。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[M1仕様](../specs/M1-vertical-slice.md)、[凍結ゲート宣言](2026-07-10-freeze-gate-declaration.md)、[plugin作者向け規約](../plugin-authoring.md)

## 1. 結論

単一path / 28 blobを、初版全文、`git log --all`で到達する全31変更commitの親子diff、分岐版、現行コード事実で処分した。28版すべてで`FrameDesc`の6項目は同形だったが、その周囲のplugin traitはM1後に実際の実装へ合わせて拡張された。

```text
width / height / stride / format / color_space / premultiplied
  → raw YUVと色タグをFrameDescへ接続
  → 内部premultiplied alphaを正規形として実証
  → TextureRef = borrowed wgpu texture + FrameDesc
  → static plugin dispatchをコード実証して凍結ゲート宣言
  → PipelineCache / RenderCtx / typed Result / LayerSourceを追加的に解凍
  → first-party外部crateが公開façadeだけで利用
```

現行へ維持する判断は五つである。

1. `FrameDesc`は画素の意味を暗黙にしない共有記述子である。解像度、stride、pixel format、color space、premultiplied状態の6意味は生存する。
2. Render系pluginの画素境界はCPU bufferでなくGPU textureである。`TextureRef`は借用textureと`FrameDesc`を対で渡し、pluginがtexture寸法や色意味を別経路から推測しないための境界である。
3. 凍結対象は共有する意味と純関数/GPU境界であり、M1文書に残る当時のRust skeletonそのものではない。現行signatureには`PipelineCache`、`RenderCtx`、`Result<_, PluginError>`、`LayerSourcePlugin`等がある。
4. `FrameDesc`を作品Documentの万能画像schema、Vism package ABI、WASM/native wire formatへ昇格しない。これは現行Rust runtimeの共有型で、将来loaderの永続・跨process契約は別審判である。
5. 現行実装には、公開入力でpanic/wrapし得るconstructor、検証を迂回できるderive Deserialize、文字列化された`validate` errorという未解消gapがある。意味を変えず、独立した`GAP-17`で型付き構築・deserialize・負例を閉じる。

## 2. 28版から生き残った意味

### 2.1 六つの明示項目

初版`5b61b83e`からcutoff最終版まで、次の形は28版すべてで同一だった。

| 項目 | 生存する意味 | 誤って足さない意味 |
|---|---|---|
| `width` / `height` | frameの画素寸法 | 正準空間、Stage表示寸法、DPI、window座標ではない |
| `stride` | packed frameの行byte数。YUVは現行実装でplane規約を別に持つ | GPU textureのrow pitch、任意plane layoutをこの1値で表したとはみなさない |
| `format` | packed/YUVのpixel format | vendor API、codec、Vism payload classではない |
| `color_space` | conversion選択に必要な色タグ | pluginごとの自由な色変換許可ではない |
| `premultiplied` | alpha表現の取り違えを防ぐ明示flag | 内部合成でstraight/premulを自由選択するmodeではない |

履歴は当初の「YUV Rec.709 limited」決め打ちから、生YUVを受け、probeしたmatrix/rangeを`FrameDesc.color_space`へ渡してGPU変換する形へ訂正した。さらに内部render targetと合成をpremultiplied alphaへ統一し、normal-over式とgoldenで実証した。この成立理由は現在の色変換一元化とpremultiplied正規形に残す。

### 2.2 TextureRefの役割

M1途中で追加されたplugin skeletonは`TextureRef`を入力／出力に置いた。現行コードでは次の組で実体化している。

```text
TextureRef<'a>
  texture: &'a wgpu::Texture
  desc: FrameDesc
```

`FrameDesc`だけではGPU resourceを所有せず、`wgpu::Texture`だけではstride、色、alpha等の作品側意味をすべて表せない。対で渡すことがVRAM常駐と明示意味を同時に守る。pluginは`TextureRef.desc`を正とし、texture label、format名、解像度、Host内部IDから不足意味を推測しない。

## 3. M1 skeletonと現行APIを分ける

M1文書のplugin例は2026-07-08に追加された入場用skeletonであり、その時点では次が無かった。

- Host所有`PipelineCache`
- `Quality`と時間予約口を畳む`RenderCtx`
- `Result<(), PluginError>`
- `LayerSourcePlugin`
- Contract Catalog、Executor Registry、first-party composition root

後続の凍結解除とVism A0〜A3がこれらを追加・実証した。従って現在の正確な作者APIは`crates/motolii-plugin/src/lib.rs`と[plugin作者向け規約](../plugin-authoring.md)で読む。M1のコードblockは歴史的milestoneの意味基線であり、copy可能な最新signature、外部ABI、配布契約ではない。

同じ理由で、初期`RenderNode`がtextureを返すskeletonと、現行Host executor、`RenderStep`、session-owned targetを同一APIとして復活させない。「previewとexportが同じ評価関数を通る」「plugin出力は明示入力の関数」という意味だけを維持する。

## 4. 「凍結」の範囲

2026-07-10の凍結ゲートは外部semver 1.0ではなく、並列実装者向けの内部契約だった。`FrameDesc`について凍結したのは六つの意味と色/premulの取り扱いであり、次を永久固定した宣言ではない。

- `packed` / `yuv` / `try_*`等のconstructor名とerror形
- public field、derive Serialize/Deserialize、`validate() -> Result<(), String>`
- `PixelFormat` / `ColorSpace`の将来variant閉集合
- Rust layout、C ABI、WASM memory layout、Vism wire format
- `motolii-plugin` façadeの再export閉集合

plugin traitはM2E-7で`RenderCtx`を追加する正規の解凍を既に経験している。共有型も意味を保ったままconstructor、validation、ownershipを安全化できる。変更時は現行fixture、旧serde面、plugin façade、preview/export一致を審判し、M1 skeletonへ形を戻さない。

## 5. 現行コードで再発見したgap

この単位ではコードを変更しない。次の事実を`GAP-17`へ登録する。

| 現行面 | 事実 | 危険 |
|---|---|---|
| `FrameDesc::packed` / `yuv` | `try_*().expect(...)`を呼ぶpublic convenience API | 外部入力由来のformatや奇数YUV寸法でpanicし得る |
| `FrameDesc::try_packed` | `width * bytes_per_pixel`を`u32`のunchecked演算でstrideへ入れる | 巨大寸法でdebug panicまたはrelease wrap。D1k側が事前`checked_mul`して回避しているだけ |
| `FrameDesc` deserialize | public fields + derive Deserialize | zero寸法、format/stride不整合、奇数4:2:0等をconstructor外から作れる |
| `validate` | `Result<(), String>` | 呼び出し側が失敗理由を型で分岐できず、errorを文字列へ潰す |
| plugin façade | `FrameDesc`は再export済み | 上記はprivate helperでなく作者向け公開面にも到達する |

修正は「6項目を消す」「opaque handleへ置換する」ことではない。型付き`FrameDescError`へ不変条件を閉じ、overflow、zero、packed/YUV mismatch、stride、serde bypassの負例を作り、既存の正当なdescriptorとpixel goldenを不変にする。public fieldやserde形の変更が必要なら、利用箇所と互換面を調査してから解凍手続きを通す。

## 6. 復活させない旧具体とSTOP線

- M1 code blockの`FilterPlugin` / `CompositePlugin` signatureを現在の正確なAPIとしてcopyしない。
- `FrameDesc`の6 fieldが長く不変だった事実だけでRust ABI、serde永続形式、Vism wire formatを宣言しない。
- `PixelFormat`を`wgpu::TextureFormat`の完全aliasとみなさない。YUVはCPU decode境界を含み、GPU upload時に変換される。
- `stride`一つから任意multi-plane layout、GPU row pitch、zero-copy import契約を発明しない。
- `premultiplied` flagをpluginごとの合成modeにせず、内部premultiplied正規形と境界変換を維持する。
- pluginがtexture metadata、cache residency、label、opaque IDから`FrameDesc`の不足意味を推測しない。
- panic回避のために不正descriptorを黙ってclamp、wrap、default化しない。型付き拒否にする。
- GAP修正をplugin trait再設計、Document schema、Vism loaderと一括実装しない。

## 7. 固定歴史出典

初版`5b61b83e`を全文で読み、cutoff 28 blobが`git log --all -- docs/specs/M1-vertical-slice.md`の全31変更commitへ対応することを照合した。全親子diffから、基礎実装、effect stack、Draft/Final、raw YUVと色tag、正準座標、premultiplied alpha、plugin skeleton、M1完遂、二度のrebrand、凍結ゲート、実装ガード、UI toolkit注記までを確認した。

機械照合では28版すべての`FrameDesc` field blockがbyte同形で、初期7版はplugin trait追加前、後続21版は同じ初期plugin skeletonを保持していた。cutoff後の現行版だけがresource runtimeの状態訂正を受けている。28 blobの完全SHAは`03e-frame-desc-shared-types.tsv`を正本とし、これらは本書でDISPOSITIONEDとする。

M2 Document内の`FrameDesc`参照、M4 cache key、M5 camera/depth、color/exportの横断lineageは、それぞれUnit 4/5/8で別に処分する。
