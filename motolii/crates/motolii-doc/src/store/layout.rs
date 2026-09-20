//! 箱と流し込み — Group の Display が Flex / Grid なら、直下の子を CSS の規則で並べる。計算は taffy
//! ([箱と流し込みの法](../../../../../docs/reviews/2026-09-14-layout-law.md))。
//! 並べた結果は書類に書かない: その時刻の子の位置・大きさ・輪郭の伸びを解くだけ。
//! 子の Position は並べた位置からのずれ(`position: relative`)、Scale は `zoom`(箱ごと大きくなり隣を押す)。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;


pub use crate::doc::store::scratch::{Frame, LayoutCache, Memo, Scratch, Slot};

mod time;


use crate::doc::core::RationalTime;
use crate::doc::eval::Value;
use crate::doc::store::{property, LayerId, LayerSource, PropertyId, StoreError, StoreView};

pub const DISPLAY: &str = "layout.display";
pub const FLEX_DIRECTION: &str = "layout.flex_direction";
pub const FLEX_WRAP: &str = "layout.flex_wrap";
pub const JUSTIFY_CONTENT: &str = "layout.justify_content";
pub const ALIGN_ITEMS: &str = "layout.align_items";
pub const GAP: &str = "layout.gap";
/// 順番の向き(提案 2026-09-16、Cavalry の Scheduling Group「Schedule from End」の写し)。箱の Stagger は移り方の遅れを
/// 配るだけでなく、子の**時刻そのもの**(鍵・効果・物理の集まり)を Stagger 秒の幅で層の順にずらす。
/// From End が On なら終わりに揃う向き(早い子が先に着く)、Off なら始まりに揃う(遅い子が後から始まる)。
pub const FROM_END: &str = "layout.from_end";
pub const PADDING: &str = "layout.padding";
pub const GRID_COLUMNS: &str = "layout.grid_columns";
pub const GRID_ROWS: &str = "layout.grid_rows";
/// 格子の線ごとの太さ(fr)。`layout.column.<n>` / `layout.row.<n>`、n は 1 から。鍵が打てる。
pub const COLUMN_PREFIX: &str = "layout.column.";
pub const ROW_PREFIX: &str = "layout.row.";
pub const BACKGROUND: &str = "layout.background";
pub const BORDER_RADIUS: &str = "layout.border_radius";
/// 箱から落ちる影(CSS の box-shadow: offset-x offset-y blur spread color、Figma の Drop shadow)。背景の後ろに描く。
/// ぼかしは、箱を広げて薄くした角丸の矩形を重ねて近づける(描く道は背景と同じ形の道、ぼかしの効果は使わない)。
pub const SHADOW_COLOR: &str = "layout.shadow_color";
pub const SHADOW_OFFSET: &str = "layout.shadow_offset";
pub const SHADOW_BLUR: &str = "layout.shadow_blur";
pub const SHADOW_SPREAD: &str = "layout.shadow_spread";
pub const OVERFLOW: &str = "layout.overflow";
/// 箱の 4 辺を内へ削る(CSS の `clip-path: inset(top right bottom left round r)`)。px、辺ごとに鍵が打てる。
/// Overflow とは別の法: Overflow が Visible でも、どれかの辺が 0 でなければ削った箱で子孫と自分の背景を切る。
pub const CLIP_TOP: &str = "layout.clip_top";
pub const CLIP_RIGHT: &str = "layout.clip_right";
pub const CLIP_BOTTOM: &str = "layout.clip_bottom";
pub const CLIP_LEFT: &str = "layout.clip_left";
pub const CLIP_RADIUS: &str = "layout.clip_radius";
pub const HORIZONTAL_SIZING: &str = "layout.horizontal_sizing";
pub const VERTICAL_SIZING: &str = "layout.vertical_sizing";
pub const WIDTH: &str = "layout.width";
pub const HEIGHT: &str = "layout.height";
pub const POSITION_TYPE: &str = "layout.position_type";
pub const ALIGN_SELF: &str = "layout.align_self";
pub const COLUMN_START: &str = "layout.column_start";
pub const ROW_START: &str = "layout.row_start";
pub const COLUMN_SPAN: &str = "layout.column_span";
pub const ROW_SPAN: &str = "layout.row_span";
pub const OBJECT_FIT: &str = "layout.object_fit";
pub const DEPTH_ALIGNMENT: &str = "layout.depth_alignment";
/// 映像の中身が格子の枠を占める(CSS Exclusions の写し): Blob Track を持つ層を指すと、その塊が重なる枠は空けて、
/// 子は残りの枠へ流れる。塊は host が解いた箱(解析の橋)。
pub const EXCLUSIONS: &str = "layout.exclusions";
/// 間合いの法(2026-09-14): 物が外との距離を宣言する。容器の子なら CSS の margin、容器の外の兄弟同士は押し合う。
pub const MARGIN: &str = "layout.margin";
/// 詰まった時に誰がどれだけ譲るかの比(CSS の flex-shrink)。0 は譲らない。
pub const FLEX_SHRINK: &str = "layout.flex_shrink";
/// 解いた行き先が変わった時の移り方(CSS の transition)。秒と、区間の形。
pub const TRANSITION_DURATION: &str = "layout.transition_duration";
pub const TRANSITION_EASING: &str = "layout.transition_easing";
/// 変形の中心を箱の割合で(CSS の transform-origin のキーワード)。Anchor は書いた px のまま、他は毎コマ層の箱から解く:
/// 文字が伸びても Bottom Left なら左下の角が Position に居続け、そこを中心に拡大・回転する。
pub const TRANSFORM_ORIGIN: &str = "layout.transform_origin";
/// 箱の輪郭を道にする(CSS の offset-path: border-box、offset-distance、offset-rotate)。道は親の箱(並べる Group の箱、角丸込み)、
/// 親が無ければ画面の枠。左上の角の後から時計回りに一周を 0〜100%(はみ出しは回る)。Auto なら道の向きに回る。Position の代わり。
pub const OFFSET_PATH: &str = "layout.offset_path";
/// 箱からの距離で効き方を変える(C4D の Fields の Box と Plain エフェクタ): Field が指す層の箱の中で強さ 1、縁から Field Falloff の
/// 距離で 0(滑らかに)。強さに応じて、自分の箱(Repeater の写しは 1 枚ずつ)を中心から Field Scale 倍、不透明度を Field Opacity 倍、
/// 箱の中心から離れる向きに Field Push px。画面の見え方だけ(並びは変えない)。
pub const FIELD: &str = "layout.field";
pub const FIELD_FALLOFF: &str = "layout.field_falloff";
pub const FIELD_SCALE: &str = "layout.field_scale";
pub const FIELD_OPACITY: &str = "layout.field_opacity";
pub const FIELD_PUSH: &str = "layout.field_push";
pub const OFFSET_DISTANCE: &str = "layout.offset_distance";
pub const OFFSET_ROTATE: &str = "layout.offset_rotate";
/// 移り方を始めるまでの遅れ(CSS の transition-delay)。
pub const TRANSITION_DELAY: &str = "layout.transition_delay";
/// 並べる容器が子の移り方の遅れを配る(GSAP の stagger の amount と from)。遅れ = Stagger × 起点からの距離 / 容器の最大の距離。
/// 物の手触り(提案 2026-09-16 の felt な軸)。0 = 返す(跳ねる・硬い)、0.5 = 吸う(布・スポンジ)、
/// 1 = 引きずる(粘る・くっつく)。物理の解き手の摩擦・反発・減衰に訳す。
pub const HARDNESS: &str = "layout.hardness";
/// 物の重さ。0 なら動かない(留め具のように扱う)。既定は大きさから。
pub const HEAVINESS: &str = "layout.heaviness";
pub const STAGGER: &str = "layout.stagger";
pub const STAGGER_FROM: &str = "layout.stagger_from";
/// 繰り返し(CSS `animation-iteration-count: infinite` + `animation-direction`、Remotion `<Loop durationInFrames>`):
/// 層の時刻を Loop Duration 秒の周期で畳んでから鍵・効果を読む(0 = 繰り返さない)。Alternate は往復(三角波)。
pub const LOOP_DURATION: &str = "layout.loop_duration";
pub const LOOP_DIRECTION: &str = "layout.loop_direction";
/// 文字が避けて流れる物の形(CSS `shape-outside`、宣言するのは避けられる物の側)。同じ親の、折り返す文字が避ける。
/// Margin Box = 箱 + Margin、Content = 形の輪郭(Blob Track の層は塊の箱)。間は `Shape Margin`。
pub const SHAPE_OUTSIDE: &str = "layout.shape_outside";
pub const SHAPE_MARGIN: &str = "layout.shape_margin";
/// 他の物の箱に付いて置く(CSS の anchor positioning: `position-anchor` と `position-area`)。
/// 付く相手の箱の外側 9 か所(Center は重ねる)へ、自分の Margin だけ離して置く。Blob Track の層なら ID の一番小さい塊。
/// 付いた物は流れの外(CSS の absolute と同じ、押し合わない)。
pub const POSITION_ANCHOR: &str = "layout.position_anchor";
/// 2 つ目の相手(提案 2026-09-16、CSS anchor positioning は辺ごとに別の相手を指せる: `top: anchor(--a bottom); bottom: anchor(--b top)`)。
/// 指すと、相手の箱は 2 つの箱の重なり(軸ごとに、重ならなければ間)。Sync の「重なった所が箱になる」。
pub const POSITION_ANCHOR_2: &str = "layout.position_anchor_2";
/// 格子へ吸い付く(Grid の Group の子、流れの外の子と Repeater の写し): 箱の左上を一番近い升目の角へ寄せる強さ 0..1。
/// Size が Fields なら大きさも升目の倍数に丸める。先例: Müller-Brockmann のモジュラーグリッド、C4D MoGraph の Quantize、
/// Illustrator / Photoshop の Snap to Grid。
pub const SNAP_TO_GRID: &str = "layout.snap_to_grid";
/// 親の箱への制約(Figma の Constraints): 並べる Group の流れの外の子が、親の箱の大きさが変わった時にどう付いていくか。
/// 基準は時刻 0 の親の箱(デザインした時の大きさ)。Left / Top は今のまま、Right / Bottom は向こうの辺からの距離を保つ、
/// Left & Right / Top & Bottom は両方の辺からの距離を保って伸びる、Center は中心からのずれを保つ、Scale は割合で。
pub const HORIZONTAL_CONSTRAINT: &str = "layout.horizontal_constraint";
pub const VERTICAL_CONSTRAINT: &str = "layout.vertical_constraint";
pub const SNAP_SIZE: &str = "layout.snap_size";
pub const POSITION_AREA: &str = "layout.position_area";
/// 箱と箱をつなぐ線(`store/connect.rs`)。形の層の欄。
pub const CONNECT_FROM: &str = "connect.from";
pub const CONNECT_TO: &str = "connect.to";
pub const FROM_SIDE: &str = "connect.from_side";
pub const TO_SIDE: &str = "connect.to_side";
pub const LINE_PATH: &str = "connect.path";
pub const SLACK: &str = "connect.slack";
pub const DASH: &str = "connect.dash";
pub const DASH_GAP: &str = "connect.dash_gap";
pub const DASH_OFFSET: &str = "connect.dash_offset";
/// 1 つの箱をなぞる形(`Connect From` だけを持つ時): 外枠(CSS の outline)・角の掴み(Figma の選択)・対角線・内接円・画面を横切る補助線。
pub const TRACE: &str = "connect.trace";
pub const HANDLE_SIZE: &str = "connect.handle_size";
pub const READOUT: &str = "readout.kind";
pub const READOUT_OF: &str = "readout.of";
/// 並びに効く回転(visionOS の rotation3DLayout): 回した物の軸に沿った箱で並べる。層の Rotation / Tilt は見た目だけ。
pub const LAYOUT_ROTATION: &str = "layout.rotation";
pub const LAYOUT_TILT_X: &str = "layout.tilt_x";
pub const LAYOUT_TILT_Y: &str = "layout.tilt_y";
/// Flex Direction の Depth(奥へ積む): 子を奥行き + Gap ずつ奥へ、面の上の位置は Align Items で揃える。
pub const DIRECTION_DEPTH: i64 = 4;

/// 欄: (property, 窓の名前, 既定値, 範囲, 選択肢)。名前は CSS の語、大きさの決め方だけ Figma(裁定 2026-09-14)。
pub type Row = (&'static str, &'static str, Value, Option<(f64, f64)>, &'static [&'static str]);

const SIZING: &[&str] = &["Hug", "Fill", "Fixed"];

/// Group の欄。Display が None の間は Display だけが意味を持つ。
pub const GROUP_ROWS: &[Row] = &[
    (DISPLAY, "Display", Value::Enum(0), None, &["None", "Flex", "Grid"]),
    (FLEX_DIRECTION, "Flex Direction", Value::Enum(0), None, &["Row", "Column", "Row Reverse", "Column Reverse", "Depth"]),
    (FLEX_WRAP, "Flex Wrap", Value::Enum(0), None, &["No Wrap", "Wrap", "Wrap Reverse"]),
    (JUSTIFY_CONTENT, "Justify Content", Value::Enum(0), None, &["Start", "End", "Center", "Space Between", "Space Around", "Space Evenly"]),
    (ALIGN_ITEMS, "Align Items", Value::Enum(0), None, &["Stretch", "Start", "End", "Center"]),
    (DEPTH_ALIGNMENT, "Depth Alignment", Value::Enum(0), None, &["Back", "Center", "Front"]),
    (GRID_COLUMNS, "Grid Columns", Value::F64(2.0), Some((1.0, 64.0)), &[]),
    (GRID_ROWS, "Grid Rows", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (EXCLUSIONS, "Exclusions", Value::LayerId(0), None, &[]),
    (GAP, "Gap", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (PADDING, "Padding", Value::Vec2([0.0, 0.0]), None, &[]),
    (HORIZONTAL_SIZING, "Horizontal Sizing", Value::Enum(0), None, SIZING),
    (VERTICAL_SIZING, "Vertical Sizing", Value::Enum(0), None, SIZING),
    (WIDTH, "Width", Value::F64(400.0), Some((0.0, 100000.0)), &[]),
    (HEIGHT, "Height", Value::F64(300.0), Some((0.0, 100000.0)), &[]),
    (BACKGROUND, "Background", Value::Color([1.0, 1.0, 1.0, 0.0]), None, &[]),
    (BORDER_RADIUS, "Border Radius", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (OVERFLOW, "Overflow", Value::Enum(0), None, &["Visible", "Clip", "Bounce"]),
    (CLIP_TOP, "Clip Top", Value::F64(0.0), None, &[]),
    (CLIP_RIGHT, "Clip Right", Value::F64(0.0), None, &[]),
    (CLIP_BOTTOM, "Clip Bottom", Value::F64(0.0), None, &[]),
    (CLIP_LEFT, "Clip Left", Value::F64(0.0), None, &[]),
    (CLIP_RADIUS, "Clip Radius", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (SHADOW_COLOR, "Shadow Color", Value::Color([0.0, 0.0, 0.0, 0.0]), None, &[]),
    (SHADOW_OFFSET, "Shadow Offset", Value::Vec2([0.0, 12.0]), None, &[]),
    (SHADOW_BLUR, "Shadow Blur", Value::F64(24.0), Some((0.0, 10000.0)), &[]),
    (SHADOW_SPREAD, "Shadow Spread", Value::F64(0.0), None, &[]),
    (STAGGER, "Stagger", Value::F64(0.0), Some((0.0, 60.0)), &[]),
    (STAGGER_FROM, "Stagger From", Value::Enum(0), None, &["Start", "Center", "End", "Edges"]),
    (FROM_END, "From End", Value::Enum(0), None, &["Off", "On"]),
];

/// 並ぶ子の欄(親の Display が Flex / Grid の時)。
pub const ITEM_ROWS: &[Row] = &[
    (POSITION_TYPE, "Position Type", Value::Enum(0), None, &["Relative", "Absolute"]),
    (SNAP_TO_GRID, "Snap to Grid", Value::F64(0.0), Some((0.0, 1.0)), &[]),
    (HORIZONTAL_CONSTRAINT, "Horizontal Constraint", Value::Enum(0), None, &["Left", "Right", "Left & Right", "Center", "Scale"]),
    (VERTICAL_CONSTRAINT, "Vertical Constraint", Value::Enum(0), None, &["Top", "Bottom", "Top & Bottom", "Center", "Scale"]),
    (SNAP_SIZE, "Snap Size", Value::Enum(0), None, &["Off", "Fields"]),
    (HORIZONTAL_SIZING, "Horizontal Sizing", Value::Enum(0), None, SIZING),
    (VERTICAL_SIZING, "Vertical Sizing", Value::Enum(0), None, SIZING),
    (WIDTH, "Width", Value::F64(100.0), Some((0.0, 100000.0)), &[]),
    (HEIGHT, "Height", Value::F64(100.0), Some((0.0, 100000.0)), &[]),
    (ALIGN_SELF, "Align Self", Value::Enum(0), None, &["Auto", "Stretch", "Start", "End", "Center"]),
    (COLUMN_START, "Column Start", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (ROW_START, "Row Start", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (COLUMN_SPAN, "Column Span", Value::F64(1.0), Some((1.0, 64.0)), &[]),
    (ROW_SPAN, "Row Span", Value::F64(1.0), Some((1.0, 64.0)), &[]),
    (OBJECT_FIT, "Object Fit", Value::Enum(0), None, &["Fill", "Contain", "Cover", "None"]),
    (LAYOUT_ROTATION, "Layout Rotation", Value::F64(0.0), None, &[]),
    (LAYOUT_TILT_X, "Layout Tilt X", Value::F64(0.0), None, &[]),
    (LAYOUT_TILT_Y, "Layout Tilt Y", Value::F64(0.0), None, &[]),
];

/// Overflow = Bounce(提案 2026-09-15、利用者「物理、これは嘘でできる」): 流れの外の子の箱を、親の箱の内側へ鏡で折り返す。
/// 書いた動き(鍵・揺らぎ)がまっすぐなら、壁で跳ね返るビリヤードと同じ道になる。積み上げの物理は持たない(時刻の純関数)。
/// 四角の箱は軸ごとに、角丸が短辺の半分に届く箱(円・帯)は中心からの向きに沿って、壁の間を往復する。戻り値は置き場所のずれ。
pub fn bounced(lo: [f32; 2], hi: [f32; 2], size: [f32; 2], radius: f32) -> [f32; 2] {
    // x を [a, a + room] の中へ鏡で折り返す(周期 2 room の三角波)。
    let fold = |x: f32, a: f32, room: f32| {
        if room <= 1e-3 { return a + room * 0.5; }
        let m = (x - a).rem_euclid(2.0 * room);
        a + if m <= room { m } else { 2.0 * room - m }
    };
    let origin = CANVAS_MARGIN;
    if radius * 2.0 >= size[0].min(size[1]) - 1e-3 && size[0] > 0.0 && size[1] > 0.0 {
        // 円(帯は短辺の円の中へ): 中心を通る直線の上で、向こうの壁まで往復する。
        let center = glam::vec2(origin + size[0] * 0.5, origin + size[1] * 0.5);
        let own = (glam::Vec2::from(hi) - glam::Vec2::from(lo)).max_element() * 0.5;
        let room = (size[0].min(size[1]) * 0.5 - own).max(0.0);
        let mid = (glam::Vec2::from(lo) + glam::Vec2::from(hi)) * 0.5;
        let offset = mid - center;
        let r = offset.length();
        if r <= room || r < 1e-6 { return [0.0, 0.0]; }
        let signed = fold(r, -room, 2.0 * room);
        return (center + offset / r * signed - mid).to_array();
    }
    [0, 1].map(|axis| {
        let w = hi[axis] - lo[axis];
        fold(lo[axis], origin, size[axis] - w) - lo[axis]
    })
}

/// 形の層の素材座標は、輪郭の canvas の左上(反アリアスの 1 画素の外)が原点。
/// Display の Group の箱もその座標で [1, 1]..[1 + 幅, 1 + 高さ] に置き、背景の形と子の枠が同じ所に来る。
pub const CANVAS_MARGIN: f32 = 1.0;

/// 間合いの欄(容器の中でも外でも、すべての物)。
pub const SPACE_ROWS: &[Row] = &[
    (MARGIN, "Margin", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    // 手触りは物ごと(容器の中でも外でも)。人は摩擦係数を操作しない(提案 2026-09-16)。
    (HARDNESS, "Hardness", Value::F64(0.5), Some((0.0, 1.0)), &[]),
    (HEAVINESS, "Heaviness", Value::F64(1.0), Some((0.0, 100.0)), &[]),
    (FLEX_SHRINK, "Flex Shrink", Value::F64(1.0), Some((0.0, 1000.0)), &[]),
    (TRANSITION_DURATION, "Transition Duration", Value::F64(0.0), Some((0.0, 60.0)), &[]),
    (TRANSITION_EASING, "Transition Easing", Value::Enum(0), None, &["Ease", "Linear", "Ease In", "Ease Out", "Ease In Out"]),
    (TRANSITION_DELAY, "Transition Delay", Value::F64(0.0), Some((0.0, 60.0)), &[]),
    (LOOP_DURATION, "Loop Duration", Value::F64(0.0), Some((0.0, 3600.0)), &[]),
    (LOOP_DIRECTION, "Loop Direction", Value::Enum(0), None, &["Normal", "Reverse", "Alternate", "Alternate Reverse"]),
    (FIELD, "Field", Value::LayerId(0), None, &[]),
    (FIELD_FALLOFF, "Field Falloff", Value::F64(200.0), Some((0.0, 100000.0)), &[]),
    (FIELD_SCALE, "Field Scale", Value::F64(1.0), Some((0.0, 100.0)), &[]),
    (FIELD_OPACITY, "Field Opacity", Value::F64(1.0), Some((0.0, 1.0)), &[]),
    (FIELD_PUSH, "Field Push", Value::F64(0.0), None, &[]),
    (OFFSET_PATH, "Offset Path", Value::Enum(0), None, &["None", "Border Box"]),
    (OFFSET_DISTANCE, "Offset Distance", Value::F64(0.0), None, &[]),
    (OFFSET_ROTATE, "Offset Rotate", Value::Enum(0), None, &["Auto", "None"]),
    (TRANSFORM_ORIGIN, "Transform Origin", Value::Enum(0), None, &["Anchor", "Top Left", "Top", "Top Right", "Left", "Center", "Right", "Bottom Left", "Bottom", "Bottom Right"]),
    (SHAPE_OUTSIDE, "Shape Outside", Value::Enum(0), None, &["None", "Margin Box", "Content"]),
    (SHAPE_MARGIN, "Shape Margin", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (POSITION_ANCHOR, "Position Anchor", Value::LayerId(0), None, &[]),
    (POSITION_ANCHOR_2, "Position Anchor 2", Value::LayerId(0), None, &[]),
    (POSITION_AREA, "Position Area", Value::Enum(0), None, &["None", "Top Left", "Top", "Top Right", "Left", "Center", "Right", "Bottom Left", "Bottom", "Bottom Right"]),
];

/// つなぐ線の欄(形の層)。語は leader-line.js と FigJam のコネクタ、破線は AE の Stroke > Dashes。
const SIDES: &[&str] = &["Auto", "Top", "Right", "Bottom", "Left", "Center"];
pub const CONNECT_ROWS: &[Row] = &[
    (CONNECT_FROM, "Connect From", Value::LayerId(0), None, &[]),
    (CONNECT_TO, "Connect To", Value::LayerId(0), None, &[]),
    (FROM_SIDE, "From Side", Value::Enum(0), None, SIDES),
    (TO_SIDE, "To Side", Value::Enum(0), None, SIDES),
    (LINE_PATH, "Line Path", Value::Enum(0), None, &["Straight", "Curved", "Elbow", "Hang", "Rope"]),
    (SLACK, "Slack", Value::F64(20.0), Some((0.0, 1000.0)), &[]),
    (DASH, "Dash", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (DASH_GAP, "Dash Gap", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (DASH_OFFSET, "Dash Offset", Value::F64(0.0), None, &[]),
    (TRACE, "Trace", Value::Enum(0), None, &["None", "Outline", "Handles", "Diagonals", "Circle", "Guides", "Grid", "Push"]),
    (HANDLE_SIZE, "Handle Size", Value::F64(10.0), Some((0.0, 10000.0)), &[]),
];

/// 文字が関係の値を読む(提案 2026-09-15、先例 AE の Source Text の expression と Figma の寸法の札): 文字の `#` を相手の値に置き換える(`#` が無ければ全部)。
pub const READOUT_ROWS: &[Row] = &[
    (READOUT, "Readout", Value::Enum(0), None, &["None", "Push", "Width", "Height"]),
    (READOUT_OF, "Readout Of", Value::LayerId(0), None, &[]),
];

/// 格子の線の太さの既定(fr)。
pub const TRACK_DEFAULT: f64 = 1.0;

/// 番号付きの線の名前(`layout.column.3` → `Column 3`)。
pub fn track_label(property: &str) -> Option<String> {
    let number = |prefix: &str| property.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()).filter(|n| *n >= 1);
    number(COLUMN_PREFIX).map(|n| format!("Column {n}")).or_else(|| number(ROW_PREFIX).map(|n| format!("Row {n}")))
}

pub fn row(property: &str) -> Option<&'static Row> {
    GROUP_ROWS.iter().chain(ITEM_ROWS).chain(SPACE_ROWS).chain(CONNECT_ROWS).chain(READOUT_ROWS).find(|row| row.0 == property)
}

/// 順番の欄そのもの(これを読む時は時刻をずらさない — ずらしの根拠なので)。
pub fn is_schedule_row(property: &str) -> bool {
    matches!(property, STAGGER | STAGGER_FROM | FROM_END)
}

/// 繰り返しの欄そのもの(これを読む時は時刻を畳まない — 畳みの根拠なので)。
pub fn is_loop_row(property: &str) -> bool {
    matches!(property, LOOP_DURATION | LOOP_DIRECTION)
}

pub fn choices(property: &str) -> &'static [&'static str] {
    row(property).map_or(&[], |row| row.4)
}


impl StoreView<'_> {














    pub fn number(&self, layer: LayerId, name: &str, default: f64, t: RationalTime) -> Result<f64, StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::F64(v)) => v,
            Some(Value::Enum(v)) => v as f64,
            _ => default,
        })
    }






    pub fn choice(&self, layer: LayerId, name: &str, t: RationalTime) -> Result<i64, StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::Enum(v)) => v,
            Some(Value::F64(v)) => v.round() as i64,
            _ => 0,
        })
    }

    pub fn pair(&self, layer: LayerId, name: &str, default: [f32; 2], t: RationalTime) -> Result<[f32; 2], StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            _ => default,
        })
    }

    pub fn layout_display(&self, layer: LayerId, t: RationalTime) -> Result<i64, StoreError> {
        self.display(layer, t)
    }

    /// 0 = None、1 = Flex、2 = Grid。Group 以外は None。
    pub fn display(&self, layer: LayerId, t: RationalTime) -> Result<i64, StoreError> {
        if !self.meta(layer)?.is_some_and(|m| m.source == LayerSource::Group) {
            return Ok(0);
        }
        self.choice(layer, DISPLAY, t)
    }

    /// その時刻に居る層か(居ない層は並びに参加しない、`display: none`)。
    pub fn here(&self, layer: LayerId, t: RationalTime) -> Result<bool, StoreError> {
        let (Some(meta), Some(comp)) = (self.meta(layer)?, self.composition()?) else { return Ok(false) };
        let frame = t.try_to_frame_floor(comp.fps).map_err(|e| StoreError::Property(e.to_string()))?;
        Ok(meta.timing.source_frame(frame).is_some())
    }






























}
