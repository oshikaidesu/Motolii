# Stage ゼロコピー — Makepad fork の切り方

作成日: 2026-08-26

状態: **決定**(利用者裁定「見通しの良い責任境界でゼロコピーを実装する。fork の切り方を後任が確認できる文書を残す」)

対象: `oshikaidesu/makepad`(r7 pin `81946efe3379556acd32d1a9479a7ebb1035dcad`)。
Rerun fork 台帳([2026-08-18-rerun-fork-seam-ledger.md](2026-08-18-rerun-fork-seam-ledger.md))と同型。
「上流を追いかけたくなったとき、何を再適用すればよいか」を1枚にする。

関連: 裁定251 / 255 / 256。`next/probes/r7-makepad-panel/src/stage_surface.rs`。
`motolii-compositor` の presentable target 検査。

## 0. なぜ fork するか

Makepad は wgpu ではない。UI は Metal / D3D11 / OpenGL。
一次: [Architecture](https://makepad.rs/guide/start/makepad-framework-architecture) の MPSL、
Rik Arends の「wgpu は薄い実行器をもう1個足すだけ。designtool が先」
([HackMD 転記](https://hackmd.io/@dspfac/r1LdPpDJ3)、[Issue #86](https://github.com/makepad/makepad/issues/86))。

Motolii が待つ物ではない。製品経路は共有面(裁定255)。
iced の同一 `wgpu::Device`(裁定171)は凍結ホストの参考であり、製品経路にしない。

fork の代価は「format を Host が指定できる共有面」1口だけ。
Studio stdin / Servo 埋め込みの既存 `SharedBGRAu8` は触らない。

## 1. 責任の3室(この切り方以外は採らない)

| 室 | 所有 | 知ってよい | 知ってはいけない |
|---|---|---|---|
| Host(engine / compositor) | 絵の意味、1枚、format=`Rgba8UnormSrgb`、寿命はリサイズ時だけ | 渡された `RENDER_ATTACHMENT` | Makepad、`Image`、IOSurface 型 |
| 窓の葉(Makepad fork) | その仕様の共有面を確保し、同じ handle を表示 | size / format / OS handle | Document、re_renderer、VISM |
| r7 継ぎ目 | サイズ変化時だけ結び、同じ handle を渡す | OS handle の import | 合成の中身、effect |

VISM / effect はこの表に入らない。layer と Host 一時面だけを見る。
共有面を作者契約に出すのは Host の GPU 境界破り。

## 2. 通常経路(禁止を先に)

```
Host が size+format を決める
  → Makepad が共有面を1枚作る(サイズ変化時のみ)
  → wgpu が同じ面を RENDER_ATTACHMENT として import
  → compositor がその面へ直接書く
  → Image.set_texture(同じ handle)
```

通常経路で認めない(裁定251):

- CPU readback
- CPU 再アップロード
- 毎フレーム Texture 再生成
- 最終 Texture から共有面への GPU blit

fallback だけ: screenshot / export / 非対応環境。
プレイヘッドを Stage から切り離さない。

## 3. Makepad fork に足す物(追加のみ)

現行 pin が既に持つ物:

| API | 場所 | 使う / 触らない |
|---|---|---|
| `TextureFormat::SharedBGRAu8` | `platform/src/texture.rs:221` | Studio / Servo 用。**製品 Stage の format にはしない** |
| `Cx::create_iosurface_render_texture` | `platform/src/os/apple/metal.rs:1366` | BGRA8 linear 固定。製品 Stage の生成口にはしない |
| `CxTexture::update_from_shared_handle` | 同 `2542` | ID lookup。Metal pixel format が `BGRA8Unorm` 固定 |
| IOSurface pixel format `'BGRA'` | 同 `2450` | 製品 format と不一致。ここを分岐する |

**足す seam(追加。既存 variant / 関数は書き換えない):**

1. `TextureFormat::SharedPresentable { width, height, id, initial, pixel }`
   - `pixel` は Host が渡す閉集合。製品 Stage は `Rgba8Srgb` だけ
   - `SharedBGRAu8` は残す。Studio stdin を壊さない
2. `Cx::create_presentable_texture(width, height, pixel) -> (Texture, SharedOsHandle)`
   - mac: IOSurface。`IOSurfacePixelFormat` と `MTLPixelFormat` を `pixel` から決める
   - win: DXGI shared handle。既存 `update_shared_texture` に format 引数を足すのではなく、新関数
   - linux: dma-buf。既存 GL `update_shared_texture` と同型の新関数
3. `SharedOsHandle` は整数 ID / HANDLE / fd だけ。wgpu 型を Makepad に入れない
4. `TextureFormat::vec_width_height()` へ `SharedPresentable` の腕を1本
   (`platform/src/texture.rs`、rev `447dcc3c`)
   - 共有面の**寸法を答えるのは葉の責任**。`as_alloc` は同じ width/height を既に持つ
   - これが無いと `Image` が 0×0 の quad を描き、3室とも "ok" のまま画だけが出ない
     (2026-08-26 に実測。Stage 黒画面の第2の根因)
   - OS 非依存ファイルなので mac / win / linux で1度に効く

**採らなかった切り方:**

| 案 | 却下理由 |
|---|---|
| 既存 `SharedBGRAu8` に compositor を合わせる | Host の色境界が front 都合になる。裁定251の既知境界そのもの |
| compositor 最終画を共有面へ blit | 裁定251禁止 |
| Makepad を wgpu バックエンドにする | Rik の分類でも「もう1バックエンド」。同一 Device 化ではない。待たない |
| 窓の横に wgpu を置くだけ | すでに r7 がそうしている。Device が二重のまま |

## 4. 同時に要る rerun fork seam(Makepad ではない)

`ViewBuilder::new` は常に自前 `main_target` を `textures.alloc` する
(`re_renderer/src/view_builder.rs`、pin `7cca401`)。
Motolii compositor は MSAA Off なので resolved = その1枚。

共有面へ直接書くには、**追加**で次が要る:

```text
ViewBuilder::new_with_external_resolved(ctx, config, id, texture)
```

- `texture.format == MAIN_TARGET_COLOR_FORMAT`(`Rgba8UnormSrgb`)
- usage に `RENDER_ATTACHMENT`
- size が `config.resolution_in_pixel`
- 既存 `new` は無改造

これは [rerun fork 台帳](2026-08-18-rerun-fork-seam-ledger.md) の
`ViewBuilder::main_target()`(裁定161)と同型の**追加1本**。
上流 file への削除をしない。

`motolii-compositor::Compositor::render_into` はこの口が着くまで
presentable 検査だけを公開する。blit で先に通さない。

## 4.5 どの室で止まったかを1行で読む

r7 は present 1回ごとに室を名指す(`stage_surface.rs` の `StageRoom` / `StageVerdict`)。

```
STAGE room=leaf owner=makepad fork reason=the shared surface reports no size (drawn 0x0)
```

`Shown` は「書けた」ではなく「**出た**」を意味する — `check_shown` が
`is_zero_copy` と「表示側が答えた寸法 == 共有面の寸法」を見る。
2026-08-26 の黒画面は3室とも成功を返していたので、この検査が無い限り
全コードを読む以外に室を絞る方法が無かった。win / linux の import を足すときも、
先に読むのはこの1行であってコードではない。

Stage 上の文言は ASCII に限る — Makepad の既定フォントに CJK グリフが無く、
日本語は `.notdef` で静かに潰れる(同日実測)。

## 5. OS 対応表

| OS | 共有面 | 受入 |
|---|---|---|
| mac | IOSurface | 必須 |
| win | DXGI shared handle | 必須 |
| linux | dma-buf | 必須 |
| wasm / WebGL | なし | 契約外(裁定255) |

## 6. 上流 rebase で最初に見る順

1. `platform/src/texture.rs` の `TextureFormat` match(網羅性が壊れる)
2. `platform/src/os/apple/metal.rs` の IOSurface 生成(pixel format 分岐)
3. `platform/src/os/windows/d3d11.rs` の shared texture
4. `platform/src/os/linux/x11/opengl_x11.rs` の shared texture
5. rerun 側は `view_builder.rs` の `new` の隣

既存 `SharedBGRAu8` 経路(Studio stdin)に手を入れた差分は、rebase 前に読み直す。
製品 Stage の差分は新 variant / 新関数だけなら conflict しにくい。

## 7. 検収

1. 通常経路で `cpu_readback_calls == 0`、`cpu_upload_calls == 0`、サイズ不変時 `texture_creations == 0`
2. screenshot / export だけ CPU 経路が非ゼロ
3. スクラブ中にプレイヘッドが滑らか(第二表示経路を作らない)
4. VISM / Glow が共有面を知らない(既存 layer オフスクリーンのまま)

計測は `--profile preview`。debug バイナリで時間を語らない。

## 8. Motolii に置いた物 / fork に残した物(2026-08-26)

Motolii 側は契約と検査だけ。GPU 書き込みは fork 2本が着くまで開かない。

| 置き場 | 何を置いたか | まだ無い物 |
|---|---|---|
| `docs/reviews/2026-08-26-stage-zero-copy-makepad-fork-seam.md` | この切り方の正本 | — |
| 裁定256 | 3室と blit 禁止を1行で固定 | — |
| `next/probes/r7-makepad-panel/src/stage_surface.rs` | Host 契約。`StagePresent::Shared` が通常経路。`FallbackCpu` は共有面が使えない時だけ | — |
| r7 `main.rs` / `stage_import.rs` | サイズ変化時だけ共有面 → import → `render_into` → `Image.set_texture`。失敗時だけ FallbackCpu | r7 は独立 workspace なので next の `[patch]` は効かない。Shared 未接続。窓は Fallback |
| `motolii-compositor::Compositor::render_into` | `ViewBuilder::new_with_external_resolved` で渡された texture へ直接書く | cargo git checkout へ rerun 差分を載せた。pin は未 bump |
| Makepad fork `/tmp/motolii-forks/makepad` | `SharedPresentable` + `Cx::create_presentable_texture`。`SharedBGRAu8` 無改造 | 未 push |
| rerun fork `/tmp/motolii-forks/rerun` | `ViewBuilder::new_with_external_resolved`。`new` / `main_target()` 無改造 | 未 push |

後任が rebase する順は §6。製品 Stage の差分は新 variant / 新関数だけ。
