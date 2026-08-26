//! Stage 入力の調停 — **3状態**(素通し / orbit 中 / 掴み中)。
//!
//! [Stage対話の概念地図](../../../docs/reviews/2026-08-18-stage-interaction-concept-map.md)
//! §2「入力の所有権」の実体である: 入力ブリッジは Rerun へ渡す**前**に入力を
//! 握っており、**掴み中は orbit へ流さない**調停をここで構造的に行う。
//!
//! ## 3状態の意味
//!
//! - [`StagePointerOwner::PassThrough`](素通し) — 誰も掴んでいない。hover・ホイールは
//!   Rerun へ流れる(picking / ズームが生きる)
//! - [`StagePointerOwner::Orbiting`](orbit 中) — Rerun 側のカメラ操作がボタンを
//!   握っている。移動・離しは**流れ続ける**(枠際で orbit が止まらない)
//! - [`StagePointerOwner::Grabbing`](掴み中) — 文書側の掴み(将来のギズモ)が
//!   握っている。**Rerun へは1手も流さない**
//!
//! ## 掴み判定は seam だけ(M-2)
//!
//! ギズモ本体は M-2 の NON-GOAL である。ここに在るのは「掴みと判定される領域」を
//! 差し込む口([`GrabRegion`])だけで、製品の M-2 は `None`(掴みは発生しない)。
//! テストがダミー領域を差して3状態の切り替えを審判する。M-2 後、ギズモの
//! hit-test がこの口の実装になる(概念地図 §3 の柵: ギズモを Rerun fork に
//! 実装しない — 調停は fork の外、つまりこの module に居る)。

use motolii_ui::rerun_stage::{PointerPhase, StagePointerButton};

use crate::stage_bridge::StageInput;

/// 掴みと判定される領域(widget 左上原点の論理座標)。
///
/// M-2 ではテスト専用のダミー。製品は `None` を渡す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrabRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl GrabRegion {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// いま pointer を握っている者。widget の `Program::State` として木に住む。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StagePointerOwner {
    /// 素通し — 誰も掴んでいない。
    #[default]
    PassThrough,
    /// orbit 中 — Rerun がこのボタンで握っている。
    Orbiting { button: StagePointerButton },
    /// 掴み中 — 文書側(ギズモの席)がこのボタンで握っている。
    Grabbing { button: StagePointerButton },
}

/// 1件の行き先。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Rerun へ流す。
    Forward,
    /// 流さない(掴みの席が消費した)。
    Swallow,
}

/// **調停の本体**。1件を裁いて状態を進める。
///
/// 規則(全て単体テストが審判する):
///
/// - `Down`: 素通し中に掴み領域の中なら **掴み中へ**(流さない)。外なら
///   **orbit 中へ**(流す)。既にどちらかの最中なら、後から来たボタンは
///   いまの所有者に従う(状態は変えない)
/// - `Move`: 掴み中だけ流さない
/// - `Up`: 所有者のボタンなら素通しへ戻る。行き先は所有者に従う
///   (orbit 中は流す = Rerun がドラッグ終了を知る。掴み中は流さない)
/// - `Cancel`(窓から出た): 状態を素通しへ戻す。掴み中だけ流さない
/// - `Scroll`: 掴み中だけ流さない(ズームは素通し・orbit 中に生きる)
/// - `Modifiers`: 常に流す(状態を持たない帳簿の更新)
pub fn route(
    owner: &mut StagePointerOwner,
    input: &StageInput,
    grab: Option<&GrabRegion>,
) -> Route {
    // いまの所有者に従う行き先。状態遷移の後で使うものではない点に注意
    // (`Up` は「所有者に従って流してから」素通しへ戻る)。
    let follow_owner = |owner: &StagePointerOwner| match owner {
        StagePointerOwner::Grabbing { .. } => Route::Swallow,
        StagePointerOwner::PassThrough | StagePointerOwner::Orbiting { .. } => Route::Forward,
    };

    match input {
        StageInput::Modifiers(_) => Route::Forward,
        StageInput::Scroll { .. } => follow_owner(owner),
        StageInput::Pointer {
            phase: PointerPhase::Move,
            ..
        } => follow_owner(owner),
        StageInput::Pointer {
            phase: PointerPhase::Down,
            button,
            x,
            y,
        } => match *owner {
            StagePointerOwner::PassThrough => {
                if grab.is_some_and(|region| region.contains(*x, *y)) {
                    *owner = StagePointerOwner::Grabbing { button: *button };
                    Route::Swallow
                } else {
                    *owner = StagePointerOwner::Orbiting { button: *button };
                    Route::Forward
                }
            }
            // 後から来たボタンはいまの所有者に従う(状態は変えない)。
            StagePointerOwner::Orbiting { .. } => Route::Forward,
            StagePointerOwner::Grabbing { .. } => Route::Swallow,
        },
        StageInput::Pointer {
            phase: PointerPhase::Up,
            button,
            ..
        } => {
            let route = follow_owner(owner);
            match *owner {
                StagePointerOwner::Orbiting { button: held } if held == *button => {
                    *owner = StagePointerOwner::PassThrough;
                }
                StagePointerOwner::Grabbing { button: held } if held == *button => {
                    *owner = StagePointerOwner::PassThrough;
                }
                _ => {}
            }
            route
        }
        StageInput::Pointer {
            phase: PointerPhase::Cancel,
            ..
        } => {
            let route = follow_owner(owner);
            *owner = StagePointerOwner::PassThrough;
            route
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn down(x: f32, y: f32) -> StageInput {
        StageInput::Pointer {
            phase: PointerPhase::Down,
            button: StagePointerButton::Primary,
            x,
            y,
        }
    }

    fn moved(x: f32, y: f32) -> StageInput {
        StageInput::Pointer {
            phase: PointerPhase::Move,
            button: StagePointerButton::Primary,
            x,
            y,
        }
    }

    fn up(x: f32, y: f32) -> StageInput {
        StageInput::Pointer {
            phase: PointerPhase::Up,
            button: StagePointerButton::Primary,
            x,
            y,
        }
    }

    fn scroll() -> StageInput {
        StageInput::Scroll {
            delta_x: 0.0,
            delta_y: 3.0,
            x: 50.0,
            y: 50.0,
        }
    }

    const GRAB: GrabRegion = GrabRegion {
        x: 10.0,
        y: 10.0,
        width: 20.0,
        height: 20.0,
    };

    /// 素通し: hover もホイールも Rerun へ流れる(picking / ズームが生きる)。
    #[test]
    fn pass_through_forwards_hover_and_wheel() {
        let mut owner = StagePointerOwner::PassThrough;
        assert_eq!(
            route(&mut owner, &moved(50.0, 50.0), Some(&GRAB)),
            Route::Forward
        );
        assert_eq!(route(&mut owner, &scroll(), Some(&GRAB)), Route::Forward);
        assert_eq!(
            owner,
            StagePointerOwner::PassThrough,
            "hover では所有は動かない"
        );
    }

    /// 掴み領域の外で押す → orbit 中。ドラッグ一式が Rerun へ流れ、離しで素通しへ戻る。
    #[test]
    fn a_press_outside_the_grab_region_starts_an_orbit_that_rerun_sees() {
        let mut owner = StagePointerOwner::PassThrough;
        assert_eq!(
            route(&mut owner, &down(50.0, 50.0), Some(&GRAB)),
            Route::Forward
        );
        assert_eq!(
            owner,
            StagePointerOwner::Orbiting {
                button: StagePointerButton::Primary
            },
            "掴み領域の外の押下で orbit 中になる"
        );
        assert_eq!(
            route(&mut owner, &moved(60.0, 55.0), Some(&GRAB)),
            Route::Forward
        );
        // 枠際でも orbit は止まらない: 掴み領域の**上**を通っても流れ続ける。
        assert_eq!(
            route(&mut owner, &moved(15.0, 15.0), Some(&GRAB)),
            Route::Forward
        );
        assert_eq!(
            route(&mut owner, &up(15.0, 15.0), Some(&GRAB)),
            Route::Forward
        );
        assert_eq!(owner, StagePointerOwner::PassThrough, "離しで素通しへ戻る");
    }

    /// 掴み領域の中で押す → 掴み中。**Rerun へは1手も流れない**。
    #[test]
    fn a_press_inside_the_grab_region_starves_rerun_of_the_whole_gesture() {
        let mut owner = StagePointerOwner::PassThrough;
        assert_eq!(
            route(&mut owner, &down(15.0, 15.0), Some(&GRAB)),
            Route::Swallow
        );
        assert_eq!(
            owner,
            StagePointerOwner::Grabbing {
                button: StagePointerButton::Primary
            },
            "掴み領域の中の押下で掴み中になる"
        );
        // 領域の外へ出ても掴みは続き、Rerun へは流れない。
        assert_eq!(
            route(&mut owner, &moved(80.0, 80.0), Some(&GRAB)),
            Route::Swallow
        );
        assert_eq!(route(&mut owner, &scroll(), Some(&GRAB)), Route::Swallow);
        assert_eq!(
            route(&mut owner, &up(80.0, 80.0), Some(&GRAB)),
            Route::Swallow
        );
        assert_eq!(owner, StagePointerOwner::PassThrough, "離しで素通しへ戻る");
    }

    /// 掴み領域が無い(= 製品の M-2)なら、掴み中は決して起きない。
    #[test]
    fn without_a_grab_region_grabbing_never_happens() {
        let mut owner = StagePointerOwner::PassThrough;
        assert_eq!(route(&mut owner, &down(15.0, 15.0), None), Route::Forward);
        assert_eq!(
            owner,
            StagePointerOwner::Orbiting {
                button: StagePointerButton::Primary
            }
        );
    }

    /// orbit の最中に掴み領域の中で押しても、掴みへは**移らない**
    /// (後から来たボタンはいまの所有者に従う)。
    #[test]
    fn a_second_button_follows_the_current_owner() {
        let mut owner = StagePointerOwner::PassThrough;
        assert_eq!(
            route(&mut owner, &down(50.0, 50.0), Some(&GRAB)),
            Route::Forward
        );
        let second = StageInput::Pointer {
            phase: PointerPhase::Down,
            button: StagePointerButton::Secondary,
            x: 15.0,
            y: 15.0,
        };
        assert_eq!(route(&mut owner, &second, Some(&GRAB)), Route::Forward);
        assert_eq!(
            owner,
            StagePointerOwner::Orbiting {
                button: StagePointerButton::Primary
            },
            "所有者は最初のボタンのまま"
        );
        // 所有者でないボタンの離しでは素通しへ戻らない。
        let second_up = StageInput::Pointer {
            phase: PointerPhase::Up,
            button: StagePointerButton::Secondary,
            x: 15.0,
            y: 15.0,
        };
        assert_eq!(route(&mut owner, &second_up, Some(&GRAB)), Route::Forward);
        assert_eq!(
            owner,
            StagePointerOwner::Orbiting {
                button: StagePointerButton::Primary
            }
        );
    }

    /// 窓から出たら(Cancel)所有は素通しへ戻る。掴み中だけ流さない。
    #[test]
    fn leaving_the_window_resets_the_owner() {
        let cancel = StageInput::Pointer {
            phase: PointerPhase::Cancel,
            button: StagePointerButton::Primary,
            x: f32::NAN,
            y: f32::NAN,
        };

        let mut owner = StagePointerOwner::Orbiting {
            button: StagePointerButton::Primary,
        };
        assert_eq!(route(&mut owner, &cancel, Some(&GRAB)), Route::Forward);
        assert_eq!(owner, StagePointerOwner::PassThrough);

        let mut owner = StagePointerOwner::Grabbing {
            button: StagePointerButton::Primary,
        };
        assert_eq!(route(&mut owner, &cancel, Some(&GRAB)), Route::Swallow);
        assert_eq!(owner, StagePointerOwner::PassThrough);
    }

    /// modifiers は帳簿なので常に流れる(掴み中でも stage 側の帳簿は最新に保つ)。
    #[test]
    fn modifiers_always_flow() {
        let mut owner = StagePointerOwner::Grabbing {
            button: StagePointerButton::Primary,
        };
        assert_eq!(
            route(&mut owner, &StageInput::Modifiers(9), Some(&GRAB)),
            Route::Forward
        );
        assert_eq!(
            owner,
            StagePointerOwner::Grabbing {
                button: StagePointerButton::Primary
            }
        );
    }
}
