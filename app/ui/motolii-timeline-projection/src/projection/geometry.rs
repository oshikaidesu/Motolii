//! comp フレーム ⇄ x px の変換とルーラー目盛り/明暗帯の刻み(SP-2 分割、
//! `projection.rs` 346-618行を移設)。**中身は無改変**。

use super::*;

/// comp フレーム → x px。`duration_frames <= 0` の空 comp では常に 0。
///
/// `pub`: screenshot 器具(`motolii_shell::screenshot`)が Timeline canvas と
/// 同じ x 座標計算を使うため(マーカー・bar の位置を2箇所で別の式にしない)。
/// **裁定160 切片7で `pub(crate)` → `pub` に緩めた** — screenshot.rs は
/// crate 分割後 `motolii-shell` 側に残るので、`pub(crate)`(同一クレート限定)
/// のままでは呼べなくなる。
pub fn frame_to_x(frame: i64, width: f32, duration_frames: i64) -> f32 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0.0;
    }
    let ratio = frame as f32 / duration_frames as f32;
    (ratio * width).clamp(0.0, width)
}

/// x px → comp フレーム。**scrub の core**(canvas の click/drag と単体 test の両方が
/// これを呼ぶ)。範囲外は端へ丸める。
pub fn frame_at_x(x: f32, width: f32, duration_frames: i64) -> i64 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0;
    }
    let ratio = (x / width).clamp(0.0, 1.0);
    let frame = (ratio * duration_frames as f32).round() as i64;
    frame.clamp(0, (duration_frames - 1).max(0))
}

/// 目盛りの目標セル比率(小目盛間隔 px / 行高 px)。利用者裁定 2026-08-21 夜
/// 「比率の原則」(`docs/ui-spatial-score.md` S4)— **形は絶対 px でなく比率で
/// 定数化する**(x/y の2変数が1つの数へ畳まれスケール不変になり、モックと
/// 実装を同じ定数で機械検査できる)。合格モック
/// (`next/reference/mocks/timeline-semantics.html`)実測 13.5px/26px ≈ 0.52 が
/// 出典。[`tick_steps`] は [`step_ladder_frames`] の中からこの比率に**最も
/// 近い**ステップを選ぶ — σ 初回実装は `MIN_MINOR_TICK_PX`(px 絶対下限)で
/// 梯子を選び 0.92 になり「モック通りでない」を利用者が検出した実例の修理
/// (発注書 EXACT TARGET 1)。絶対 px が許されるのは物理由来の下限のみ
/// (ヒット寸 12px 等 — ここは該当しない)。
/// 目盛りの候補ステップ(フレーム数、昇順・重複無し)。**時刻へ絶対整列**
/// (0, step, 2*step, ... — 全尺等分と違い端数のフレームが出ない)。
///
/// fps が引ければ「1f, 5f, 10f, 1s, 2s, 5s, 10s, 30s, 1m, 5m」という
/// 人間に読みやすい混合ラダー(発注書 EXACT TARGET 1 の候補列そのもの)。
/// fps が引けない(comp 無し)時は秒/分を frame へ直せないので、先頭の
/// 「1, 5, 10」十進ラダーだけへ落ちる(`duration_frames` も同時に 0 になる
/// 経路がほとんどなので、この短いラダーで実害は無い — [`TimelinePane::new`]
/// 参照)。
fn step_ladder_frames(fps: Option<Fps>) -> Vec<i64> {
    // 2f と半秒(fps/2)を含む(σ2 検収 FINDING の処置 2026-08-21: 梯子が粗いと
    // 比率最近傍でも目標 0.52 に届かない — 30fps・幅1426px で 10f=0.305/30f=0.914 の
    // 二択だった穴を 15f=0.457 が埋める。半秒は時間整列の正当な段)。
    let mut out: Vec<i64> = vec![1, 2, 5, 10];
    if let Some(fps) = fps {
        let fps = fps.as_f64();
        for seconds in [0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 300.0] {
            out.push((fps * seconds).round().max(1.0) as i64);
        }
        out.sort_unstable();
        out.dedup();
    }
    out
}

/// ルーラー目盛りの小目盛/大目盛のステップ(フレーム数)。利用者裁定
/// 2026-08-21 夜: 全尺等分(旧 `RULER_TICK_DIVISIONS`)を撤去し、小目盛/大目盛の
/// 階層を導入する。明暗帯([`time_band_segment_frames`])はこの大目盛の周期に
/// 揃える(単一のラダーが目盛りと明暗帯の両方の出典 — 2箇所で別の刻み方を
/// 持たない)。
///
/// 小目盛 = [`step_ladder_frames`] の中で「セル比率(小目盛の px 間隔 /
/// `row_height`)が JSON 正本の `target_cell_ratio` に最も近い」ステップ(比率の原則
/// — 発注書 EXACT TARGET 1・2)。同点(浮動小数の完全一致)はラダー順で先に
/// 見つかった方(`Iterator::min_by` の安定順)を採る。
///
/// 大目盛 = ラダー上で小目盛のちょうど5倍か10倍になっている直近上位のステップ
/// (`draw_ruler_ticks` doc 参照 — 秒/分混在ラダーは全区間が等比ではないため、
/// 2倍/3倍しか離れていない隣接ステップは飛ばす)。ラダー上に見つからなければ
/// 小目盛の10倍を直接計算し(ラダー外でも構わない)、「大目盛は常に小目盛の
/// 整数倍」だけは常に守る。時間整列(0, step, 2*step, ...)は不変。
///
/// `pub`: `super::canvas::draw_ruler_ticks`/`draw_time_bands` と
/// `motolii_shell::screenshot` 器具が同じ刻みを再現するため(`frame_to_x` と
/// 同じ理由)。`row_height` は呼び手の `dims.row_height`(pane 側)/
/// `dims.row_height`(screenshot 側)をそのまま渡す — 比率の分母は常にこの1つ
/// (2箇所で別の行高を使わない)。
pub fn tick_steps(
    fps: Option<Fps>,
    duration_frames: i64,
    clip_width: f32,
    row_height: f32,
) -> (i64, i64) {
    tick_steps_with_target(
        fps,
        duration_frames,
        clip_width,
        row_height,
        motolii_tokens_rs::Dimensions::default()
            .components
            .timeline
            .target_cell_ratio,
    )
}

pub fn tick_steps_with_target(
    fps: Option<Fps>,
    duration_frames: i64,
    clip_width: f32,
    row_height: f32,
    target_cell_ratio: f32,
) -> (i64, i64) {
    let ladder = step_ladder_frames(fps);
    if duration_frames <= 0 || clip_width <= 0.0 || row_height <= 0.0 {
        let minor = ladder.first().copied().unwrap_or(1);
        return (minor, minor.saturating_mul(5));
    }
    let px_per_frame = clip_width / duration_frames as f32;
    let cell_ratio_gap = |step: i64| -> f32 {
        (step as f32 * px_per_frame / row_height - target_cell_ratio).abs()
    };
    let minor = ladder
        .iter()
        .copied()
        .min_by(|&a, &b| {
            cell_ratio_gap(a)
                .partial_cmp(&cell_ratio_gap(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(1);
    let major = ladder
        .iter()
        .copied()
        .find(|&candidate| {
            candidate > minor
                && candidate % minor == 0
                && matches!(candidate / minor, 5 | 10)
        })
        .unwrap_or_else(|| minor.saturating_mul(10));
    (minor, major)
}

/// 時間方向の明暗リズム(裁定148(1))の区間幅(フレーム数)。**大目盛の周期に
/// 揃える**([`tick_steps`] の第2要素、利用者裁定 2026-08-21 夜 — 旧・固定1秒/
/// `RULER_TICK_DIVISIONS` 等分は撤去)。`draw_time_bands` と screenshot 器具の
/// 両方がこの1つの式から区間境界を出す(2箇所で別のフォールバックを持たない)。
///
/// `clip_width`/`row_height` を引数に足した(裁定160 切片7時点は `clip_width`
/// すら無かった) — 大目盛は JSON 正本の `target_cell_ratio` のセル比率(`clip_width`と
/// `row_height` の両方に依存)から出るので、区間幅も同じ入力を要る。
///
/// `pub`: `motolii_shell::screenshot` が Timeline canvas と同じ区間の刻み方を
/// 再現するため(`frame_to_x` と同じ理由、裁定160 切片7で緩めた)。
pub fn time_band_segment_frames(
    fps: Option<Fps>,
    duration_frames: i64,
    clip_width: f32,
    row_height: f32,
) -> i64 {
    time_band_segment_frames_with_target(
        fps,
        duration_frames,
        clip_width,
        row_height,
        motolii_tokens_rs::Dimensions::default()
            .components
            .timeline
            .target_cell_ratio,
    )
}

pub fn time_band_segment_frames_with_target(
    fps: Option<Fps>,
    duration_frames: i64,
    clip_width: f32,
    row_height: f32,
    target_cell_ratio: f32,
) -> i64 {
    tick_steps_with_target(
        fps,
        duration_frames,
        clip_width,
        row_height,
        target_cell_ratio,
    )
    .1
}

#[cfg(test)]
mod tick_tests {
    use super::*;
    use motolii_tokens_rs::Dimensions;

    fn target_cell_ratio() -> f32 {
        Dimensions::default().components.timeline.target_cell_ratio
    }

    fn fps30() -> Fps {
        Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
    }

    /// 合格モックの実測 `row_height`(`next/reference/mocks/timeline-semantics.html`
    /// の行高 26px)— 発注書の EXACT TARGET/ORACLE (a) が指す値。
    const MOCK_ROW_HEIGHT: f32 = 26.0;

    /// ラダーの中で JSON 正本の `target_cell_ratio` に最も近いステップを、テスト側でも
    /// 独立に計算する(発注書オラクル(a)「期待値はテスト内で梯子から計算」)。
    /// `tick_steps` 本体と同じ式を **意図して重複実装**する — 実装のバグを
    /// 実装自身の式で覆い隠さないため(oracle は独立検算)。
    fn nearest_minor_by_ratio(duration_frames: i64, clip_width: f32, row_height: f32) -> i64 {
        let ladder = step_ladder_frames(Some(fps30()));
        let px_per_frame = clip_width / duration_frames as f32;
        ladder
            .into_iter()
            .min_by(|&a, &b| {
                let gap = |step: i64| {
                    (step as f32 * px_per_frame / row_height - target_cell_ratio()).abs()
                };
                gap(a).partial_cmp(&gap(b)).unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("ladder は常に非空(step_ladder_frames が最低 [1,5,10] を返す)")
    }

    /// **オラクル(a)**: 30fps・尺1800f・幅1426px・行高26px(発注書 EXACT TARGET)
    /// → 選ばれる小目盛が、全ラダー候補の中でセル比率(spacing_px/row_height)を
    /// JSON 正本のセル比率(0.52)に最も近づけるステップと一致する。具体値
    /// (10f)も assert する — 期待値はテスト内の独立計算([`nearest_minor_by_ratio`])
    /// から出しており、実装のマジックナンバーをそのまま複製していない。
    #[test]
    fn tick_steps_picks_the_ladder_step_nearest_the_target_cell_ratio() {
        // 期待値 15f = 半秒(σ2 検収 FINDING の処置で梯子へ 2f・fps/2 を追加した後の
        // 最近傍: 15f→比率0.457。旧梯子では 10f→0.305 が最善だった — 梯子の粗さが
        // 比率原則の到達度を制約していた実例)。
        let expected_minor = nearest_minor_by_ratio(1800, 1426.0, MOCK_ROW_HEIGHT);
        assert_eq!(expected_minor, 15, "テスト側の独立計算自体が想定値からずれている");

        let (minor, major) = tick_steps(Some(fps30()), 1800, 1426.0, MOCK_ROW_HEIGHT);
        assert_eq!(minor, expected_minor, "小目盛が比率最近傍のステップになっていない");
        assert_eq!(minor, 15, "小目盛の具体値が想定(15f=半秒)とずれている");
        assert_eq!(major % minor, 0, "大目盛が小目盛の整数倍でない");
    }

    /// **オラクル(a)**: 幅810px(モック寸そのもの)でも同じ比率最近傍の原則が
    /// 成立する — このケースは実際にモック実測(13.5px/26px≈0.519)に極めて近い
    /// step(30f=1s級)を選ぶ、比率原則の直接の実例。
    #[test]
    fn tick_steps_picks_the_nearest_ratio_step_at_mock_width_too() {
        let expected_minor = nearest_minor_by_ratio(1800, 810.0, MOCK_ROW_HEIGHT);
        assert_eq!(expected_minor, 30, "テスト側の独立計算自体が想定値からずれている");

        let (minor, _major) = tick_steps(Some(fps30()), 1800, 810.0, MOCK_ROW_HEIGHT);
        assert_eq!(minor, expected_minor, "幅810pxで比率最近傍のステップになっていない");

        // 実際にモック実測比率(0.52)へ極めて近いことも確認する(比率の原則の
        // 目的そのもの — 「モック通りでない」の再発防止)。
        let px_per_frame = 810.0 / 1800.0;
        let ratio = minor as f32 * px_per_frame / MOCK_ROW_HEIGHT;
        assert!(
            (ratio - target_cell_ratio()).abs() < 0.01,
            "選ばれた小目盛のセル比率がモック実測(0.52)から遠すぎる: {ratio}"
        );
    }

    /// **オラクル(a)**: 大目盛は常に小目盛の整数倍(5倍か10倍)— どんな尺でも。
    #[test]
    fn major_is_always_an_integer_multiple_of_minor() {
        for duration in [1, 2, 10, 37, 100, 1_800, 12_345, 100_000, 5_000_000] {
            let (minor, major) = tick_steps(Some(fps30()), duration, 1349.0, MOCK_ROW_HEIGHT);
            assert!(minor >= 1, "小目盛が1未満に退化した(duration={duration})");
            assert!(
                major >= minor && major % minor == 0,
                "大目盛が小目盛の整数倍でない(duration={duration}, minor={minor}, major={major})"
            );
        }
    }

    /// **オラクル(a)**: 極端に短い尺(10f)でも最小1f まで退化するだけで
    /// パニックしない・0にならない。
    #[test]
    fn tick_steps_does_not_degenerate_on_a_tiny_duration() {
        let (minor, major) = tick_steps(Some(fps30()), 10, 1349.0, MOCK_ROW_HEIGHT);
        assert!(minor >= 1);
        assert!(major >= minor);
    }

    /// **オラクル(a)**: 極端に長い尺(100000f)でも同じラダーから退化なく
    /// 値が出る(巨大 duration で minor/major が0や負にならない)。
    #[test]
    fn tick_steps_does_not_degenerate_on_a_huge_duration() {
        let (minor, major) = tick_steps(Some(fps30()), 100_000, 1349.0, MOCK_ROW_HEIGHT);
        assert!(minor >= 1);
        assert!(major > 0 && major % minor == 0);
    }

    /// fps が引けない(comp 無し)時も 0 割り/パニックせず、最小ラダー
    /// (1,5,10)から値を返す。
    #[test]
    fn tick_steps_without_fps_falls_back_to_the_short_ladder() {
        let (minor, major) = tick_steps(None, 100, 1349.0, MOCK_ROW_HEIGHT);
        assert!(minor >= 1);
        assert!(major >= minor && major % minor == 0);
    }

    /// `duration_frames <= 0`/`clip_width <= 0.0`/`row_height <= 0.0` は空 comp
    /// と同じ安全側(パニックしない、`minor <= major`)。`row_height` の退化
    /// ガードは比率原則導入で新設した経路(発注書 EXACT TARGET 2 の「退化ガード
    /// は維持」— 0除算の分母が増えたので、ここも同格で守る)。
    #[test]
    fn tick_steps_guards_non_positive_inputs() {
        assert_eq!(tick_steps(Some(fps30()), 0, 1349.0, MOCK_ROW_HEIGHT).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 0.0, MOCK_ROW_HEIGHT).0, 1);
        assert_eq!(tick_steps(Some(fps30()), -5, 1349.0, MOCK_ROW_HEIGHT).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 1349.0, 0.0).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 1349.0, -1.0).0, 1);
    }

    /// **オラクル(b)**: 明暗帯の区間幅(旧 `time_band_segment_frames`)は
    /// `tick_steps` の大目盛と常に一致する(2箇所で別の刻み方を持たない)。
    #[test]
    fn time_band_segment_matches_tick_steps_major() {
        let (_minor, major) = tick_steps(Some(fps30()), 1800, 1349.0, MOCK_ROW_HEIGHT);
        assert_eq!(
            time_band_segment_frames(Some(fps30()), 1800, 1349.0, MOCK_ROW_HEIGHT),
            major
        );
    }
}
