use std::sync::Arc;

use iced::widget::shader;
use iced::wgpu;

use motolii_core::CompSpec;
use motolii_engine::Engine;

use crate::tokens::Colors;
use crate::{
    metrics, settings_pane, stage, Message, PresenterSource,
    PreviewSnapshot,
};

/// Stage 表示用に RGBA を縮める。**裁定166**: 旧 `stage_handle_rgba` の
/// 置き換え——旧実装は iced の `image::Handle::from_rgba` が同期アップロード
/// できる上限(`iced_wgpu-0.14.0/src/image/cache.rs::upload_raster`の
/// `MAX_SYNC_SIZE = 2MB`)を超えないよう `stage_auto_scale`(sqrt 自動縮小)を
/// 掛けていたが、Stage の絵を shader Program の永続テクスチャへ移したことで
/// その非同期アップロード境界(「その間 draw_image は何も描かない」穴、
/// `docs/reviews/2026-08-21-stage-presenter-decision.md` 事実2)自体が経路に
/// 存在しなくなったので、`stage_auto_scale` は撤去した(常に `1.0` を渡す =
/// フル解像度復帰)。
///
/// 残るのは **裁定163 Stage 下縁状態帯**が持つ `resolution_cap`(ユーザーが
/// 明示的に選ぶ上限、Auto/½/¼)だけ——[`stage::effective_preview_scale`] で
/// auto 側 `1.0` と min 合成する。`Auto` は cap=1.0固定なので合成しても
/// 値が変わらず、この関数は入力をそのまま返す(EXACT TARGET (b) 「presenter
/// へ渡る寸法 == comp 寸法」)。½/¼ が選ばれている時だけ実際に縮む
/// (nearest-neighbor — プレビュー用途なので品質は問わない、
/// `screenshot.rs::blit_letterboxed` と同じ考え方)。**画面には
/// `Length::Fill` で引き伸ばして出すので実素材解像度である必要が無い**
/// (screenshot 器具は `frame_rgba()` が返す元解像度の RGBA を別途持っている
/// — 縮めるのは presenter 用のコピーだけで、pixel 精度が要る経路には触らない)。
fn stage_presenter_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    resolution_cap: stage::PreviewResolutionCap,
) -> (u32, u32, Vec<u8>) {
    if width == 0 || height == 0 {
        return (width, height, rgba.to_vec());
    }
    let scale = stage::effective_preview_scale(1.0, resolution_cap);
    if scale >= 1.0 {
        return (width, height, rgba.to_vec());
    }

    let dst_w = ((width as f64 * scale).floor() as u32).max(1);
    let dst_h = ((height as f64 * scale).floor() as u32).max(1);

    let mut out = vec![0u8; (dst_w as usize) * (dst_h as usize) * 4];
    for dy in 0..dst_h {
        let sy = ((u64::from(dy) * u64::from(height)) / u64::from(dst_h)).min(u64::from(height) - 1)
            as u32;
        for dx in 0..dst_w {
            let sx = ((u64::from(dx) * u64::from(width)) / u64::from(dst_w))
                .min(u64::from(width) - 1) as u32;
            let si = ((sy * width + sx) * 4) as usize;
            let di = ((dy * dst_w + dx) * 4) as usize;
            out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    (dst_w, dst_h, out)
}

/// 純関数レベルの試験(裁定166 ORACLE (b) — 「presenter へ渡る寸法 ==
/// comp 寸法」を GPU/Shell を一切介さずに確かめる)。`tests/suite/
/// render_pipeline_fence.rs` は同じ主張を `Shell::stage_presenter_dims()`
/// 経由で統合試験として重ねて見ている(二重の証拠、どちらか片方が偶然
/// 通っただけではないことを示す)。
#[cfg(test)]
mod stage_presenter_rgba_tests {
    use super::*;

    /// fixture と同じ 1920×1080。**現状(裁定166 前)は red**: 旧
    /// `stage_handle_rgba` は `stage_auto_scale` が sqrt 縮小を掛けるので
    /// 816×459 になっていた — この関数はもう `stage_auto_scale` を呼ばない。
    #[test]
    fn auto_cap_passes_native_resolution_through_unchanged() {
        let width = 1920u32;
        let height = 1080u32;
        let rgba = vec![0u8; (width as usize) * (height as usize) * 4];

        let (out_w, out_h, out_rgba) =
            stage_presenter_rgba(width, height, &rgba, stage::PreviewResolutionCap::Auto);

        assert_eq!((out_w, out_h), (width, height), "Auto なのに縮んでいる");
        assert_eq!(out_rgba.len(), rgba.len());
    }

    /// ½/¼ cap は「明示的な縮小」として維持する(EXACT TARGET 2)。
    #[test]
    fn half_and_quarter_caps_still_shrink_relative_to_native_resolution() {
        let width = 1920u32;
        let height = 1080u32;
        let rgba = vec![0u8; (width as usize) * (height as usize) * 4];

        let (half_w, half_h, _) =
            stage_presenter_rgba(width, height, &rgba, stage::PreviewResolutionCap::Half);
        assert!(half_w < width && half_h < height, "½ cap で縮んでいない");

        let (quarter_w, quarter_h, _) =
            stage_presenter_rgba(width, height, &rgba, stage::PreviewResolutionCap::Quarter);
        assert!(
            quarter_w < half_w && quarter_h < half_h,
            "¼ cap が ½ よりさらに縮んでいない"
        );
    }
}


/// Stage 表示用の RGBA を作る唯一の場所(裁定166: 旧 `build_stage_handle` の
/// 置き換え — 戻り値が `image::Handle` ではなく shader Primitive が直接使う
/// `(width, height, rgba)` になった)。`stage_presenter_rgba` で縮め(resolution
/// cap ½/¼ の時だけ)、**市松が有効なら display 用の複製にだけ**
/// [`settings_pane::composite_checkerboard_with_tile_px`] を乗せる — 呼び出し
/// 側が渡す `full_rgba` 自体は一切変更しない。
///
/// `full_rgba` は呼び出し側(`refresh_frame`)が選ぶ: 市松 OFF なら
/// `RenderedFrame::rgba`(背景込みの export 真値)、市松 ON なら
/// `Engine::render_frame_without_background` の結果(裁定141、背景を敷かない
/// 可視化専用の合成)— どちらの場合も、export/screenshot が読む生値
/// (`RenderedFrame::rgba`)自体はここでは一切変更しない。
///
/// **市松v2(利用者較正 2026-08-21「市松が見えない」の根治)**: `ui_scale` を
/// 明示的に受け取り、`stage_presenter_rgba` と同じ縮小率
/// (`stage::effective_preview_scale(1.0, resolution_cap)` — 裁定166 で auto 側
/// は常に `1.0`)を自分でも算出して [`settings_pane::checkerboard_tile_px`] に
/// 渡す — comp 画素空間固定だった旧タイル寸(8px)が縮小後にさらに痩せて実質
/// 不可視になっていた根因1をここで補正する
/// (`settings_pane::checkerboard_tile_px` doc 参照)。
pub(crate) fn build_stage_presenter_rgba(
    width: u32,
    height: u32,
    full_rgba: &[u8],
    checkerboard: bool,
    resolution_cap: stage::PreviewResolutionCap,
    colors: Colors,
    ui_scale: f32,
) -> (u32, u32, Vec<u8>) {
    let (presenter_width, presenter_height, mut presenter_rgba) =
        stage_presenter_rgba(width, height, full_rgba, resolution_cap);
    if checkerboard {
        let effective_scale = stage::effective_preview_scale(1.0, resolution_cap);
        let tile_px = settings_pane::checkerboard_tile_px(ui_scale, effective_scale);
        settings_pane::composite_checkerboard_with_tile_px(
            presenter_width,
            presenter_height,
            &mut presenter_rgba,
            colors,
            tile_px,
        );
    }
    (presenter_width, presenter_height, presenter_rgba)
}

// ---------------------------------------------------------------------------
// Stage presenter — shader widget の永続テクスチャ(裁定166)。
//
// `image(frame.handle.clone())` の置き換え。`iced::widget::shader::Program`
// (`Shader<Message, P>` widget)を自前実装する — `P::Primitive` は毎フレーム
// `Program::draw` が新しく作る軽い値(Arc の参照カウントを増やすだけ)、
// `P::Primitive::Pipeline` が実際の `wgpu::Texture`/`wgpu::RenderPipeline` を
// 持つ永続状態(`iced_wgpu::primitive::Storage` に `TypeId` 単位で1個だけ
// 生きる、`iced_wgpu-0.14.0/src/primitive.rs::BlackBox::prepare` 実測)。
//
// wgpu 型はすべて `iced::wgpu`(`iced_wgpu` の re-export、workspace の
// `wgpu 27.0.1` そのもの)を通す — 新規の wgpu 直接依存を足さない
// (裁定166 決定文書、fork の re_renderer は wgpu 29.0.4 で型が別物のため
// 混ぜられない)。
// ---------------------------------------------------------------------------

/// uniform buffer のレイアウト: letterbox(vertex shader 側、NDC 空間での
/// [offset_x, offset_y, scale_x, scale_y] — widget の `bounds` を viewport その
/// ものとして扱う shader Primitive の性質上(`iced_wgpu-0.14.0/src/lib.rs` の
/// render ループが `render_pass.set_viewport` を primitive の `bounds` へ
/// 設定してから `draw` を呼ぶ、実測)、この4値だけで letterbox 矩形が NDC 上に
/// 定まる)16 byte + `pixel_scale`(fragment shader 側、残コスト調査 §1-4の
/// 修理 — cap ½/¼ の「明示的な縮小」を GPU 高速路でも表現する fragment 側
/// サンプリング粒度、`fs_main` 参照)4 byte + WGSL 構造体アラインメント
/// (`vec2<f32>` の align=8 に揃えるための)4 byte padding = 24 byte。
const STAGE_PRESENTER_UNIFORM_BYTES: u64 = 24;

/// Stage 提示 shader の WGSL。頂点は `vertex_index`(0..6)から生成する
/// full-screen quad(2三角形)——専用の vertex buffer は持たない(letterbox の
/// 位置/大きさは uniform 側で表現する)。
///
/// **裁定171 v2(M4)`fs_main` の unmultiply(実窓検分要)**: `stage_texture` は
/// 常に `Rgba8UnormSrgb`(CPU 経路の `upload_cpu`・GPU 経路の main_target
/// 双方)——GPU が `textureSample` 時に自動で sRGB→linear decode する。fork の
/// `composite.wgsl`(`crates/viewer/re_renderer/shader/composite.wgsl`)は
/// `BlendWithBackground::Premultiplied` モードで
/// 「source is already premultiplied」と明記しており、CPU 経路の
/// `frame.rgba`/`presenter_rgba` も同じ compositor 出力(`Compositor::render*`)
/// を経由するので、**サンプル結果は経路によらず premultiplied な linear 値**
/// になる(main_target を直接サンプルする GPU 高速路は composite.wgsl を
/// 一切通らないが、`Compositor::render_to_texture` のモジュール doc
/// 「main_target の生存期間」が main_target 自体は composite 前の
/// premultiplied 値であることを示している)。この render pipeline の blend
/// state(下記、`SrcAlpha`/`OneMinusSrcAlpha` — 非 premultiplied over)は
/// straight alpha を前提にしているため、`fs_main` 側で明示的に unmultiply
/// してから返す(alpha=0 での 0 除算は `max(a, eps)` で回避)。**不透明画素
/// (alpha=1)では unmultiply は数学的に恒等**(`rgb/1.0 == rgb`)なので、
/// 既定の不透明黒背景コンポジションでは無改造時と見た目が変わらないはず——
/// 変わるのは半透明が絡む場合(市松 ON・透明背景プリセット)だけ。
/// **KNOWN.md 記載どおり、Stage の GPU 実描画は headless では検証できない
/// (`iced_test::simulator` が `Widget::draw` を叩かない)ので実窓検分が必須**。
const STAGE_PRESENTER_WGSL: &str = r#"
struct Uniforms {
    offset: vec2<f32>,
    scale: vec2<f32>,
    pixel_scale: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var stage_texture: texture_2d<f32>;
@group(0) @binding(2) var stage_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let uv = corners[vertex_index];

    var out: VertexOutput;
    out.position = vec4<f32>(
        uniforms.offset.x + uv.x * uniforms.scale.x,
        uniforms.offset.y - uv.y * uniforms.scale.y,
        0.0,
        1.0,
    );
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 残コスト調査(§1-4)の修理: `pixel_scale` < 1.0(GPU 高速路 + cap ½/¼)の
    // 間だけ、UV を「comp ネイティブ解像度 × pixel_scale」個のブロックへ量子化
    // してからサンプルする——GPU 高速路(main_target はネイティブ解像度のまま、
    // `Shell::refresh_frame` doc 参照)でも cap の「明示的な縮小」を、テクスチャ
    // の実サイズは変えずに再現する(CPU 経路の nearest-neighbor 事前縮小と
    // 同じ見た目、`stage_presenter_rgba` 参照)。
    //
    // `pixel_scale == 1.0`(CPU 経路は常にこれ——既にテクスチャ自体が cap
    // 相当に縮小済み、`StagePresenterProgram` doc 参照。GPU 経路も cap=Auto
    // の間は 1.0)の間は量子化を一切しない——素通しの `textureSample` のまま
    // (裁定166 の見た目を無改変で保つ。仮に `grid == dims` として量子化しても
    // 数学的にはテクセル中心へのスナップに退化するだけだが、それでも通常の
    // bilinear 補間からわずかにズレるため、"変える理由が無い経路は本当に
    // 何も変えない" を優先する)。
    var uv = in.uv;
    if (uniforms.pixel_scale < 1.0) {
        let dims = vec2<f32>(textureDimensions(stage_texture));
        let grid = max(dims * uniforms.pixel_scale, vec2<f32>(1.0));
        // WGSL の `/` は vecN/vecN か T/T のみ(scalar/vector 混在は `*` だけ
        // 許される) — `1.0 / grid` は無効なので `vec2<f32>(1.0) / grid` にする。
        let cell = vec2<f32>(1.0) / grid;
        uv = (floor(uv / cell) + vec2<f32>(0.5)) * cell;
    }
    let sampled = textureSample(stage_texture, stage_sampler, uv);
    // 裁定171 v2(M4、上のモジュール doc 参照): サンプル値は premultiplied
    // alpha。この pipeline の blend state は straight alpha を前提にしている
    // ので、ここで unmultiply する。alpha=1(不透明)では恒等。
    let straight_rgb = sampled.rgb / max(sampled.a, 1e-6);
    return vec4<f32>(straight_rgb, sampled.a);
}
"#;

/// `bounds`(widget local、論理px)へ comp(`width`×`height`)を letterbox で
/// 収めた矩形を、shader の viewport(=widget `bounds` そのもの)基準の NDC
/// offset/scale へ変換する。letterbox の実際の幾何は
/// [`stage::letterboxed_rect`](`image` widget の既定 `ContentFit::Contain` を
/// Rust で再現した単一源、`screenshot.rs::blit_letterboxed` と共有)をそのまま
/// 呼ぶ — 2箇所目の letterbox 実装を作らない(裁定166 EXACT TARGET 1)。
///
/// 退化(bounds/comp が 0 幅高)した時は `[0.0; 4]` を返す — 頂点が全て同じ
/// NDC 点に潰れるだけで、`draw` 自体は panic せず何も見えない矩形を描いて
/// 終わる(M16: 描けなくても panic しない)。
fn stage_presenter_letterbox_ndc(bounds: iced::Rectangle, width: u32, height: u32) -> [f32; 4] {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return [0.0; 4];
    }
    let comp = CompSpec { width, height };
    let Some(rect) = stage::letterboxed_rect(bounds, comp) else {
        return [0.0; 4];
    };

    let rel_x = (rect.x - bounds.x) / bounds.width;
    let rel_y = (rect.y - bounds.y) / bounds.height;
    let rel_w = rect.width / bounds.width;
    let rel_h = rect.height / bounds.height;

    // NDC: x+ は右、y+ は上。widget 左上(rel_x, rel_y)が NDC の
    // (offset_x, offset_y)、右下(rel_x+rel_w, rel_y+rel_h)が
    // (offset_x + 2*rel_w, offset_y - 2*rel_h) になるよう解く。
    [rel_x * 2.0 - 1.0, 1.0 - rel_y * 2.0, rel_w * 2.0, rel_h * 2.0]
}

#[cfg(test)]
mod stage_presenter_letterbox_ndc_tests {
    use super::*;

    /// bounds と comp が同じアスペクト(16:9)なら letterbox 帯が無い —
    /// widget いっぱいに描く、つまり NDC の [-1,1]×[-1,1] を丸ごと使う。
    #[test]
    fn matching_aspect_fills_the_full_ndc_range() {
        let bounds = iced::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1600.0,
            height: 900.0,
        };
        let [offset_x, offset_y, scale_x, scale_y] = stage_presenter_letterbox_ndc(bounds, 1920, 1080);
        assert!((offset_x - -1.0).abs() < 1e-6);
        assert!((offset_y - 1.0).abs() < 1e-6);
        assert!((scale_x - 2.0).abs() < 1e-6);
        assert!((scale_y - 2.0).abs() < 1e-6);
    }

    /// 正方形の bounds へ 16:9 comp を収めると上下に帯ができる —
    /// scale_y は 2.0 未満(全高は使わない)、offset_y は 1.0 未満(上端から
    /// 少し内側)。
    #[test]
    fn narrower_bounds_letterbox_shrinks_the_vertical_scale() {
        let bounds = iced::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 900.0,
            height: 900.0,
        };
        let [_offset_x, offset_y, _scale_x, scale_y] = stage_presenter_letterbox_ndc(bounds, 1920, 1080);
        assert!(scale_y < 2.0, "letterbox 帯があるのに scale_y が全高のまま: {scale_y}");
        assert!(offset_y < 1.0, "letterbox 帯があるのに offset_y が上端のまま: {offset_y}");
    }

    /// 退化した bounds(幅0)では panic せず全ゼロを返す(M16)。
    #[test]
    fn degenerate_bounds_returns_all_zero_without_panicking() {
        let bounds = iced::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 900.0,
        };
        assert_eq!(stage_presenter_letterbox_ndc(bounds, 1920, 1080), [0.0; 4]);
    }
}


/// Stage の絵を描く shader widget の `Program`(裁定166)。**書ける状態を
/// 持たない**(`State = ()`)— カメラ操作等は別 widget(`stage::StageOverlay`、
/// `stack!` でこの上に重なる)が受ける、既存構造は無改変(`stage_pane` 参照)。
#[derive(Debug)]
pub(crate) struct StagePresenterProgram {
    pub(crate) source: PresenterSource,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) generation: u64,
    /// 残コスト調査(§1-4)の修理: fragment 側サンプリング粒度(`fs_main` の
    /// `pixel_scale` uniform へそのまま渡す)。`1.0` = 通常サンプリング
    /// (縮小無し)、`0.5`/`0.25` = ½/¼ cap 相当の粗さ。`stage_pane` 側が
    /// `PresenterSource::Cpu`/`Gpu` を見て決める(doc 参照)。
    pub(crate) pixel_scale: f32,
}

impl shader::Program<Message> for StagePresenterProgram {
    type State = ();
    type Primitive = StagePresenterPrimitive;

    fn draw(&self, _state: &Self::State, _cursor: iced::mouse::Cursor, bounds: iced::Rectangle) -> Self::Primitive {
        StagePresenterPrimitive {
            source: self.source.clone(),
            width: self.width,
            height: self.height,
            generation: self.generation,
            pixel_scale: self.pixel_scale,
            letterbox: stage_presenter_letterbox_ndc(bounds, self.width, self.height),
        }
    }
}

/// 1描画分の Stage 提示データ。**`Program::draw` が描画のたびに新しく作る**
/// (`iced_widget::shader::Program::draw` の契約)——だが [`PresenterSource`] は
/// `Arc` を貸す/複製するだけなので、内容が変わらない限り実コピーのコストは
/// ゼロ。実際に GPU 側の資源(CPU 経路= `queue.write_texture`・GPU 経路=
/// `Engine::render_resolved_to_texture`)を動かすかどうかは `generation` を
/// `StagePresenterPipeline` 側の記憶と比較して決める(裁定166/裁定171 v2
/// EXACT TARGET 1/2「フレーム内容が変わった時だけ」)。
#[derive(Debug)]
pub(crate) struct StagePresenterPrimitive {
    source: PresenterSource,
    width: u32,
    height: u32,
    generation: u64,
    /// `fs_main` の `pixel_scale` uniform(`StagePresenterProgram` doc 参照)。
    /// letterbox と同じく世代ゲートの対象外 — cap を巡回するだけなら世代を
    /// 進めない Message は無い(`CycleResolutionCap` は presenter_generation を
    /// 進める側)が、万一ズレても軽い float 1個の書き込みなので実害は無い。
    pixel_scale: f32,
    /// NDC 空間での letterbox 矩形 [offset_x, offset_y, scale_x, scale_y]
    /// (`stage_presenter_letterbox_ndc` 参照)。widget bounds が変わるたび
    /// (pane resize)再計算が要るので、世代ゲートの対象外(4 float の書き込み
    /// は軽い — `iced_wgpu::image::Layer::prepare` も transform uniform を
    /// 毎フレーム書いている、同じ考え方)。
    letterbox: [f32; 4],
}

impl shader::Primitive for StagePresenterPrimitive {
    type Pipeline = StagePresenterPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        pipeline.resolve(device, queue, self);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        pipeline.draw(render_pass)
    }
}

/// comp 寸法変化時だけ再作成する実体(裁定166 EXACT TARGET 1「永続
/// `wgpu::Texture`」)。`bind_group` はテクスチャ view を束ねているので、
/// テクスチャ再作成のたびに一緒に作り直す(`uniform_buffer`/`sampler` は
/// `StagePresenterPipeline` 側で使い回す)。**CPU フォールバック経路専用**
/// (裁定171 v2 §0-6、`PresenterSource::Cpu`)——GPU 高速路は
/// [`StagePresenterGpuTarget`] を使う。
pub(crate) struct StagePresenterTexture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    /// 直近でこのテクスチャへ実際に書き込んだ世代。`None` = まだ一度も
    /// 書いていない(テクスチャ再作成直後は必ず `None` に戻す — 新しい
    /// テクスチャの中身は不定なので、世代が偶然一致しても再アップロードが
    /// 要る)。
    uploaded_generation: Option<u64>,
}

/// 裁定171 v2(M4)。GPU 高速路が [`Engine::render_resolved_to_texture`] から
/// 直接受け取った main_target(+それを束ねた bind_group)。**CPU readback も
/// `queue.write_texture` もしない** — `texture`/`view` は fork の
/// `GpuTexture`(main_target)から `clone()` した薄いハンドル
/// (`motolii-compositor::Compositor::render_to_texture` のモジュール doc
/// 「main_target の生存期間」参照——次にこの Pipeline が GPU 高速路を再度
/// 呼ぶ時まで有効)。
pub(crate) struct StagePresenterGpuTarget {
    width: u32,
    height: u32,
    /// `bind_group` が参照している view の親 texture。**明示的に握り続ける**
    /// (drop すると view 経由の参照だけが残る形になり得るため、texture 自体も
    /// このスコープに留める——wgpu は resource の生存を内部で追跡するので
    /// 実害は無いはずだが、疑わしきは持つ側に倒す)。
    #[allow(dead_code)]
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    /// 直近でこの target を作った時の世代。`None` = まだ一度も描いていない。
    resolved_generation: Option<u64>,
}

/// `StagePresenterPipeline::draw` がどちらの bind_group を使うかの選択
/// (裁定171 v2 M4)。`prepare`(`resolve`)が世代ゲート越しに更新する。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum ActivePresenter {
    #[default]
    None,
    Cpu,
    Gpu,
}

/// Stage 提示 shader の永続 GPU 状態。`iced_widget::shader::Storage` に
/// `TypeId::of::<StagePresenterPrimitive>()` を鍵として1個だけ生きる
/// (iced の仕組みそのもの、`shader::Program`/`Pipeline` の doc 参照)。
pub(crate) struct StagePresenterPipeline {
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    /// CPU フォールバック経路(裁定171 v2 §0-6)。裁定166 の経路そのまま、
    /// 無改造。
    cpu_texture: Option<StagePresenterTexture>,
    /// 裁定171 v2(M4)。`Compositor::with_device` の上に組んだ Engine —
    /// **この Pipeline インスタンスが所有**(decode/upload キャッシュもここに
    /// 付いてくる、supervisor 裁定の推奨構造どおり)。Shell 側の headless
    /// `Engine`(export/screenshot 真値専用)とは完全に別インスタンス。
    gpu_engine: Engine,
    /// GPU 高速路が直近描いた main_target(裁定171 v2 M4)。
    gpu_target: Option<StagePresenterGpuTarget>,
    /// 直近の `resolve` がどちらの経路を使ったか——`draw` はこれで bind_group
    /// を選ぶ(CPU/GPU 両方の bind_group が生きていても、表示すべきは
    /// 「今のフレームで実際に描いた方」だけ)。
    active: ActivePresenter,
}

impl shader::Pipeline for StagePresenterPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-shell::stage_presenter sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            // wgpu 29(M01 統一後): mipmap は専用の `MipmapFilterMode` 型に分離された
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-shell::stage_presenter uniforms"),
            size: STAGE_PRESENTER_UNIFORM_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-shell::stage_presenter bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // 残コスト調査(§1-4)の修理: `pixel_scale` を `fs_main` も
                    // 読むようになったので FRAGMENT を足す(letterbox の
                    // offset/scale は引き続き vertex 側専用)。
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(STAGE_PRESENTER_UNIFORM_BYTES),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motolii-shell::stage_presenter pipeline layout"),
            // wgpu 29: layout は Option 化・push_constant_ranges は immediate_size へ
            bind_group_layouts: &[Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-shell::stage_presenter shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(STAGE_PRESENTER_WGSL)),
        });

        // blend state は `iced_wgpu::image` の pipeline(`src/image/mod.rs`)と
        // 同じ非 premultiplied alpha "over" — Stage の絵は元々 image widget
        // 経由でこの blend で描かれていたので、見た目のパリティをそのまま保つ。
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-shell::stage_presenter render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // 裁定171 v2(M4): iced が渡す device/queue の上に、この Pipeline
        // 専用の Engine(`Compositor::with_device` 版)を組む——供給者側
        // (compositor)のクローンではなく、`wgpu::Device`/`Queue` 自体が薄い
        // ハンドル(clone 可能、compositor 側 doc・`with_device` の実測どおり)
        // なので、ここで clone しても新しい GPU を建てるわけではない。
        // 失敗したら panic(`Shell::new` の `Engine::new().expect(...)` と
        // 同じ規律 — GPU が無ければ Stage 自体が成立しない)。
        let gpu_engine =
            Engine::with_device(device.clone(), queue.clone()).expect("GPU 高速路の Engine を用意できない");

        Self {
            render_pipeline,
            sampler,
            texture_bind_group_layout,
            uniform_buffer,
            cpu_texture: None,
            gpu_engine,
            gpu_target: None,
            active: ActivePresenter::None,
        }
    }
}

impl StagePresenterPipeline {
    /// **裁定171 v2(M4)入口**。`primitive.source` を見て CPU/GPU いずれかの
    /// 経路で実際に描き(世代ゲート越し)、letterbox uniform を書く。
    /// letterbox は経路に関わらず毎回書く(widget bounds は世代と無関係に
    /// 変わりうる — pane resize)。旧 `upload` の後継 — 引数を
    /// `&StagePresenterPrimitive` 1本にまとめる規律(clippy
    /// `too_many_arguments`)はそのまま引き継ぐ。
    fn resolve(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, primitive: &StagePresenterPrimitive) {
        match &primitive.source {
            PresenterSource::Cpu(rgba) => {
                self.upload_cpu(device, queue, primitive.width, primitive.height, primitive.generation, rgba);
                self.active = ActivePresenter::Cpu;
            }
            PresenterSource::Gpu(snapshot) => {
                self.resolve_gpu(device, primitive.width, primitive.height, primitive.generation, snapshot);
                self.active = ActivePresenter::Gpu;
            }
        }

        let letterbox = primitive.letterbox;
        let mut uniform_bytes = [0u8; STAGE_PRESENTER_UNIFORM_BYTES as usize];
        uniform_bytes[0..4].copy_from_slice(&letterbox[0].to_ne_bytes());
        uniform_bytes[4..8].copy_from_slice(&letterbox[1].to_ne_bytes());
        uniform_bytes[8..12].copy_from_slice(&letterbox[2].to_ne_bytes());
        uniform_bytes[12..16].copy_from_slice(&letterbox[3].to_ne_bytes());
        // 残コスト調査(§1-4)の修理: `fs_main` の `pixel_scale`(WGSL 構造体
        // `Uniforms.pixel_scale`、offset=16)。bytes[20..24] は WGSL 側の
        // `vec2<f32>` アラインメント(8 byte)に揃えるための padding —
        // ゼロのままで良い(`fs_main` は読まない)。
        uniform_bytes[16..20].copy_from_slice(&primitive.pixel_scale.to_ne_bytes());
        queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);
    }

    /// 裁定166 の経路——**無改造**(旧 `upload` のこの部分をそのまま移した)。
    /// comp 寸法変化時だけテクスチャを作り直し、世代が前回と違う時だけ
    /// `queue.write_texture` する(裁定166 EXACT TARGET 1)。裁定171 v2 §0-6
    /// の CPU フォールバック(市松 ON・観測カメラ中・½/¼ cap 中)がここを使う。
    fn upload_cpu(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        generation: u64,
        rgba: &Arc<Vec<u8>>,
    ) {
        if width == 0 || height == 0 {
            self.cpu_texture = None;
            return;
        }

        let needs_new_texture = match &self.cpu_texture {
            Some(existing) => existing.width != width || existing.height != height,
            None => true,
        };

        if needs_new_texture {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("motolii-shell::stage_presenter cpu texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // `iced_wgpu::image` の atlas と同じ sRGB フォーマット(`color::
                // GAMMA_CORRECTION` が既定 true の時に選ぶ物、実測)— iced 全体が
                // 線形空間で合成する前提と合わせておかないと、他 widget(背景色
                // 等)と並んだ時に明るさがズレる。GPU 高速路の main_target
                // (`re_renderer::ViewBuilder::MAIN_TARGET_COLOR_FORMAT`)も同じ
                // sRGB タグ付き format なので、`fs_main` は経路を区別せず同じ
                // sampling で扱える(下の WGSL doc 参照)。
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("motolii-shell::stage_presenter cpu bind group"),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.cpu_texture = Some(StagePresenterTexture {
                width,
                height,
                texture,
                bind_group,
                uploaded_generation: None,
            });
        }

        let presenter_texture = self.cpu_texture.as_mut().expect("直前で確実に作成済み");

        if presenter_texture.uploaded_generation != Some(generation) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &presenter_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            presenter_texture.uploaded_generation = Some(generation);
            metrics::record_presenter_upload(rgba.len());
        }
    }

    /// **裁定171 v2(M4)高速路**。CPU readback を一切しない —
    /// `Engine::render_resolved_to_texture`(→ 内部で
    /// `Compositor::render_to_texture`)が返す GPU texture/view をそのまま
    /// bind_group へ束ねるだけ(EXACT TARGET 3「readback/write_texture が
    /// 表示経路から消滅」)。世代が前回と同じなら何もしない(EXACT TARGET 2)。
    /// 描画に失敗したら(comp/layer が読めない等)前回の `gpu_target` を
    /// そのまま残す——M16「無反応より前フレームのまま」。
    fn resolve_gpu(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        generation: u64,
        snapshot: &Arc<PreviewSnapshot>,
    ) {
        if width == 0 || height == 0 {
            self.gpu_target = None;
            return;
        }

        let needs_render = match &self.gpu_target {
            Some(existing) => {
                existing.width != width || existing.height != height || existing.resolved_generation != Some(generation)
            }
            None => true,
        };
        if !needs_render {
            return;
        }

        let Ok((texture, view)) = self.gpu_engine.render_resolved_to_texture_with_shapes(
            snapshot.comp,
            snapshot.background,
            snapshot.camera,
            snapshot.time,
            &snapshot.resolved,
            &snapshot.text_documents,
            &snapshot.shape_documents,
        ) else {
            return;
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-shell::stage_presenter gpu bind group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.gpu_target = Some(StagePresenterGpuTarget {
            width,
            height,
            texture,
            bind_group,
            resolved_generation: Some(generation),
        });
        metrics::record_presenter_blit();
    }

    fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let bind_group = match self.active {
            ActivePresenter::Cpu => self.cpu_texture.as_ref().map(|texture| &texture.bind_group),
            ActivePresenter::Gpu => self.gpu_target.as_ref().map(|target| &target.bind_group),
            ActivePresenter::None => None,
        };
        let Some(bind_group) = bind_group else {
            return false;
        };
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        true
    }
}
