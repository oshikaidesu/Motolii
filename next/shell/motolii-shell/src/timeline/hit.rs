//! `Hit` 型と当たり判定(`hit_test`)。click/drag した先が bar か空白部かだけを
//! 判定する純粋な読み取り。

use iced::Point;

use motolii_store::LayerId;

use super::projection::{frame_to_x, RowProjection};

/// click/drag した先が何か。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// この layer の bar の上。
    Bar(LayerId),
    /// ルーラーまたは行の空白部(scrub の対象)。
    Blank,
}

/// `point` がどこに当たったか。ルーラー帯(`ruler_height` 未満)は常に [`Hit::Blank`]。
/// 行の内側では bar の区間(`start..start+duration`)だけを [`Hit::Bar`] とし、
/// それ以外はその行の空白部として [`Hit::Blank`] を返す。
pub fn hit_test(
    point: Point,
    rows: &[RowProjection],
    ruler_height: f32,
    row_height: f32,
    width: f32,
    duration_frames: i64,
) -> Hit {
    if point.y < ruler_height || row_height <= 0.0 {
        return Hit::Blank;
    }
    let row_index = ((point.y - ruler_height) / row_height).floor();
    if row_index < 0.0 {
        return Hit::Blank;
    }
    let Some(row) = rows.get(row_index as usize) else {
        return Hit::Blank;
    };
    let start_x = frame_to_x(row.start, width, duration_frames);
    let end_x = frame_to_x(row.start + row.duration, width, duration_frames).max(start_x + 1.0);
    if point.x >= start_x && point.x < end_x {
        Hit::Bar(row.id)
    } else {
        Hit::Blank
    }
}
