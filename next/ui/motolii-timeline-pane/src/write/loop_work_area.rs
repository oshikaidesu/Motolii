//! Timeline のループ帯・作業範囲(work area)のドラッグと確定(SP-2 分割、
//! `write.rs` 752-834行を移設)。**中身は無改変** — `impl PaneState` を
//! そのままここへ移した(private field アクセスは子モジュールから見える
//! Rust の可視性のまま)。

use super::*;
use super::misc::comp_duration;

impl PaneState {
    // ---- 作業範囲/ループ帯(B21+B18 第1切片、正典 §5) ----

    /// ループ on/off(map 1082/1083)。帯が無い時は理由つき拒否(M13: 無反応
    /// ゼロ — 何も起きないトグルを黙って飲み込まない)。
    pub(crate) fn toggle_loop(&mut self) -> Option<String> {
        if self.work_area.is_none() {
            return Some(
                "作業範囲が無いのでループできない — ルーラー上端の帯をドラッグして範囲を引く".into(),
            );
        }
        self.loop_enabled = !self.loop_enabled;
        None
    }

    /// ループ帯を掴んだ瞬間(正典 §5: 空白=新規・端=リサイズ・中=平行移動)。
    /// **新規は引いたら即 on**(正典 §5 — 別キーで有効化させない)。リサイズ/
    /// 移動は on/off を変えない。
    pub(crate) fn start_loop_drag(&mut self, doc: &Document, part: LoopBandPart, at_frame: i64) {
        if self.loop_drag.is_some() {
            return; // 既に別のドラッグが進行中 — 多重起動しない(clip と同型)。
        }
        let duration = comp_duration(doc);
        let origin_area = self.work_area;
        let origin_enabled = self.loop_enabled;
        let kind = match (part, self.work_area) {
            (LoopBandPart::EdgeIn, Some(area)) => LoopDragKind::Span { anchor: area.end },
            (LoopBandPart::EdgeOut, Some(area)) => LoopDragKind::Span { anchor: area.start },
            (LoopBandPart::Body, Some(area)) => {
                LoopDragKind::Move { origin: area, grab_at_frame: at_frame }
            }
            // 空白=新規(帯が無い時の Edge*/Body は classify が返さないが、
            // 万一来ても新規へ倒す — 安全側)。
            _ => {
                self.loop_enabled = true; // 引いたら即 on(正典 §5)。
                self.work_area = Some(work_area::dragged_area(at_frame, at_frame, duration));
                LoopDragKind::Span { anchor: at_frame }
            }
        };
        self.loop_drag = Some(LoopDragState { kind, origin_area, origin_enabled });
    }

    /// ループ帯ドラッグ中のポインタ移動。掴んだ瞬間の anchor/origin を基準に
    /// **絶対値で出し直す**(delta 蓄積禁止 — 正典 §2 の思想)。
    pub(crate) fn continue_loop_drag(&mut self, doc: &Document, at_frame: i64) {
        let Some(drag) = self.loop_drag else {
            return;
        };
        let duration = comp_duration(doc);
        self.work_area = Some(match drag.kind {
            LoopDragKind::Span { anchor } => work_area::dragged_area(anchor, at_frame, duration),
            LoopDragKind::Move { origin, grab_at_frame } => {
                work_area::moved_area(origin, grab_at_frame, at_frame, duration)
            }
        });
    }

    /// Mark Clip / Mark Selection(map 724/727): 選択 layer の clip 範囲
    /// (複数選択は合併区間)を作業範囲へ。選択が無ければ理由つき拒否(M13)。
    pub(crate) fn set_work_area_to_selection(&mut self, doc: &Document, session: &Session) -> Option<String> {
        let targets: Vec<LayerId> = if session.selected_layers.is_empty() {
            session.selection.into_iter().collect()
        } else {
            session.selected_layers.clone()
        };
        if targets.is_empty() {
            return Some("選択が無いので作業範囲にできない — layer を選んでから".into());
        }
        let rows = rows(&doc.view(), session);
        let spans: Vec<(i64, i64)> = rows
            .iter()
            .filter(|row| targets.contains(&row.id))
            .map(|row| (row.start, row.start + row.duration))
            .collect();
        let (Some(start), Some(end)) = (
            spans.iter().map(|(s, _)| *s).min(),
            spans.iter().map(|(_, e)| *e).max(),
        ) else {
            return Some("選択 layer が見当たらないので作業範囲にできない".into());
        };
        // clip 範囲を dragged_area(clamp・最短1フレーム)へ通して不変量を守る。
        self.work_area = Some(work_area::dragged_area(start, end, comp_duration(doc)));
        None
    }
}
