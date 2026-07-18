# D1k-S CQ-5 解凍記録: runtime planar `CompCamera` と必須 camera-bearing render signature (2026-07-18)

ステータス: **【凍結】** — D1k 実装前の文書-only 契約。本記録が D1k の正本。

正本チェーン: [planar v1 camera 決定](2026-07-16-m2-comp-camera-decision.md) → 本解凍記録 → D1k 実装 → D3f Document camera 接続。

レーン: `CAM-G0 (DONE)` → `D1j (DONE)` → **`D1k-S (本記録)`** → `D1k (WAIT)` → `D3f (WAIT)`。

## 1. 解凍三要素

### 1.1 変更理由

- 現行 `motolii_core::CompCamera` は公開 field の `position` / `target` / `fov_y_degrees` / `roll_degrees`、度単位、Perspective 意味、`Serialize` / `Deserialize`、`DEFAULT` / `Default`、`validate() -> Result<(), String>` に依存している。これは [2026-07-16 決定](2026-07-16-m2-comp-camera-decision.md) の planar orthographic 方程式と矛盾する。
- `RenderGraphInputs` は `#[derive(Default)]` と `impl Default` により camera 無し呼び出しが型上可能であり、`dispatch_plugin` が `CompCamera::DEFAULT` を直書きしている（監査 C-5）。preview/export 同一関数契約の下で camera を呼び出し側が明示できない。
- D1j で `CompCameraDoc::PlanarOrthographic` を永続化したが、runtime API と render entry の必須 signature は未凍結のまま D1k へ進むと実装が旧型を adapter として残す逃げ道になる。

### 1.2 migration / wire 非影響

- **永続 camera の正本は `CompCameraDoc` のみ**。D1k は Document version、wire JSON、`min_reader_version`、D1e migration 経路を変更しない。
- D1k は `CompCameraDoc` を時刻 `t` で評価しない。Document camera 評価と graph 接続は **D3f** のみ。
- D1k 完了まで既存 callsite は §3.3 の **明示 identity camera** を `try_new` で構築する。identity 用の追加 constructor / 定数 / 暗黙 fallback は禁止。

### 1.3 golden 影響

- **CAM-G0 semantic oracle**（`crates/motolii-render/tests/oracles/cam_g0_planar_identity.tsv`）の期待 RGBA bytes は不変。D1k は oracle 更新で通してはならない。
- 既存 semantic oracle・保護ゴールデンの期待値更新は不要。D1k の意味審判は本記録 §8 の新規 fixture で行う。
- `Quality::render_desc` の aspect 保全規則は [2026-07-16 決定](2026-07-16-m2-comp-camera-decision.md) と一致させる。`1920×1080 / 2 = 960×540` は維持し、`16×9 / 2` は入力 desc 不変。

## 2. 旧 API の撤去（D1k で消えるもの）

現行 `motolii_core::CompCamera` から **すべて** 撤去する。互換 shim、adapter、並行 camera 型、公開 raw field、serde bridge は残さない。

| 撤去対象 | 現状 |
|---|---|
| 公開 field | `position`, `target`, `fov_y_degrees`, `roll_degrees` |
| 意味 | 度単位・Perspective 固定 |
| 永続化 | `Serialize` / `Deserialize` |
| 暗黙既定 | `DEFAULT` 定数、`Default` trait |
| 検証 | `validate() -> Result<(), String>`（文字列潰し） |

**禁止（採用契約にしない）**

- `Option<CompCamera>`、`None` = 暗黙 identity、`CompCamera::DEFAULT` 直書き、builder での遅延 fill
- `RenderGraphInputs` の `Default` derive / `impl Default`（camera 必須化と両立しない）
- 旧 `position` / `target` / FOV から新 planar への公開変換 shim
- `CompCameraDoc` 以外の durable camera 型

## 3. 凍結 runtime API（D1k でそのまま実装）

型は既存の `CanonicalPoint`（`motolii_core`）、`PixelPoint`（`motolii_core`）、`FrameDesc`（`motolii_core`）を用いる。raw 配列で world / pixel を代替しない。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompCamera {
    center: CanonicalPoint,
    roll_radians: f64,
    height: f64,
    aspect_num: i64,
    aspect_den: i64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CompCameraError {
    #[error("camera center must be finite, got ({x}, {y})")]
    NonFiniteCenter { x: f64, y: f64 },
    #[error("camera roll must be finite, got {roll_radians}")]
    NonFiniteRoll { roll_radians: f64 },
    #[error("camera height must be finite, got {height}")]
    NonFiniteHeight { height: f64 },
    #[error("camera height must be positive, got {height}")]
    NonPositiveHeight { height: f64 },
    #[error("camera aspect numerator must be positive, got {aspect_num}")]
    NonPositiveAspectNum { aspect_num: i64 },
    #[error("camera aspect denominator must be positive, got {aspect_den}")]
    NonPositiveAspectDen { aspect_den: i64 },
    #[error("world point must be finite, got ({x}, {y})")]
    NonFiniteWorldPoint { x: f64, y: f64 },
    #[error("NDC point must be finite, got ({x}, {y})")]
    NonFiniteNdc { x: f64, y: f64 },
    #[error("pixel point must be finite, got ({x}, {y})")]
    NonFinitePixel { x: f64, y: f64 },
    #[error("frame width must be non-zero")]
    ZeroFrameWidth,
    #[error("frame height must be non-zero")]
    ZeroFrameHeight,
    #[error(
        "frame {width}x{height} does not match camera aspect {aspect_num}/{aspect_den}"
    )]
    AspectMismatch {
        width: u32,
        height: u32,
        aspect_num: i64,
        aspect_den: i64,
    },
}

impl CompCamera {
    pub fn try_new(
        center: CanonicalPoint,
        roll_radians: f64,
        height: f64,
        aspect_num: i64,
        aspect_den: i64,
    ) -> Result<Self, CompCameraError>;

    pub fn center(self) -> CanonicalPoint;
    pub fn roll_radians(self) -> f64;
    pub fn height(self) -> f64;
    pub fn aspect_num(self) -> i64;
    pub fn aspect_den(self) -> i64;

    pub fn world_to_ndc(
        self,
        point: CanonicalPoint,
    ) -> Result<(f64, f64), CompCameraError>;

    pub fn ensure_matches_frame_desc(
        self,
        desc: &FrameDesc,
    ) -> Result<(), CompCameraError>;

    pub fn ndc_to_pixel(
        self,
        ndc_x: f64,
        ndc_y: f64,
        desc: &FrameDesc,
    ) -> Result<PixelPoint, CompCameraError>;

    pub fn world_to_pixel(
        self,
        point: CanonicalPoint,
        desc: &FrameDesc,
    ) -> Result<PixelPoint, CompCameraError>;
}
```

### 3.1 構築と不変条件

- **`try_new` が唯一の constructor**。非有限 camera 値、非正 `height` / aspect を拒否した後、正の aspect を **内部で gcd 既約化**する。「既約済みのみ受け付ける」別口は無い。
- `CompCameraError` は `f64` payload を持つため **`Eq` / `Hash` を derive しない**。
- frame aspect 照合は exact `i128` 乗算: `W * aspect_den == H * aspect_num`。正の `u32 × i64 < 2^95` のため `i128` 溢れは起こらず、**`AspectOverflow` variant は定義しない**。

### 3.2 凍結方程式

```text
a = aspect_num / aspect_den
q = R(-roll_radians) * (point - center)
ndc_x = 2 * q.x / (height * a)
ndc_y = 2 * q.y / height
pixel_x = (ndc_x + 1) * W / 2
pixel_y = (1 - ndc_y) * H / 2
```

- NDC は Y-up。ラスタ pixel は Y-down。
- `world_to_ndc`: 非有限 world 入力を拒否する。計算 NDC が非有限なら `NonFiniteNdc` で typed reject する。
- **`world_to_ndc` は `height * a`（すなわち `height * aspect_num / aspect_den`）を浮動小数点の中間積として形成してはならない**。`ndc_x` は overflow-aware な逐次比評価で求める（推奨順序: `((q.x / height) * 2.0) * aspect_den / aspect_num`、または同等の scaled algorithm）。各段で有限性を確認し、数学的に有限な結果を `NonFiniteNdc` で誤拒否してはならない。
- `ndc_to_pixel`: 非有限 NDC を拒否し、**先に** `ensure_matches_frame_desc` を呼ぶ。計算 `PixelPoint` が非有限なら typed reject。
- `world_to_pixel`: 上記2メソッドの `Result` 合成のみ。別方程式を持たない。

### 3.3 Identity（明示のみ・aspect 出所を分離）

identity 用の追加 constructor / 定数は無い。D3f までの一時 callsite は次の **どちらか一方** で `try_new` する。`try_new` が aspect を gcd 既約化する。

**A. Document / 製品経路（`Composition` を所有）**

```rust
CompCamera::try_new(
    CanonicalPoint::CENTER,
    0.0,
    1.0,
    composition.aspect_num(),
    composition.aspect_den(),
)?
```

**B. レガシー非 Document 経路（M1 / `ProjectV1` / 単体 test / CAM-G0 等、`Composition` を持たない）**

呼び出し側が既に正本とする **一時的な出力または graph descriptor** の `FrameDesc` から aspect を明示する。

```rust
CompCamera::try_new(
    CanonicalPoint::CENTER,
    0.0,
    1.0,
    i64::from(desc.width),
    i64::from(desc.height),
)?
```

- 経路 B は **一時的** のみ。Document へ書き込まず、永続化・wire・migration へ流さない。新しい constructor / helper を追加しない。
- 全既存 callsite が `Composition` を持つとは限らない。経路 A と B の使い分けを D1k 実装で混同しない。
- matching aspect では現行 `ViewportTransform` に帰着する（[CAM-G0](../specs/M2-document-model.md#compcameraレーンm2再締結対象直列) で byte 固定済み）。

### 3.4 D3f までの callsite 規律

- D1k は `CompCameraDoc` を時刻 `t` で評価しない。
- 既存 callsite は §3.3 の明示 identity のみ。Document 評価は D3f。

## 4. 凍結 render API disposition

### 4.1 必須 `camera: CompCamera` field

次の公開型は **すべて** 必須 `camera: CompCamera` を持つ。

| 型 | 所在 |
|---|---|
| `LayerSourceContext` | `motolii_plugin` |
| `RenderGraphInputs<'a>` | `motolii_render` |
| `RenderFrameRequest` | `motolii_render` |
| `BackgroundTextureRequest<'a>` | `motolii_render` |

`RenderGraphInputs` から **`#[derive(Default)]` と `impl Default` の両方を削除**する。camera を `Option`、builder、遅延 fill へ退避する案は採用しない。

### 4.2 公開 graph entry

```rust
pub fn render_graph(
    gpu: &GpuCtx,
    timeline_time: RationalTime,
    graph: &LinearRenderGraph,
    camera: CompCamera,
    quality: Quality,
) -> Result<RenderedFrame, RenderError>;
```

| entry | camera の取得元 | 検証対象の元 `FrameDesc` |
|---|---|---|
| `render_frame` | 必須 `RenderFrameRequest.camera` | `request.desc` |
| `render_frame_with_background_texture` | 必須 `BackgroundTextureRequest.camera` | `request.desc` |
| `render_graph` | 引数 `camera` | `graph.desc` |
| `render_graph_cached` | 必須 `RenderGraphInputs.camera` | `graph.desc` |
| `render_graph_cached_pool_alias_for_test` | 同上（公開テスト別名） | `graph.desc` |

### 4.3 render 境界の camera 検証（全 entry 共通）

**すべての公開 render entry** は、次を **`Quality::render_desc`・GPU リソース作成・plugin dispatch のいずれより前** に行う。

```rust
camera.ensure_matches_frame_desc(&original_desc)?;
```

- `original_desc` は上表の「検証対象の元 `FrameDesc`」。縮小後 desc、LayerSource 内だけ、plugin 付き graph だけ、など **部分経路だけの検証は禁止**。
- 縮小後の内部レンダ desc に対しても、camera は元 desc との aspect 一致をすでに満たしている必要がある（縮小は exact aspect 保全時のみ許可 — §5）。
- `render_graph_cached` 内の graph camera は **変更なく** `LayerSourceContext.camera` へ転送する。
- preview と export は **同一 render 関数**。差は `Quality` のみ（絶対規律 6）。
- render 層は `CompCameraError` を `String` 化せず、次で **transparent** に伝播する。

```rust
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    // ... 既存 variant ...
    #[error(transparent)]
    Camera(#[from] CompCameraError),
}
```

## 5. `Quality::render_desc` 契約（signature 維持）

```rust
pub fn render_desc(self, desc: FrameDesc) -> FrameDesc;
```

スケール `s = resolution_scale.max(1)`。`s == 1` なら入力をそのまま返す。`s > 1` のとき、次を **両方** 満たす場合のみ縮小する。

1. `desc.width % s == 0` かつ `desc.height % s == 0`
2. 候補 `(desc.width / s, desc.height / s)` が入力と **exact rational aspect 一致**（`i128` 乗算）

どちらか失敗なら **入力 descriptor を不変で返す**（epsilon stretch / crop / letterbox 禁止）。

縮小が成立したとき、候補 descriptor は次の **順序固定** で構築する。`render_desc` は panic も wrap もしてはならない。

1. `format` / `color_space` / `premultiplied` は入力から **そのまま写す**
2. 候補 `width = desc.width / s`、`height = desc.height / s`
3. 入力 `format` の `bpp` を取得する。supported packed render format でない（`bpp` が取れない）場合は、縮小せず **入力 descriptor を不変で返す**
4. `candidate_width.checked_mul(bpp)` を計算する。`None`（stride 乗算 overflow）なら、縮小せず **入力 descriptor を不変で返す**
5. 上記が安全な場合のみ、**非 panicking** の `FrameDesc::try_packed` で候補 descriptor を構築する。`try_packed` が `Err` なら、縮小せず **入力 descriptor を不変で返す**

`try_packed` は stride 安全性が証明されるまで呼ばない。`bpp` 取得と `checked_mul` は `try_packed` より前に必ず行う。

必須例:

| 入力 | scale | 出力 |
|---|---|---|
| `1920×1080` | `2` | `960×540`（format / color_space / premultiplied 不変、stride 再計算） |
| `16×9` | `2` | `16×9` 不変 |

camera / frame aspect の一致は render 境界（§4.3）で `ensure_matches_frame_desc` が **元 desc** に対して審判する。`render_desc` は aspect を製造しない。

## 6. D1k 実装タスク境界（本記録の非目標）

- D3f の Document camera 評価
- `CompCameraDoc` → runtime 変換の恒久 helper（D3f まで callsite は §3.3 の明示 identity のみ）
- Spatial / 将来 Perspective variant の serde または runtime 追加
- Composition 以外への aspect 永続化
- Slint / px / DPI の camera 契約への流入
- migration / wire version 変更
- CAM-G0 oracle 変更
- allowlist / lint 抑制 / fixture special-case / 生 JSON scanner / 公開 raw mutation API

## 7. 旧 → 新対応表

| 領域 | 旧（撤去） | 新（D1k） |
|---|---|---|
| 姿勢 | `position[3]`, `target[3]` | `center: CanonicalPoint`（XY planar） |
| 回転 | `roll_degrees` | `roll_radians` |
| 視野 | `fov_y_degrees` (Perspective) | `height: f64`（orthographic 可視高） |
| aspect | 暗黙 / 未接続 | 正既約 `aspect_num` / `aspect_den`（Composition 正本；一時経路は §3.3） |
| 構築 | `DEFAULT` / `Default` | `try_new` のみ |
| 検証 | `validate() -> Result<(), String>` | `CompCameraError` + 各メソッドの `Result` |
| render 入力 | camera 無し / `DEFAULT` 直書き | 全公開 entry で必須 `CompCamera` + §4.3 事前検証 |
| 永続 | 旧 core 型の serde | `CompCameraDoc` のみ（D1j 済み） |

## 8. D1k 完了審判用テスト列挙（実装チケットで実施）

### 8.1 正例

- center / 四隅 / roll の world→NDC→pixel 一致
- Y-up NDC → Y-down pixel flip
- identity camera + matching aspect が CAM-G0 / `ViewportTransform` と一致
- `1920×1080` Draft scale 2 → `960×540`（metadata 保全・stride 再計算）
- `16×9` Draft scale 2 → 入力 `16×9` 不変
- 巨大 `width` で候補 packed stride が `checked_mul` overflow する Draft scale → 入力 descriptor 不変（panic / wrap 禁止）
- **`world_to_ndc` 評価順序回帰（正例）**: `q.x = f64::MAX / 4`、`height = f64::MAX / 2`、aspect `4/1`（`aspect_num=4`, `aspect_den=1`）。`height * aspect` を浮動小数点中間積として形成すると overflow するが、正しい `ndc_x` は有限・非ゼロで **`0.25`**（camera 方程式 fixture の stated numerical tolerance 内）。`NonFiniteNdc` を返してはならない。評価は `((q.x / height) * 2.0) * aspect_den / aspect_num` 等の scaled algorithm を用いる
- preview / export が同一 render 関数（`Quality` のみ差）
- `LayerSourceContext` へ graph camera がそのまま伝播
- 全公開 render entry が `Quality::render_desc` より前に元 `FrameDesc` で `ensure_matches_frame_desc` する

### 8.2 負例 — runtime typed reject

- 非有限 center / roll / height / world / NDC / pixel
- 非正 height / aspect_num / aspect_den
- `FrameDesc` ゼロ次元
- aspect mismatch（`AspectMismatch`）
- 数学的に非有限になる入力の fixture で `NonFiniteNdc` を返す（§8.1 の評価順序正例と混同しない）
- render entry が aspect 不一致 camera で `RenderError::Camera(CompCameraError::AspectMismatch { .. })` を返す（縮小前の元 desc で検証）

### 8.3 負例 — compile-time / API 撤去証跡

次は **実行時の typed reject テストではなく**、D1k PR の compile fallout、read-only API レビュー、通常の `cargo test --workspace` 緑で確認する。生ソース文字列 scanner や `trybuild` 依存の compile-fail test は **要求しない**。

- 旧公開 field / `Serialize` / `Deserialize` / `DEFAULT` / `Default` / `validate() -> String` の残存なし
- `RenderGraphInputs` の `Default` derive / `impl Default` 残存なし
- `Option<CompCamera>` や暗黙 fallback の導入なし
- 旧 position / target / FOV shim の残存なし

### 8.4 不変

- CAM-G0 semantic oracle bytes 不変
- D1k が `CompCameraDoc` を `t` で評価しない（D3f 待ち）

## 9. 参照

- [M2 CompCamera 決定](2026-07-16-m2-comp-camera-decision.md)
- [M2 仕様 CompCamera レーン](../specs/M2-document-model.md#compcameraレーンm2再締結対象直列)
- [統一 Stage / Output Frame 設計](2026-07-14-unified-stage-camera-design.md)（M2 schema/runtime 節は本記録で superseded）
