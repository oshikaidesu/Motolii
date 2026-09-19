//! 箱と流し込み — Group の Display が Flex / Grid なら、直下の子を CSS の規則で並べる。計算は taffy
//! ([箱と流し込みの法](../../../../../docs/reviews/2026-09-14-layout-law.md))。
//! 並べた結果は書類に書かない: その時刻の子の位置・大きさ・輪郭の伸びを解くだけ。
//! 子の Position は並べた位置からのずれ(`position: relative`)、Scale は `zoom`(箱ごと大きくなり隣を押す)。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;


mod boxes;
mod flow;
mod path;
mod text;
mod time;

pub(crate) use boxes::stretched_shape_box;
pub(crate) use path::rounded_rect_path;

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
const CANVAS_MARGIN: f32 = 1.0;

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

/// その時刻に並べた結果。`slots` は並ぶ子、`sizes` は Display の Group の箱の大きさ(素材座標で [0, 0]..size)。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub slots: HashMap<LayerId, Slot>,
    pub sizes: HashMap<LayerId, [f32; 2]>,
    /// 容器の外で押し合った物の、親の空間でのずれ。
    pub nudges: HashMap<LayerId, [f32; 2]>,
    /// 押し合いの奥行きのずれ(両方が 2D でない物同士、Position Z に足す)。
    pub nudges_z: HashMap<LayerId, f32>,
    /// Display の Group の奥行きの範囲 [手前, 奥]。揃えなら面が奥で [-奥行き, 0]、奥へ積むなら [0, 積んだ厚み]。
    pub depths: HashMap<LayerId, [f32; 2]>,
    /// Grid の Group の升目(素材座標): 列の [始, 終] と行の [始, 終]。格子へ吸い付く子と、格子を描く線が読む。
    pub fields: HashMap<LayerId, (Vec<(f32, f32)>, Vec<(f32, f32)>)>,
}

/// 並ぶ子の変換の差し替え: 層の Position と Scale の代わりに使う値と、形の輪郭の伸び。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub stretch: [f32; 2],
    /// 横が Fill の文字: その幅で折り返す(素材座標の幅、Scale で割った値)。
    pub wrap: Option<f32>,
    /// 奥行きの揃えで足す z(Position Z に足す)。面が奥(z = 0)、物は camera 側(負)へ出る。
    pub z: f32,
    /// 奥行きに足す倍率(Scale Z に掛ける)。網・点群の奥行きは描く側が xy の拡縮から伸ばすので、今は 1。
    pub scale_z: f32,
    /// 並びに効く回転(Tilt X・Tilt Y・Rotation、度)。層の Rotation / Tilt に足す。
    pub rotation: [f32; 3],
    /// 拡縮・回転の中心(素材座標)。Anchor を書いていなければ箱の中心(CSS の transform-origin: 50% 50%)。
    pub anchor: [f32; 2],
}

/// 読み view 1 つが、巡る配置のために持つ手控え。中身は全部この view の物で、
/// 寿命も view と同じ(別の view と混ぜると、巡り止めが他人の巡りを止める)。
#[derive(Default)]
pub(crate) struct Scratch {
    /// 解いたコマの配置。view は値を変えないので、時刻ごとに 1 回で足りる。
    frames: HashMap<RationalTime, std::sync::Arc<Frame>>,
    /// 解いている入れ子の深さ。0 から入った物だけがコマをまたぐ覚えへ書く。
    depth: u32,
    /// 付き合いの輪と線の輪。掛けた物は必ず外す。
    anchoring: std::collections::HashSet<(u64, i64, i64)>,
    routing: std::collections::HashSet<(u64, i64, i64)>,
    /// 箱の子の順。版ごとに覚える。
    kids: HashMap<(u64, LayerId), std::sync::Arc<Vec<LayerId>>>,
}

impl Scratch {
    /// 一番外側から入ったか。出る時は必ず `leave`。
    fn enter(&mut self) -> bool {
        let outermost = self.depth == 0;
        self.depth += 1;
        outermost
    }

    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

/// 巡り止め: 掛かれば解いてよい、掛からなければ既に自分が巡っている。外すのは掛けた者の責任。
impl Scratch {
    pub(crate) fn begin_route(&mut self, key: (u64, i64, i64)) -> bool {
        self.routing.insert(key)
    }

    pub(crate) fn end_route(&mut self, key: (u64, i64, i64)) {
        self.routing.remove(&key);
    }
}

/// view の寿命の間、時刻ごとに 1 回だけ解く(view は値を変えない)。
pub(crate) type Memo = Rc<RefCell<Scratch>>;

/// 書類が持つ、コマをまたぐ配置の覚え。版が変われば丸ごと捨てる。移り方が 1 コマに過去の時刻の配置を何十回も解くので、
/// 次のコマで同じ時刻を解き直さない(天井の棚卸し 2026-09-15)。
#[derive(Default)]
pub struct LayoutCache {
    revision: Option<crate::doc::store::Revision>,
    frames: HashMap<RationalTime, std::sync::Arc<Frame>>,
}

impl LayoutCache {
    const LIMIT: usize = 4096;
}


impl StoreView<'_> {

    /// その時刻に並べた結果。Display の Group が無ければ空。
    pub fn layout_frame(&self, t: RationalTime) -> Result<std::sync::Arc<Frame>, StoreError> {
        if let Some(hit) = self.layout_memo().borrow().frames.get(&t) {
            return Ok(hit.clone());
        }
        if let Some((cache, revision)) = self.shared_layout_cache() {
            let cache = cache.borrow();
            if cache.revision.as_ref() == Some(revision) {
                if let Some(hit) = cache.frames.get(&t) {
                    self.layout_memo().borrow_mut().frames.insert(t, hit.clone());
                    return Ok(hit.clone());
                }
            }
        }
        // 解いている間に同じ時刻を問われたら(面の Group の親を辿る時など)、空の結果で答えて巡らない。
        self.layout_memo().borrow_mut().frames.insert(t, std::sync::Arc::new(Frame::default()));
        // 解いている途中の内側の時刻は、巡り止めの空の結果を読んでいるかもしれない。コマをまたいで覚えるのは一番外側だけ。
        let outermost = self.layout_memo().borrow_mut().enter();
        let computed = self.compute_layout(t);
        self.layout_memo().borrow_mut().leave();
        let frame = std::sync::Arc::new(computed?);
        self.layout_memo().borrow_mut().frames.insert(t, frame.clone());
        if let Some((cache, revision)) = self.shared_layout_cache().filter(|_| outermost) {
            let mut cache = cache.borrow_mut();
            if cache.revision.as_ref() != Some(revision) || cache.frames.len() >= LayoutCache::LIMIT {
                cache.revision = Some(revision.clone());
                cache.frames.clear();
            }
            cache.frames.insert(t, frame.clone());
        }
        Ok(frame)
    }

    /// 並ぶ子なら、層の Position と Scale の代わりに使う値。
    pub(crate) fn laid_out(&self, layer: LayerId, t: RationalTime) -> Result<Option<Slot>, StoreError> {
        let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent else { return Ok(None) };
        if self.display(parent, t)? == 0 {
            return Ok(None);
        }
        let Some(mut slot) = self.layout_frame(t)?.slots.get(&layer).copied() else { return Ok(None) };
        // 移り方: 少し前の行き先を、区間の重みで混ぜる(位置と大きさ)。
        let mut acc = [[0.0f32; 2]; 4];
        let mut total = 0.0f32;
        for (at, weight) in self.transition_samples(layer, t)? {
            if let Some(past) = self.layout_frame(at)?.slots.get(&layer) {
                for (sum, value) in acc.iter_mut().zip([past.position, past.scale, past.stretch, past.anchor]) {
                    for axis in 0..2 {
                        sum[axis] += value[axis] * weight;
                    }
                }
                total += weight;
            }
        }
        if total > 1e-6 {
            [slot.position, slot.scale, slot.stretch, slot.anchor] = acc.map(|sum| sum.map(|v| v / total));
        }
        Ok(Some(slot))
    }












    pub(crate) fn number(&self, layer: LayerId, name: &str, default: f64, t: RationalTime) -> Result<f64, StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::F64(v)) => v,
            Some(Value::Enum(v)) => v as f64,
            _ => default,
        })
    }






    pub(crate) fn choice(&self, layer: LayerId, name: &str, t: RationalTime) -> Result<i64, StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::Enum(v)) => v,
            Some(Value::F64(v)) => v.round() as i64,
            _ => 0,
        })
    }

    fn pair(&self, layer: LayerId, name: &str, default: [f32; 2], t: RationalTime) -> Result<[f32; 2], StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            _ => default,
        })
    }

    pub(crate) fn layout_display(&self, layer: LayerId, t: RationalTime) -> Result<i64, StoreError> {
        self.display(layer, t)
    }

    /// 0 = None、1 = Flex、2 = Grid。Group 以外は None。
    pub(crate) fn display(&self, layer: LayerId, t: RationalTime) -> Result<i64, StoreError> {
        if !self.meta(layer)?.is_some_and(|m| m.source == LayerSource::Group) {
            return Ok(0);
        }
        self.choice(layer, DISPLAY, t)
    }

    /// その時刻に居る層か(居ない層は並びに参加しない、`display: none`)。
    pub(crate) fn here(&self, layer: LayerId, t: RationalTime) -> Result<bool, StoreError> {
        let (Some(meta), Some(comp)) = (self.meta(layer)?, self.composition()?) else { return Ok(false) };
        let frame = t.try_to_frame_floor(comp.fps).map_err(|e| StoreError::Property(e.to_string()))?;
        Ok(meta.timing.source_frame(frame).is_some())
    }






























}



#[cfg(test)]
mod tests {
    use super::boxes::shape_box;
    use super::*;
    use crate::doc::store::{
        blank_project, rect_shape, ContentKeyframe, ContentTrack, Document, FontRef, Intent, LayerAttrsPatch, LayerMeta, LayerTiming, TextDocument, TextDocumentStyle,
        TextJustify, TextStyleId,
    };

    const T: RationalTime = RationalTime::ZERO;

    fn add(doc: &mut Document, id: u64, source: LayerSource, parent: Option<LayerId>) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), ..Default::default() } },
        ])
        .unwrap();
        layer
    }

    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }

    fn rect(doc: &mut Document, id: u64, parent: LayerId, size: [f32; 2]) -> LayerId {
        let layer = add(doc, id, LayerSource::Shape, Some(parent));
        doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], size)] }).unwrap();
        layer
    }

    /// 画面の上の箱(world): 解いた変換を、輪郭を伸ばした後の箱に掛ける。
    fn shown(doc: &Document, layer: LayerId, t: RationalTime) -> [f32; 4] {
        let view = doc.view();
        let resolved = view.resolved_layers(t).unwrap();
        let r = resolved.iter().find(|l| l.id == layer).unwrap();
        let b = if r.source == LayerSource::Shape {
            shape_box(&crate::doc::vector::stretch_outline(&view.shapes_at(layer, t).unwrap(), r.shape_stretch)).unwrap()
        } else {
            view.layer_box(layer, t).unwrap().unwrap()
        };
        let (lo, hi) = (r.placement.transform.transform_point2(glam::vec2(b[0], b[1])), r.placement.transform.transform_point2(glam::vec2(b[2], b[3])));
        // 群の箱の左上(素材座標の 1 画素の外)から測る。
        [lo.x.min(hi.x), lo.y.min(hi.y), lo.x.max(hi.x), lo.y.max(hi.y)].map(|v| ((v - CANVAS_MARGIN) * 100.0).round() / 100.0)
    }

    fn flex_row(doc: &mut Document) -> LayerId {
        let group = add(doc, 1, LayerSource::Group, None);
        put(doc, group, DISPLAY, Value::Enum(1));
        put(doc, group, GAP, Value::F64(10.0));
        put(doc, group, PADDING, Value::Vec2([20.0, 8.0]));
        group
    }

    #[test]
    fn a_flex_row_butts_boxes_with_gap_and_padding_and_hugs_them() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        let c = rect(&mut doc, 4, group, [40.0, 50.0]);
        assert_eq!(shown(&doc, a, T), [20.0, 8.0, 120.0, 58.0]);
        assert_eq!(shown(&doc, b, T), [130.0, 8.0, 190.0, 58.0]);
        assert_eq!(shown(&doc, c, T), [200.0, 8.0, 240.0, 58.0]);
        assert_eq!(doc.view().layer_box(group, T).unwrap(), Some([1.0, 1.0, 261.0, 67.0]), "Hug: padding + boxes + gaps");
        let background = doc.view().background_shapes(group, T).unwrap();
        assert!(background.is_none(), "no Background colour, nothing to draw");
        put(&mut doc, group, BACKGROUND, Value::Color([0.2, 0.2, 0.6, 1.0]));
        let background = doc.view().background_shapes(group, T).unwrap().unwrap();
        assert_eq!(shape_box(&background), Some([1.0, 1.0, 261.0, 67.0]), "the background is drawn where the box is");
    }

    #[test]
    fn scale_is_zoom_and_position_is_a_relative_offset() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, a, property::SCALE, Value::Vec2([2.0, 1.0]));
        assert_eq!(shown(&doc, a, T), [20.0, 8.0, 220.0, 58.0], "the box grows");
        assert_eq!(shown(&doc, b, T)[0], 230.0, "and pushes the neighbour");
        put(&mut doc, a, property::POSITION, Value::Vec2([0.0, 30.0]));
        assert_eq!(shown(&doc, a, T)[1], 38.0, "Position moves the layer from its place");
        assert_eq!(shown(&doc, b, T)[0], 230.0, "without moving anyone else");
    }

    #[test]
    fn an_absolute_child_and_a_layer_out_of_time_take_no_space() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        let c = rect(&mut doc, 4, group, [40.0, 50.0]);
        put(&mut doc, a, POSITION_TYPE, Value::Enum(1));
        assert_eq!(shown(&doc, b, T)[0], 20.0);
        doc.apply(Intent::SetTiming { layer: b, timing: LayerTiming::place(50, None, 300) }).unwrap();
        assert_eq!(shown(&doc, c, T)[0], 20.0, "b is not there yet at frame 0");
    }

    #[test]
    fn grid_tracks_are_keyable_fr_and_fill_stretches_the_outline() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [
            (DISPLAY, Value::Enum(2)),
            (GRID_COLUMNS, Value::F64(2.0)),
            (GRID_ROWS, Value::F64(1.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)),
            (VERTICAL_SIZING, Value::Enum(2)),
            (WIDTH, Value::F64(400.0)),
            (HEIGHT, Value::F64(100.0)),
            ("layout.column.1", Value::F64(3.0)),
        ] {
            put(&mut doc, grid, name, value);
        }
        let wide = rect(&mut doc, 2, grid, [10.0, 10.0]);
        let round = rect(&mut doc, 3, grid, [10.0, 10.0]);
        for layer in [wide, round] {
            put(&mut doc, layer, HORIZONTAL_SIZING, Value::Enum(1));
            put(&mut doc, layer, VERTICAL_SIZING, Value::Enum(1));
        }
        put(&mut doc, round, OBJECT_FIT, Value::Enum(1));
        assert_eq!(shown(&doc, wide, T), [0.0, 0.0, 300.0, 100.0], "3fr of 400, squashed and stretched");
        assert_eq!(shown(&doc, round, T), [300.0, 0.0, 400.0, 100.0], "Contain: 100 x 100, centred in the 100 wide cell");

        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: T, value: Value::F64(3.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: RationalTime::try_from_frame(10, doc.view().composition().unwrap().unwrap().fps).unwrap(), value: Value::F64(1.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: grid, property: PropertyId::new("layout.column.1").unwrap(), track }).unwrap();
        let later = RationalTime::try_from_frame(10, doc.view().composition().unwrap().unwrap().fps).unwrap();
        assert_eq!(shown(&doc, wide, later), [0.0, 0.0, 200.0, 100.0], "the key on Column 1 moves the line");
    }

    fn put_text(doc: &mut Document, layer: LayerId, content: &str) {
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: content.to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        doc.apply(Intent::SetTextDocument { layer, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
    }

    #[test]
    fn text_filling_its_cell_wraps_at_the_cell_and_grows_down() {
        let mut doc = blank_project();
        let column = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (FLEX_DIRECTION, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(300.0))] {
            put(&mut doc, column, name, value);
        }
        let words = add(&mut doc, 2, LayerSource::Text, Some(column));
        put_text(&mut doc, words, "Taro Yamada is the creative director of this studio");
        let below = rect(&mut doc, 3, column, [40.0, 40.0]);
        let one_line = shown(&doc, below, T)[1];
        put(&mut doc, words, HORIZONTAL_SIZING, Value::Enum(1));
        let view = doc.view();
        let wrap = view.resolved_text_document(words, T).unwrap().unwrap().wrap_size.expect("wraps");
        assert_eq!(wrap[0], 300.0, "at the cell width");
        drop(view);
        assert!(shown(&doc, below, T)[1] > one_line + 40.0, "the wrapped lines push the next item down: {one_line} → {}", shown(&doc, below, T)[1]);

        put(&mut doc, words, property::SCALE, Value::Vec2([2.0, 2.0]));
        assert_eq!(doc.view().resolved_text_document(words, T).unwrap().unwrap().wrap_size.unwrap()[0], 150.0, "Scale is zoom: the words wrap at half the width, then double");
    }

    #[test]
    fn overflow_clip_cuts_every_descendant_at_the_groups_box() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        assert!(doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == a).unwrap().masks.is_empty(), "Visible cuts nothing");
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, a, property::POSITION, Value::Vec2([0.0, 30.0]));
        let resolved = doc.view().resolved_layers(T).unwrap();
        let layer = resolved.iter().find(|l| l.id == a).unwrap();
        assert_eq!(layer.masks.len(), 1);
        assert_eq!(layer.masks[0].mode, crate::doc::store::MaskMode::Intersect);
        let world: Vec<glam::Vec2> = layer.masks[0].shape.vertices.iter().map(|v| layer.placement.transform.transform_point2(glam::vec2(v.point[0] as f32, v.point[1] as f32))).collect();
        let (lo, hi) = world.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        assert_eq!((lo.round(), hi.round()), (glam::vec2(301.0, 201.0), glam::vec2(441.0, 267.0)), "the group's box on screen, whatever the child's offset");
    }

    /// CSS `clip-path: inset(10% 0 0 0)` の写し: 箱の上から 10% を削った矩形が見える範囲。辺ごとに鍵が打てる(値は px)。
    #[test]
    fn clip_inset_cuts_the_box_from_each_edge_like_css_inset() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        for (name, value) in [(HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(400.0)), (HEIGHT, Value::F64(300.0))] {
            put(&mut doc, group, name, value);
        }
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let bounds = |doc: &Document, id: LayerId| {
            let resolved = doc.view().resolved_layers(T).unwrap();
            let layer = resolved.iter().find(|l| l.id == id).unwrap().clone();
            let world: Vec<glam::Vec2> = layer.masks.iter().flat_map(|m| m.shape.vertices.iter().map(|v| layer.placement.transform.transform_point2(glam::vec2(v.point[0] as f32, v.point[1] as f32)))).collect();
            let (lo, hi) = world.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            (layer.masks.len(), lo.round(), hi.round())
        };
        assert_eq!(bounds(&doc, a).0, 0, "inset(0) cuts nothing, Overflow Visible");
        put(&mut doc, group, CLIP_TOP, Value::F64(30.0));
        assert_eq!(bounds(&doc, a), (1, glam::vec2(301.0, 231.0), glam::vec2(701.0, 501.0)), "inset(10% 0 0 0) of a 400 x 300 box: the top 30 px are gone");
        assert_eq!(bounds(&doc, group), (1, glam::vec2(301.0, 231.0), glam::vec2(701.0, 501.0)), "the group's own background is cut too (clip-path is per element)");
        put(&mut doc, group, CLIP_RIGHT, Value::F64(100.0));
        put(&mut doc, group, CLIP_BOTTOM, Value::F64(50.0));
        put(&mut doc, group, CLIP_LEFT, Value::F64(40.0));
        assert_eq!(bounds(&doc, a), (1, glam::vec2(341.0, 231.0), glam::vec2(601.0, 451.0)), "inset(top right bottom left) in CSS order");
        put(&mut doc, group, CLIP_LEFT, Value::F64(1000.0));
        let (_, lo, hi) = bounds(&doc, a);
        assert_eq!(hi.x - lo.x, 0.0, "an edge past the opposite edge leaves an empty box");
        put(&mut doc, group, CLIP_LEFT, Value::F64(40.0));
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        assert_eq!(bounds(&doc, a).0, 2, "Overflow Clip and the inset are two cuts (border-radius and `round` are separate in CSS)");
    }

    /// 箱の切りは箱の枠に付く(CSS の overflow: clip / clip-path は要素の箱に掛かり、中で transform した子孫は箱で切れる):
    /// Overflow Clip の箱の子が持つ切りは `Box`(ブロックのずれの後に箱で切る)、箱自身の inset は `Layer`(要素ごと、自分と動く)。
    #[test]
    fn a_boxs_clip_binds_to_the_box_and_a_layers_own_clip_to_the_layer() {
        use crate::doc::store::MaskFrame;
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        put(&mut doc, group, CLIP_TOP, Value::F64(10.0));
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let frames = |doc: &Document, id: LayerId| -> Vec<MaskFrame> {
            doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == id).unwrap().masks.iter().map(|m| m.frame).collect()
        };
        assert_eq!(frames(&doc, a), vec![MaskFrame::Box, MaskFrame::Box], "the child is cut by the box's Overflow and inset, both bound to the box");
        assert_eq!(frames(&doc, group), vec![MaskFrame::Layer], "the box's own inset moves with the box (clip-path is per element)");
    }

    /// Split Words + Stagger: 語ごとに時刻がずれる(鍵は 1 本、単位は箱の mask、書類に子の層は無い)。
    /// Stagger は箱の子と同じ法 = 全体の幅(GSAP の `stagger: {amount}`): 3 語で 0.2 なら 2 語目は 0.1、3 語目は 0.2 遅れる。
    #[test]
    fn split_words_give_each_word_its_own_time_under_the_texts_stagger() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let words = add(&mut doc, 2, LayerSource::Text, None);
        put_text(&mut doc, words, "ONE TWO THREE");
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::F64(0.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::F64(1.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: words, property: PropertyId::new(property::OPACITY).unwrap(), track }).unwrap();
        put(&mut doc, words, STAGGER, Value::F64(0.2));
        let copies = |doc: &Document| -> Vec<(f32, f32)> {
            let mut out: Vec<_> = doc.view().resolved_layers(at(15)).unwrap().into_iter().filter(|l| l.id == words)
                .map(|l| (l.placement.opacity, l.masks.first().map_or(f32::NAN, |m| m.shape.vertices[0].point[0] as f32))).collect();
            out.sort_by(|a, b| b.0.total_cmp(&a.0));
            out
        };
        assert_eq!(copies(&doc).len(), 1, "Split None: one layer, no unit");
        put(&mut doc, words, crate::doc::store::names::TEXT_SPLIT, Value::Enum(2));
        let c = copies(&doc);
        assert_eq!(c.len(), 3, "three words");
        assert!((c[0].0 - 0.5).abs() < 0.02 && (c[1].0 - 0.4).abs() < 0.02 && (c[2].0 - 0.3).abs() < 0.02, "at 0.5 s the words read their keys at 0.5 / 0.4 / 0.3 s: {c:?}");
        assert!(c[0].1 < c[1].1 && c[1].1 < c[2].1, "the units are in reading order, each cut to its own box: {c:?}");
        put(&mut doc, words, crate::doc::store::names::TEXT_SPLIT, Value::Enum(1));
        assert_eq!(copies(&doc).len(), 11, "Chars: the spaces are not units");
        put(&mut doc, words, crate::doc::store::names::TEXT_SPLIT, Value::Enum(3));
        assert_eq!(copies(&doc).len(), 1, "Lines: one line is not split");
    }

    /// Loop: 鍵 0 → 1 s、Loop Duration 1 なら t = 2.5 は 0.5 として読む。Alternate は奇数回目が逆向き(t = 1.5 → 0.5、1.2 → 0.8)。
    #[test]
    fn loop_folds_the_layers_time_like_css_animation_direction() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let thing = add(&mut doc, 2, LayerSource::Shape, None);
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::F64(0.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::F64(1.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: thing, property: PropertyId::new(property::OPACITY).unwrap(), track }).unwrap();
        let opacity = |doc: &Document, f: i64| match doc.view().value_at(thing, &PropertyId::new(property::OPACITY).unwrap(), at(f)).unwrap() {
            Some(Value::F64(v)) => v,
            other => panic!("{other:?}"),
        };
        assert_eq!(opacity(&doc, 75), 1.0, "no loop: past the last key it holds");
        put(&mut doc, thing, LOOP_DURATION, Value::F64(1.0));
        assert!((opacity(&doc, 75) - 0.5).abs() < 1e-6, "Normal: t = 2.5 reads as 0.5");
        assert!((opacity(&doc, 36) - 0.2).abs() < 1e-6, "Normal: t = 1.2 reads as 0.2");
        put(&mut doc, thing, LOOP_DIRECTION, Value::Enum(2));
        assert!((opacity(&doc, 45) - 0.5).abs() < 1e-6, "Alternate: t = 1.5 reads as 0.5");
        assert!((opacity(&doc, 36) - 0.8).abs() < 1e-6, "Alternate: t = 1.2 is on the way back, 0.8");
        assert!((opacity(&doc, 15) - 0.5).abs() < 1e-6, "Alternate: the first pass runs forward");
        put(&mut doc, thing, LOOP_DIRECTION, Value::Enum(1));
        assert!((opacity(&doc, 15) - 0.5).abs() < 1e-6 && (opacity(&doc, 7) - (1.0 - 7.0 / 30.0)).abs() < 1e-6, "Reverse runs every pass backwards");
        assert_eq!(doc.view().value_at(thing, &PropertyId::new(LOOP_DURATION).unwrap(), at(75)).unwrap(), Some(Value::F64(1.0)), "the loop rows themselves are read unfolded");
    }

    #[test]
    fn things_on_a_laid_out_face_share_its_point_even_when_the_face_is_tilted() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, group, property::ROTATION_Y, Value::F64(30.0));
        let flat = rect(&mut doc, 2, group, [100.0, 50.0]);
        let lifted = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, lifted, property::POSITION_Z, Value::F64(-40.0));
        let resolved = doc.view().resolved_layers(T).unwrap();
        let plane = |id| resolved.iter().find(|l| l.id == id).unwrap().placement.plane;
        let world = resolved.iter().find(|l| l.id == group).unwrap().placement.world_transform.unwrap();
        let b = doc.view().layer_box(group, T).unwrap().unwrap();
        let face = world.transform_point3(glam::vec3((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5, 0.0)).to_array();
        assert_eq!(plane(group), Some(face), "the tilted group is the face");
        assert_eq!(plane(flat), Some(face), "a flat child lies on it and stacks by order");
        assert_eq!(plane(lifted), None, "a child lifted off the face sorts by distance");
    }

    #[test]
    fn depth_alignment_sets_each_childs_z_against_the_deepest_one() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let slab = rect(&mut doc, 2, group, [100.0, 50.0]);
        let card = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, slab, property::DEPTH, Value::F64(100.0));
        let z = |doc: &Document, id| doc.view().layout_frame(T).unwrap().slots[&id].z;
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, 0.0), "Back: backs on the face, the slab stands out toward the camera");
        assert_eq!(doc.view().layout_frame(T).unwrap().depths[&group], [-100.0, 0.0]);
        put(&mut doc, group, DEPTH_ALIGNMENT, Value::Enum(1));
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, -50.0), "Center: the card floats at the slab's middle");
        put(&mut doc, group, DEPTH_ALIGNMENT, Value::Enum(2));
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, -100.0), "Front: fronts together");
        let resolved = doc.view().resolved_layers(T).unwrap();
        assert_eq!(resolved.iter().find(|l| l.id == card).unwrap().placement.z, -100.0, "the z reaches the resolved layer");
    }

    #[test]
    fn flex_direction_depth_stacks_children_back_by_thickness_and_gap() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, FLEX_DIRECTION, Value::Enum(DIRECTION_DEPTH));
        put(&mut doc, group, GAP, Value::F64(20.0));
        let back = rect(&mut doc, 2, group, [100.0, 50.0]);
        let slab = rect(&mut doc, 3, group, [100.0, 50.0]);
        let front = rect(&mut doc, 4, group, [100.0, 50.0]);
        put(&mut doc, slab, property::DEPTH, Value::F64(30.0));
        let frame = doc.view().layout_frame(T).unwrap();
        assert_eq!([frame.slots[&front].z, frame.slots[&slab].z, frame.slots[&back].z], [0.0, 20.0, 70.0], "top of the stack in front, then thickness + gap each");
        assert_eq!(frame.depths[&group], [0.0, 70.0]);
        assert_eq!(frame.slots[&front].position, frame.slots[&back].position, "one spot on the face");
    }

    #[test]
    fn layout_rotation_lays_out_the_turned_box_and_rotation_does_not() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let turned = rect(&mut doc, 2, group, [100.0, 50.0]);
        let next = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, turned, property::ROTATION, Value::F64(90.0));
        assert_eq!(shown(&doc, next, T)[0], 130.0, "Rotation is only the look");
        put(&mut doc, turned, property::ROTATION, Value::F64(0.0));
        put(&mut doc, turned, LAYOUT_ROTATION, Value::F64(90.0));
        assert_eq!(shown(&doc, next, T)[0], 80.0, "Layout Rotation: the 100 x 50 box stands 50 wide");
        let turned_box = shown(&doc, turned, T);
        assert_eq!([turned_box[0], turned_box[2] - turned_box[0], turned_box[3] - turned_box[1]], [20.0, 50.0, 100.0], "and sits in its place turned");
    }

    #[test]
    fn exclusions_leave_the_cells_a_tracked_blob_covers_empty() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [
            (DISPLAY, Value::Enum(2)), (GRID_COLUMNS, Value::F64(3.0)), (GRID_ROWS, Value::F64(3.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(300.0)), (HEIGHT, Value::F64(300.0)),
            (EXCLUSIONS, Value::LayerId(20)),
        ] {
            put(&mut doc, grid, name, value);
        }
        let cards: Vec<LayerId> = (2..11).map(|id| rect(&mut doc, id, grid, [10.0, 10.0])).collect();
        for &card in &cards {
            put(&mut doc, card, HORIZONTAL_SIZING, Value::Enum(1));
            put(&mut doc, card, VERTICAL_SIZING, Value::Enum(1));
        }
        let mut inputs = crate::doc::store::analysis::AnalysisInputs::default();
        // 真ん中の枠(comp の 101..201)に人の群れが 1 つ。
        inputs.set_blobs(LayerId(20), crate::doc::store::EffectId(0), T, vec![crate::doc::store::analysis::BlobMark { id: 1, center: [151.0, 151.0], size: [30.0, 30.0], age: 0 }]);
        let view = doc.view().with_analysis(&inputs);
        let resolved = view.resolved_layers(T).unwrap();
        let centre = |id: LayerId| {
            let r = resolved.iter().find(|l| l.id == id).unwrap();
            let b = shape_box(&crate::doc::vector::stretch_outline(&view.shapes_at(id, T).unwrap(), r.shape_stretch)).unwrap();
            r.placement.transform.transform_point2(glam::vec2((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5))
        };
        let in_middle = cards.iter().filter(|&&c| { let p = centre(c); p.x > 101.0 && p.x < 201.0 && p.y > 101.0 && p.y < 201.0 }).count();
        assert_eq!(in_middle, 0, "no card sits where the crowd is");
        assert!(cards.iter().any(|&c| centre(c).y >= 300.0), "the ninth card flows into an implicit row below the grid");
    }

    #[test]
    fn free_boxes_keep_their_declared_distance_and_the_one_that_does_not_yield_stays() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for (layer, x) in [(a, 100.0), (b, 150.0)] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, property::POSITION, Value::Vec2([x, 100.0]));
            put(&mut doc, layer, MARGIN, Value::F64(10.0));
        }
        let left = |doc: &Document, id| shown(doc, id, T)[0];
        // 素のままなら 50 px 重なる。間合い 10 ずつ → 箱の間は 20 空く。譲りは半分ずつ。
        assert_eq!(left(&doc, b) - (left(&doc, a) + 100.0), 20.0, "they stand apart by both margins");
        assert_eq!((left(&doc, a), left(&doc, b)), (65.0, 185.0), "each yields half");
        put(&mut doc, a, FLEX_SHRINK, Value::F64(0.0));
        assert_eq!((left(&doc, a), left(&doc, b)), (100.0, 220.0), "a does not yield, b takes all of it");
    }

    #[test]
    fn a_transition_moves_to_the_new_place_over_its_duration_without_state() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(10.0));
        }
        put(&mut doc, a, property::POSITION, Value::Vec2([100.0, 100.0]));
        put(&mut doc, a, FLEX_SHRINK, Value::F64(0.0));
        // b は 30 コマ目に a の真上へ飛び込む(鍵は Hold)。押し戻されて右へ。
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::Vec2([600.0, 100.0]), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::Vec2([150.0, 100.0]), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: b, property: PropertyId::new("position").unwrap(), track }).unwrap();
        put(&mut doc, b, TRANSITION_DURATION, Value::F64(0.5));
        put(&mut doc, b, TRANSITION_EASING, Value::Enum(1));
        // 移り方が掛かるのは解いた関係(押し戻し)だけ。b 自身の鍵の動きは区間イージングの係で、そのまま効く。
        let x = |f: i64| shown(&doc, b, at(f))[0];
        assert!(x(30) < 160.0, "b lands where its key says, on top of a, and only begins to be pushed: {}", x(30));
        assert!(x(37) > 170.0 && x(37) < 210.0, "halfway through the duration it is being pushed out: {}", x(37));
        assert_eq!(x(45), 220.0, "after the duration it rests at the solved distance");
        assert_eq!(x(37), shown(&doc, b, at(37))[0], "the same frame asked again gives the same picture");
    }

    #[test]
    fn a_container_staggers_its_childrens_transitions_by_distance_and_nothing_jumps() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        // 1 行に 4 つ並べ、30 コマ目に Flex Direction を Column へ(Hold)。子は下へ積み直す。
        let group = flex_row(&mut doc);
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::Enum(0), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::Enum(1), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: group, property: PropertyId::new(FLEX_DIRECTION).unwrap(), track }).unwrap();
        let children: Vec<LayerId> = (2..6).map(|id| rect(&mut doc, id, group, [60.0, 60.0])).collect();
        for &child in &children {
            put(&mut doc, child, TRANSITION_DURATION, Value::F64(0.5));
            put(&mut doc, child, TRANSITION_EASING, Value::Enum(4));
        }
        put(&mut doc, group, STAGGER, Value::F64(1.0));
        let y = |doc: &Document, child: LayerId, f: i64| shown(doc, child, at(f))[1];
        // 最初の子は並びの上で位置が変わらない。2 番目から、起点から遠いほど遅れて動き出す。
        let started = |doc: &Document, child: LayerId| (30..120).find(|f| (y(doc, child, *f) - y(doc, child, 29)).abs() > 1.0);
        let (s1, s3) = (started(&doc, children[1]).expect("moves"), started(&doc, children[3]).expect("moves"));
        assert!(s3 > s1 + 5, "the far child starts later than the near one: {s1} vs {s3}");
        for &child in &children {
            let ys: Vec<f32> = (25..120).map(|f| y(&doc, child, f)).collect();
            // 速さの変わり方(2 階の差)が小さい = 跳ばない、尖らない。
            let biggest = ys.windows(3).map(|w| (w[2] - 2.0 * w[1] + w[0]).abs()).fold(0.0, f32::max);
            assert!(biggest < 12.0, "no frame jumps or kinks (largest change of speed {biggest})");
        }
        put(&mut doc, group, STAGGER_FROM, Value::Enum(2));
        let (e1, e3) = (started(&doc, children[1]).expect("moves"), started(&doc, children[3]).expect("moves"));
        assert!(e1 > e3, "from the end, the near-the-start child waits: {e1} vs {e3}");
    }

    #[test]
    fn solid_things_push_apart_in_depth_only_when_their_depths_overlap() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(5.0));
            put(&mut doc, layer, property::DEPTH, Value::F64(100.0));
            put(&mut doc, layer, property::POSITION, Value::Vec2([300.0, 300.0]));
        }
        put(&mut doc, b, property::POSITION_Z, Value::F64(400.0));
        let frame = doc.view().layout_frame(T).unwrap();
        assert!(frame.nudges.is_empty() && frame.nudges_z.is_empty(), "one behind the other with room between: nobody moves");
        put(&mut doc, b, property::POSITION_Z, Value::F64(60.0));
        let frame = doc.view().layout_frame(T).unwrap();
        let (za, zb) = (frame.nudges_z.get(&a).copied().unwrap_or(0.0), frame.nudges_z.get(&b).copied().unwrap_or(0.0));
        assert!(za < 0.0 && zb > 0.0, "overlapping in depth, they push apart along depth: {za} {zb}");
        assert!((zb - za - 50.0).abs() < 2.0, "just enough to clear depth + margins: {}", zb - za);
        assert!(frame.nudges.values().all(|d| d[0].abs() < 1e-3 && d[1].abs() < 1e-3), "straight behind each other: no sideways push");
        doc.apply(Intent::SetAttrs { layer: b, patch: LayerAttrsPatch { projection: Some(crate::doc::store::LayerProjection::TwoD), ..Default::default() } }).unwrap();
        let frame = doc.view().layout_frame(T).unwrap();
        assert!(frame.nudges_z.is_empty(), "a 2D thing has no depth to push along");
    }

    /// なぞる形の Margin は箱からの間合いで、押し合いの余白ではない(渋谷の窓で角の掴みが奥行きに押されて跳んだ)。
    #[test]
    fn a_trace_keeps_its_margin_to_its_box_and_is_not_pushed() {
        let mut doc = blank_project();
        let thing = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: thing, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
        put(&mut doc, thing, MARGIN, Value::F64(5.0));
        put(&mut doc, thing, property::POSITION, Value::Vec2([300.0, 300.0]));
        let trace = add(&mut doc, 2, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: trace, shapes: vec![rect_shape([255; 4], [10.0, 10.0])] }).unwrap();
        put(&mut doc, trace, CONNECT_FROM, Value::LayerId(thing.0));
        put(&mut doc, trace, TRACE, Value::Enum(2));
        put(&mut doc, trace, MARGIN, Value::F64(6.0));
        let frame = doc.view().layout_frame(T).unwrap();
        assert!(frame.nudges.is_empty() && frame.nudges_z.is_empty(), "the trace and its box overlap by design: nobody moves");
    }

    /// 押された跡(Trace = Push)と、押された量を読む文字(Readout = Push): 跡の箱は書いた場所、矢印は今の中心へ、文字は押された px。
    #[test]
    fn a_push_trace_shows_where_it_wanted_to_be_and_a_readout_reads_how_far() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for (layer, x) in [(a, 300.0), (b, 360.0)] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(5.0));
            put(&mut doc, layer, property::POSITION, Value::Vec2([x, 300.0]));
            doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(crate::doc::store::LayerProjection::TwoD), ..Default::default() } }).unwrap();
        }
        let trace = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: trace, shapes: vec![rect_shape([255; 4], [10.0, 10.0])] }).unwrap();
        put(&mut doc, trace, CONNECT_FROM, Value::LayerId(b.0));
        put(&mut doc, trace, TRACE, Value::Enum(7));
        let view = doc.view();
        let pushed = glam::Vec2::from(view.nudge(b, T).unwrap());
        assert!(pushed.x > 10.0, "b is pushed right, away from a: {pushed:?}");
        let (lo, hi) = view.box_seen_from(b, trace, T).unwrap().unwrap();
        let path = view.trace_path(trace, T).unwrap().unwrap();
        assert_eq!(path.len(), 3, "the wanted box, the shaft and the head");
        let ghost: Vec<glam::Vec2> = path[0].vertices.iter().map(|v| glam::vec2(v.point.x as f32, v.point.y as f32)).collect();
        assert!((ghost[0] - (lo - pushed)).length() < 0.01, "the wanted box is the box moved back by the push: {ghost:?} {lo:?}");
        let tip = path[1].vertices[1].point;
        assert!((glam::vec2(tip.x as f32, tip.y as f32) - (lo + hi) * 0.5).length() < 0.01, "the arrow ends at the centre of where it is");
        drop(view);

        let reader = add(&mut doc, 4, LayerSource::Text, None);
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: "# px".to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        doc.apply(Intent::SetTextDocument { layer: reader, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
        put(&mut doc, reader, READOUT, Value::Enum(1));
        put(&mut doc, reader, READOUT_OF, Value::LayerId(b.0));
        let text = doc.view().resolved_text_document(reader, T).unwrap().unwrap();
        assert_eq!(text.content.eval(T), format!("{} px", pushed.length().round() as i64), "the # becomes the push in px");
    }

    /// Overflow = Bounce: 箱の外へ書いた動きは内側へ鏡で折り返る(四角は軸ごと、円は中心を通る線の上)。中にいれば動かない。
    #[test]
    fn a_bouncing_box_folds_what_leaves_it_back_inside() {
        let m = CANVAS_MARGIN;
        assert_eq!(bounced([m + 10.0, m + 10.0], [m + 30.0, m + 30.0], [200.0, 100.0], 0.0), [0.0, 0.0], "inside: untouched");
        let right = bounced([m + 200.0, m + 10.0], [m + 220.0, m + 30.0], [200.0, 100.0], 0.0);
        assert!((right[0] + 40.0).abs() < 1e-3 && right[1] == 0.0, "20 px past the right wall comes back 20 px from it: {right:?}");
        let far = bounced([m + 180.0 + 360.0, m + 10.0], [m + 200.0 + 360.0, m + 30.0], [200.0, 100.0], 0.0);
        assert!((far[0] + 360.0).abs() < 1e-3, "one full round trip lands where it started: {far:?}");
        // 円: 半径 50 の中の直径 20 の物。中心から 60 外へ出た物は、同じ線の上を戻る。
        let out = bounced([m + 100.0 + 60.0 - 10.0, m + 50.0 - 10.0], [m + 100.0 + 60.0 + 10.0, m + 50.0 + 10.0], [200.0, 100.0], 50.0);
        assert!((out[0] + 40.0).abs() < 1e-3 && out[1].abs() < 1e-3, "room 40: 60 out folds to 20 out: {out:?}");
        let through = bounced([m + 100.0 + 120.0 - 10.0, m + 40.0], [m + 100.0 + 120.0 + 10.0, m + 60.0], [200.0, 100.0], 50.0);
        assert!((through[0] + 160.0).abs() < 1e-3, "80 past the wall crosses the centre and reaches the far wall: {through:?}");
    }

    /// 2 つ目の相手を指すと、相手の箱は 2 つの箱の重なり(離れていれば間)。
    #[test]
    fn a_label_with_two_anchors_sits_on_their_overlap_or_between_them() {
        let mut doc = blank_project();
        let square = |doc: &mut Document, id: u64, at: [f64; 2]| {
            let layer = add(doc, id, LayerSource::Shape, None);
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(doc, layer, property::POSITION, Value::Vec2(at));
            layer
        };
        let a = square(&mut doc, 1, [100.0, 100.0]);
        let b = square(&mut doc, 2, [100.0, 160.0]);
        let label = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: label, shapes: vec![rect_shape([255; 4], [20.0, 10.0])] }).unwrap();
        put(&mut doc, label, POSITION_ANCHOR, Value::LayerId(a.0));
        put(&mut doc, label, POSITION_ANCHOR_2, Value::LayerId(b.0));
        put(&mut doc, label, POSITION_AREA, Value::Enum(5));
        let (ba, bb) = (shown(&doc, a, T), shown(&doc, b, T));
        let bl2 = shown(&doc, label, T);
        let centre = |r: [f32; 4]| [(r[0] + r[2]) * 0.5, (r[1] + r[3]) * 0.5];
        let overlap = [ba[0].max(bb[0]), ba[1].max(bb[1]), ba[2].min(bb[2]), ba[3].min(bb[3])];
        assert!((centre(bl2)[0] - centre(overlap)[0]).abs() < 0.01 && (centre(bl2)[1] - centre(overlap)[1]).abs() < 0.01, "centred on the overlap: {bl2:?} {overlap:?}");
        put(&mut doc, b, property::POSITION, Value::Vec2([100.0, 300.0]));
        let (ba, bb, bl) = (shown(&doc, a, T), shown(&doc, b, T), shown(&doc, label, T));
        let gap_y = (ba[3] + bb[1]) * 0.5;
        assert!((centre(bl)[1] - gap_y).abs() < 0.01, "apart: centred in the gap between them: {bl:?} {ba:?} {bb:?}");
    }

    /// 配置はコマをまたいで覚えるが、書類を直せば古い覚えは読まない(版で捨てる)。仮の編集の view は覚えを使わない。
    #[test]
    fn the_layout_cache_forgets_on_edit_and_ignores_previews() {
        let mut doc = blank_project();
        let row = add(&mut doc, 1, LayerSource::Group, None);
        put(&mut doc, row, DISPLAY, Value::Enum(1));
        put(&mut doc, row, GAP, Value::F64(10.0));
        let a = add(&mut doc, 2, LayerSource::Shape, Some(row));
        let b = add(&mut doc, 3, LayerSource::Shape, Some(row));
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [40.0, 40.0])] }).unwrap();
        }
        let x = |doc: &Document| doc.view().layout_frame(T).unwrap().slots[&b].position[0];
        let first = x(&doc);
        assert_eq!(x(&doc), first, "a second view reads the same frame");
        put(&mut doc, row, GAP, Value::F64(50.0));
        assert!((x(&doc) - first - 40.0).abs() < 0.01, "the edit shows at once: {} → {}", first, x(&doc));
        let owner = doc.begin_preview();
        doc.preview_edits(owner, &[Intent::SetConstant { layer: row, property: PropertyId::new(GAP).unwrap(), value: Value::F64(0.0) }]).unwrap();
        assert!((x(&doc) - (first - 10.0)).abs() < 0.01, "a preview is laid out fresh, not from the cache: {}", x(&doc));
        doc.clear_preview_edits(owner);
        assert!((x(&doc) - first - 40.0).abs() < 0.01, "and after the preview the committed layout is back");
        doc.set_transient(row, PropertyId::new(GAP).unwrap(), Value::F64(0.0));
        let shown = doc.view();
        let transient = shown.layout_frame(T).unwrap();
        let committed = shown.clone().without_transients().layout_frame(T).unwrap();
        assert!((committed.slots[&b].position[0] - first - 40.0).abs() < 0.01,
            "excluding transients must not reuse the shown layout");
        assert!((transient.slots[&b].position[0] - (first - 10.0)).abs() < 0.01);
        assert_eq!(shown.layout_frame(T).unwrap(), transient,
            "the original view keeps its own evaluation inputs");
        let owner = doc.begin_preview();
        doc.preview_edits(owner, &[Intent::SetConstant { layer: row, property: PropertyId::new(GAP).unwrap(), value: Value::F64(25.0) }]).unwrap();
        let unchanged = doc.view().without_transients().layout_frame(T).unwrap();
        assert!(std::sync::Arc::ptr_eq(&committed, &unchanged),
            "a committed read must reuse its layout while excluded previews change");
    }

    #[test]
    fn a_label_anchored_to_a_thing_sits_on_the_side_it_names_and_follows() {
        let mut doc = blank_project();
        let thing = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: thing, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
        put(&mut doc, thing, property::POSITION, Value::Vec2([500.0, 300.0]));
        let label = add(&mut doc, 2, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: label, shapes: vec![rect_shape([255; 4], [40.0, 20.0])] }).unwrap();
        put(&mut doc, label, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, label, MARGIN, Value::F64(10.0));
        put(&mut doc, label, POSITION_ANCHOR, Value::LayerId(thing.0));
        put(&mut doc, label, POSITION_AREA, Value::Enum(2));
        let (a, b) = (shown(&doc, thing, T), shown(&doc, label, T));
        assert!((b[3] - (a[1] - 10.0)).abs() < 0.01, "Top: its bottom edge a margin above the thing's top: {a:?} {b:?}");
        assert!(((b[0] + b[2]) * 0.5 - (a[0] + a[2]) * 0.5).abs() < 0.01, "centred along the thing");
        put(&mut doc, label, POSITION_AREA, Value::Enum(6));
        let b = shown(&doc, label, T);
        assert!((b[0] - (a[2] + 10.0)).abs() < 0.01 && ((b[1] + b[3]) * 0.5 - (a[1] + a[3]) * 0.5).abs() < 0.01, "Right: beside it, middles level: {b:?}");
        put(&mut doc, thing, property::POSITION, Value::Vec2([800.0, 600.0]));
        let (a, b) = (shown(&doc, thing, T), shown(&doc, label, T));
        assert!((b[0] - (a[2] + 10.0)).abs() < 0.01, "moving the thing carries the label");
        let placed = shown(&doc, label, T);
        put(&mut doc, label, POSITION_AREA, Value::Enum(0));
        let back = shown(&doc, label, T);
        assert!(back != placed && (back[0] + back[2]) * 0.5 < 100.0, "None: back where it was written: {back:?}");
    }

    #[test]
    fn a_free_child_snaps_to_the_nearest_field_of_its_grid() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(2)), (GRID_COLUMNS, Value::F64(4.0)), (GRID_ROWS, Value::F64(4.0)), (GAP, Value::F64(20.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(860.0)), (HEIGHT, Value::F64(860.0))] {
            put(&mut doc, grid, name, value);
        }
        put(&mut doc, grid, property::POSITION, Value::Vec2([100.0, 50.0]));
        let card = rect(&mut doc, 2, grid, [150.0, 120.0]);
        put(&mut doc, card, POSITION_TYPE, Value::Enum(1));
        put(&mut doc, card, property::POSITION, Value::Vec2([250.0, 470.0]));
        let free = shown(&doc, card, T);
        put(&mut doc, card, SNAP_TO_GRID, Value::F64(1.0));
        let snapped = shown(&doc, card, T);
        // 升目は 200 px + 溝 20 px: 画面の上で列は Group の Position から 220 px ごとに始まる。
        let starts_x: Vec<f32> = (0..4).map(|i| 100.0 + i as f32 * 220.0).collect();
        let starts_y: Vec<f32> = (0..4).map(|i| 50.0 + i as f32 * 220.0).collect();
        let near = |v: f32, list: &[f32]| list.iter().cloned().min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs())).unwrap();
        assert!((snapped[0] - near(free[0], &starts_x)).abs() < 0.5 && (snapped[1] - near(free[1], &starts_y)).abs() < 0.5,
            "the box's corner lands on the nearest field's corner: free {free:?} snapped {snapped:?}");
        assert!(((snapped[2] - snapped[0]) - (free[2] - free[0])).abs() < 0.5, "Size Off keeps the size");

        put(&mut doc, card, SNAP_SIZE, Value::Enum(1));
        let fitted = shown(&doc, card, T);
        assert!(((fitted[2] - fitted[0]) - 200.0).abs() < 1.0 && ((fitted[3] - fitted[1]) - 200.0).abs() < 1.0, "Size Fields: one whole field: {fitted:?}");

        put(&mut doc, card, SNAP_SIZE, Value::Enum(0));
        put(&mut doc, card, SNAP_TO_GRID, Value::F64(0.5));
        let half = shown(&doc, card, T);
        assert!((half[0] - (free[0] + snapped[0]) * 0.5).abs() < 0.5, "half strength goes half way");
    }

    #[test]
    fn a_transform_origin_keeps_its_corner_on_the_position_as_the_box_grows() {
        let mut doc = blank_project();
        let card = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: card, shapes: vec![rect_shape([255; 4], [100.0, 60.0])] }).unwrap();
        put(&mut doc, card, property::POSITION, Value::Vec2([400.0, 300.0]));
        put(&mut doc, card, TRANSFORM_ORIGIN, Value::Enum(7));
        let b = shown(&doc, card, T);
        assert!((b[0] - 400.0).abs() <= 1.01 && (b[3] - 300.0).abs() <= 1.01, "Bottom Left: that corner sits on the Position: {b:?}");
        put(&mut doc, card, property::SHAPE_SIZE, Value::Vec2([300.0, 200.0]));
        let grown = shown(&doc, card, T);
        assert!((grown[0] - 400.0).abs() <= 1.01 && (grown[3] - 300.0).abs() <= 1.01, "the box grows up and to the right, the corner stays: {grown:?}");
        put(&mut doc, card, property::SCALE, Value::Vec2([2.0, 2.0]));
        let scaled = shown(&doc, card, T);
        assert!((scaled[0] - 400.0).abs() <= 1.01 && (scaled[3] - 300.0).abs() <= 1.01 && (scaled[2] - scaled[0] - 600.0).abs() < 1.0, "and scaling grows from that corner: {scaled:?}");
        put(&mut doc, card, TRANSFORM_ORIGIN, Value::Enum(5));
        let centred = shown(&doc, card, T);
        assert!(((centred[0] + centred[2]) * 0.5 - 400.0).abs() <= 1.01, "Center: the middle sits on the Position: {centred:?}");
    }

    #[test]
    fn free_children_follow_the_parents_edges_by_their_constraints() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let panel = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (HEIGHT, Value::F64(300.0))] {
            put(&mut doc, panel, name, value);
        }
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::F64(400.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::F64(600.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: panel, property: PropertyId::new(WIDTH).unwrap(), track }).unwrap();
        let badge = rect(&mut doc, 2, panel, [40.0, 40.0]);
        let bar = rect(&mut doc, 3, panel, [300.0, 20.0]);
        for (layer, position, mode) in [(badge, [360.0, 40.0], 1), (bar, [200.0, 280.0], 2)] {
            put(&mut doc, layer, POSITION_TYPE, Value::Enum(1));
            put(&mut doc, layer, property::POSITION, Value::Vec2(position));
            put(&mut doc, layer, HORIZONTAL_CONSTRAINT, Value::Enum(mode));
        }
        let (b0, b30) = (shown(&doc, badge, at(0)), shown(&doc, badge, at(30)));
        assert!(((b30[0] - b0[0]) - 200.0).abs() < 0.5 && ((b30[2] - b30[0]) - (b0[2] - b0[0])).abs() < 0.5, "Right: keeps its distance from the right edge: {b0:?} {b30:?}");
        let (r0, r30) = (shown(&doc, bar, at(0)), shown(&doc, bar, at(30)));
        assert!((r30[0] - r0[0]).abs() < 0.5 && ((r30[2] - r0[2]) - 200.0).abs() < 0.5, "Left & Right: both distances kept, it stretches: {r0:?} {r30:?}");
        put(&mut doc, badge, HORIZONTAL_CONSTRAINT, Value::Enum(3));
        let c30 = shown(&doc, badge, at(30));
        assert!(((c30[0] - b0[0]) - 100.0).abs() < 0.5, "Center: moves half as much: {c30:?}");
    }

    #[test]
    fn an_object_travels_the_border_box_of_its_parent_and_turns_with_it() {
        let mut doc = blank_project();
        let card = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(400.0)), (HEIGHT, Value::F64(200.0))] {
            put(&mut doc, card, name, value);
        }
        let dot = rect(&mut doc, 2, card, [10.0, 10.0]);
        put(&mut doc, dot, POSITION_TYPE, Value::Enum(1));
        put(&mut doc, dot, OFFSET_PATH, Value::Enum(1));
        let at = |doc: &mut Document, percent: f64| {
            put(doc, dot, OFFSET_DISTANCE, Value::F64(percent));
            let view = doc.view();
            (view.on_offset_path(dot, T).unwrap().unwrap(), view.offset_rotation(dot, T).unwrap())
        };
        let ((p, _), turn) = at(&mut doc, 25.0);
        assert!((p[0] - (1.0 + 300.0)).abs() < 0.01 && (p[1] - 1.0).abs() < 0.01 && turn.abs() < 0.01, "a quarter of 1200 px is 300 px along the top: {p:?}");
        let ((p, _), turn) = at(&mut doc, 40.0);
        assert!((p[0] - 401.0).abs() < 0.01 && (p[1] - (1.0 + 80.0)).abs() < 0.01 && (turn - 90.0).abs() < 0.01, "down the right side, turned to face down: {p:?} {turn}");
        let ((p, _), _) = at(&mut doc, 125.0);
        assert!((p[0] - 301.0).abs() < 0.01, "past 100% it goes round again");
        put(&mut doc, card, BORDER_RADIUS, Value::F64(50.0));
        let ((p, _), turn) = at(&mut doc, 0.0);
        assert!((p[0] - 51.0).abs() < 0.01 && turn.abs() < 0.01, "with round corners the path starts after the corner");
        // 角を曲がる間も滑らかに(1% ずつで大きく跳ばない)。
        let mut previous: Option<[f32; 2]> = None;
        for k in 0..=100 {
            let ((p, _), _) = at(&mut doc, k as f64);
            if let Some(q) = previous {
                assert!(((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt() < 12.0, "no jump around the corners at {k}%");
            }
            previous = Some(p);
        }
        let shown_at = shown(&doc, dot, T);
        assert!(shown_at[0] < 60.0, "the object is drawn where the path puts it: {shown_at:?}");
    }

    #[test]
    fn a_field_box_swells_what_is_near_it_and_leaves_the_far_alone() {
        let mut doc = blank_project();
        let lens = add(&mut doc, 1, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: lens, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
        put(&mut doc, lens, property::POSITION, Value::Vec2([500.0, 300.0]));
        let near = add(&mut doc, 2, LayerSource::Shape, None);
        let far = add(&mut doc, 3, LayerSource::Shape, None);
        for (layer, at) in [(near, [540.0, 300.0]), (far, [1200.0, 300.0])] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [20.0, 20.0])] }).unwrap();
            put(&mut doc, layer, property::POSITION, Value::Vec2(at));
            put(&mut doc, layer, FIELD, Value::LayerId(lens.0));
            put(&mut doc, layer, FIELD_SCALE, Value::F64(3.0));
            put(&mut doc, layer, FIELD_FALLOFF, Value::F64(150.0));
        }
        let (n, f) = (shown(&doc, near, T), shown(&doc, far, T));
        assert!(((n[2] - n[0]) - 60.0).abs() < 1.0, "inside the field box: full strength, three times the size: {n:?}");
        assert!(((f[2] - f[0]) - 20.0).abs() < 0.5, "beyond the falloff: untouched: {f:?}");
        put(&mut doc, near, property::POSITION, Value::Vec2([625.0, 300.0]));
        let half = shown(&doc, near, T);
        assert!((half[2] - half[0]) > 21.0 && (half[2] - half[0]) < 59.0, "in the falloff: part way: {half:?}");
        put(&mut doc, near, FIELD_SCALE, Value::F64(1.0));
        put(&mut doc, near, property::POSITION, Value::Vec2([540.0, 300.0]));
        let still = shown(&doc, near, T);
        put(&mut doc, near, FIELD_PUSH, Value::F64(40.0));
        let pushed = shown(&doc, near, T);
        let lens_box = shown(&doc, lens, T);
        let away = glam::vec2((still[0] + still[2]) * 0.5 - (lens_box[0] + lens_box[2]) * 0.5, (still[1] + still[3]) * 0.5 - (lens_box[1] + lens_box[3]) * 0.5).normalize();
        let moved = glam::vec2((pushed[0] + pushed[2] - still[0] - still[2]) * 0.5, (pushed[1] + pushed[3] - still[1] - still[3]) * 0.5);
        assert!((moved - away * 40.0).length() < 1.5, "pushed 40 px away from the field's centre: {moved:?} along {away:?}");
    }

    #[test]
    fn a_picture_takes_the_size_the_host_measured() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let picture = add(&mut doc, 2, LayerSource::File { path: "/tmp/photo.png".to_owned(), fingerprint: None }, Some(group));
        let after = rect(&mut doc, 3, group, [40.0, 40.0]);
        let mut inputs = crate::doc::store::analysis::AnalysisInputs::default();
        inputs.set_extent("/tmp/photo.png", [320.0, 180.0, 0.0]);
        let view = doc.view().with_analysis(&inputs);
        let frame = view.layout_frame(T).unwrap();
        assert_eq!(view.layer_box(picture, T).unwrap(), Some([0.0, 0.0, 320.0, 180.0]));
        // 箱の左端 = 位置 - 中心 + 箱の左(形の素材座標は輪郭の canvas の 1 画素外が原点、画は 0)。
        let left = |id: LayerId, min: f32| frame.slots[&id].position[0] - frame.slots[&id].anchor[0] + min;
        assert_eq!(left(after, 1.0), left(picture, 0.0) + 320.0 + 10.0, "the next item starts after the picture and the gap");
    }

    #[test]
    fn wrapping_text_flows_around_a_sibling_that_declares_its_shape() {
        let mut doc = blank_project();
        let words = add(&mut doc, 2, LayerSource::Text, None);
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: "Shibuya crossing at nine in the evening, the lights change and three thousand people walk at once across the white lines".to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 40.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        let comp = doc.view().composition().unwrap().unwrap();
        doc.apply(Intent::SetTextDocument { layer: words, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: Some([600.0, comp.height as f32]), styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
        let object = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: object, shapes: vec![rect_shape([255; 4], [160.0, 160.0])] }).unwrap();
        let middle = doc.view().world_2d(words, T).unwrap().transform_point2(glam::vec2(300.0, comp.height as f32 * 0.5));
        put(&mut doc, object, property::POSITION, Value::Vec2([middle.x as f64, middle.y as f64]));
        let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        // 物の占める範囲(文字の枠の座標)と重なる字の数。
        let overlapping = |doc: &Document| {
            let view = doc.view();
            let resolved = view.resolved_layers(T).unwrap();
            let around = resolved.iter().find(|l| l.id == words).unwrap().flow_around.clone().expect("the object is declared");
            let (lo, hi) = around.iter().flat_map(|o| o.points.iter()).fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(glam::Vec2::from(*p)), hi.max(glam::Vec2::from(*p))));
            let document = view.resolved_text_document(words, T).unwrap().unwrap();
            let count = |shaped: &crate::doc::vector::text::ShapedText| shaped.contours.iter().filter(|c| {
                let (a, b) = c.vertices.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(a, b), v| (a.min(glam::vec2(v.point.x as f32, v.point.y as f32)), b.max(glam::vec2(v.point.x as f32, v.point.y as f32))));
                a.x < hi.x && b.x > lo.x && a.y < hi.y && b.y > lo.y
            }).count();
            let plain = crate::doc::store::text_frame::shape_document(&document, T, &canvas).unwrap().unwrap();
            let flowed = crate::doc::store::text_frame::shape_document_around(&document, T, &canvas, &around).unwrap().unwrap();
            let right_of = flowed.contours.iter().any(|c| c.vertices.iter().all(|v| v.point.x as f32 > hi.x && (v.point.y as f32) > lo.y && (v.point.y as f32) < hi.y));
            (count(&plain), count(&flowed), plain.contours.len() == flowed.contours.len(), right_of)
        };
        put(&mut doc, object, SHAPE_OUTSIDE, Value::Enum(2));
        assert!(doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == words).unwrap().flow_around.is_some());
        let (before, after, all_glyphs, both_sides) = overlapping(&doc);
        assert!(before > 0, "without flowing, the words run under the object");
        assert_eq!(after, 0, "the words flow around it");
        assert!(all_glyphs, "no word is lost");
        assert!(both_sides, "lines continue on the other side of the object (wrap-flow: both)");

        put(&mut doc, object, SHAPE_OUTSIDE, Value::Enum(0));
        assert!(doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == words).unwrap().flow_around.is_none(), "None declares nothing");
    }

    #[test]
    fn a_growing_line_of_text_pushes_its_neighbour() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let name = add(&mut doc, 2, LayerSource::Text, Some(group));
        let icon = rect(&mut doc, 3, group, [40.0, 40.0]);
        let x = |doc: &mut Document, content: &str| {
            let mut track = ContentTrack::new();
            track.insert(ContentKeyframe { t: T, content: content.to_owned() });
            let style = TextDocumentStyle {
                id: TextStyleId(0),
                font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
                size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
            };
            doc.apply(Intent::SetTextDocument { layer: name, document: TextDocument {
                content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
            } }).unwrap();
            shown(doc, icon, T)[0]
        };
        let short = x(&mut doc, "Taro");
        let long = x(&mut doc, "Taro Yamada");
        assert!(long > short + 50.0, "the icon moves right as the name grows: {short} → {long}");
        let text = shown(&doc, name, T);
        assert_eq!((text[0], text[1]), (20.0, 8.0), "the line box sits at the padding");
    }
}



/// CSS の timing function(区間の形)。Ease / Linear / Ease In / Ease Out / Ease In Out。
fn ease(kind: i64, u: f64) -> f64 {
    let (x1, y1, x2, y2) = match kind {
        1 => return u.clamp(0.0, 1.0),
        2 => (0.42, 0.0, 1.0, 1.0),
        3 => (0.0, 0.0, 0.58, 1.0),
        4 => (0.42, 0.0, 0.58, 1.0),
        _ => (0.25, 0.1, 0.25, 1.0),
    };
    let bezier = |a: f64, b: f64, s: f64| 3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s * s * (1.0 - s) + s.powi(3);
    let u = u.clamp(0.0, 1.0);
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let mid = (lo + hi) * 0.5;
        if bezier(x1, x2, mid) < u { lo = mid } else { hi = mid }
    }
    bezier(y1, y2, (lo + hi) * 0.5)
}
