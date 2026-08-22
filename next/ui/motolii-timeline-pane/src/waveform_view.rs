//! Timeline クリップバー内の**音声波形(WAVE の顔)**の意味側の投影(発注
//! 2026-08-22「Timeline の音声波形表示」)。map(`next/reference/normal-map.tsv`)
//! 採用予定 id 74(`Zoom Audio Waveform`、bundle B24)が直接の消化対象 —
//! [`required_buckets`]/[`plan`] の「クリップ画面幅(=ズーム/パンの帰結)に
//! 応じて要求 bucket 数が変わる」写像がその意味そのもの。id 58
//! (`Show Audio Track Layers`、bundle B52)は**このモジュールでは消化しない**
//! — 1クリップの合成波形(モノラル畳み込み min/max)ではなく複数音声レイヤーを
//! 並べて見せる別の意味(RETURN 参照、次波以降の候補)。
//!
//! ## 何をするか / しないか
//! `motolii_media::waveform_peaks(path, buckets) -> Result<Vec<(f32, f32)>>`
//! (2026-08-22 着地、ffmpeg サイドカー・ストリーミング畳み込み・LRU 64
//! キャッシュ)が意味の一次資料。**このモジュールは `motolii-media` を
//! 一切叩かない** — この crate の `Cargo.toml` は発注書 EXACT TARGET で
//! 読み専用(新ファイル縛り)のため、依存追加自体が次波の仕事になる。
//! ここが持つのはその手前の**純粋な投影**だけ:
//!
//! - クリップの画面幅(px)から要求すべき bucket 数を決める写像
//!   ([`required_buckets`])と、再計算が要るかの判定([`needs_recompute`])
//! - 「まだ無い/取得中/ある」の3状態を持つ投影型 [`WaveformState`]
//! - 状態+画面幅から「この frame で何をすべきか」を返す純関数 [`plan`]
//!   (**発火はしない** — 呼び出し側が `WaveformAction::Fetch` を見て実際の
//!   非同期取得を起こす。`Message`/`PaneState` への配線・fetch の発火自体は
//!   次波 — `write.rs`/`canvas.rs` は既存ファイルで発注書の対象外)
//! - peaks(min, max のバケット列)をクリップの画面矩形へ写す純関数
//!   ([`waveform_segments`]/[`waveform_state_segments`])
//! - 波形インク色(tokens `data` ロール × クリップ色、[`waveform_ink`])
//!
//! **`canvas.rs` は無改変**(発注書 EXACT TARGET: 新ファイル縛り・既存ファイル
//! 読み専用)。実際にこのモジュールの関数を `canvas::draw` から呼ぶ配線
//! (`RowProjection` へ音声の有無/path を足す、`PaneState` に
//! `HashMap<LayerId, WaveformState>` を持たせる、非同期 fetch を起こす)は
//! 次波の統合手順(RETURN 参照)。
//!
//! ## bucket 数の写像(裁定なし・実装内固定値 — `canvas::loop_band_height`
//! と同じ「実窓で見てから直す」姿勢)
//! 1 bucket ≈ [`TARGET_PX_PER_BUCKET`] px。ズーム/パンで `clip_width_px` が
//! 連続的に変わっても、要求 bucket 数は [`BUCKET_QUANTUM`] の階段にしか
//! 丸めない — [`plan`] の再計算判定は「丸め後の値が変わったか」だけを見るので、
//! 1px 動くたびに ffmpeg 再デコードを発火しない(自然なヒステリシス、
//! 専用のタイマー/しきい値ロジックを別に持たない)。
//!
//! ## 3状態(発注書「まだ無い/取得中/ある」)
//! [`WaveformState::NotRequested`](未着手/音声なし)・
//! [`WaveformState::Loading`](取得中、要求した bucket 数を覚える)・
//! [`WaveformState::Ready`](取得済み)。**この型は状態を持ち回らない** —
//! `work_area.rs`/`shuttle.rs` と同じ「値を受けて値を返すだけ」の純関数群の
//! 対象として存在し、実体は次波で `PaneState` 側に置かれる想定(RETURN 参照)。

use iced::{Color, Rectangle};

/// 1 bucket が目標とする画面幅(px)。小さくすると波形の解像度は上がるが
/// 要求 bucket 数(= ffmpeg 側の畳み込み量)が増える。
pub const TARGET_PX_PER_BUCKET: f32 = 2.0;

/// 要求 bucket 数の丸め幅。ズーム/パン中の連続的な px 変化を吸収し、
/// [`plan`] が1px単位で再取得を発火しないようにする(ヒステリシス)。
pub const BUCKET_QUANTUM: usize = 32;

/// クリップの画面幅(px)から、要求すべき bucket 数を決める純関数
/// (発注書 1「要求する buckets = クリップの画面幅 px を基準にする写像」)。
/// 退化(幅が 0 以下・非有限)は `0`(=要求しない)。
pub fn required_buckets(clip_width_px: f32) -> usize {
    if !clip_width_px.is_finite() || clip_width_px <= 0.0 {
        return 0;
    }
    let raw = (clip_width_px / TARGET_PX_PER_BUCKET).ceil().max(1.0) as usize;
    raw.div_ceil(BUCKET_QUANTUM) * BUCKET_QUANTUM
}

/// 今持っている bucket 数(取得中/取得済みのどちらか、未着手なら `None`)と、
/// 新しく要求すべき bucket 数を比べ、再取得が要るかを判定する
/// (発注書 1「再計算の要否判定も」)。`required == 0`(クリップ幅が退化)は
/// 常に不要 — 要求すべき数が無いので取得しようがない。
pub fn needs_recompute(current: Option<usize>, required: usize) -> bool {
    if required == 0 {
        return false;
    }
    current != Some(required)
}

/// クリップの波形取得状態。**「まだ無い/取得中/ある」の3状態**そのもの
/// (発注書3)。engine 側 I/O は非同期なので、この型は結果を待つ間の顔を
/// 名指しできる。
#[derive(Debug, Clone, PartialEq)]
pub enum WaveformState {
    /// 未着手(音声を持たないクリップ、またはまだ要求していない)。
    NotRequested,
    /// 取得中。要求した bucket 数を覚えておく — 結果が届いた時に
    /// [`plan`] がまだ有効な要求か(その間にズームされていないか)を
    /// 照合できるようにするため(次波の結線がこの値を使う)。
    Loading { buckets: usize },
    /// 取得済み。`buckets` は取得時に要求した数 — `peaks.len()` と食い違って
    /// いても [`waveform_segments`] は panic しない(実際に来た `peaks` の
    /// 長さだけを見る、EXACT TARGET 5「退化」の一種として扱う)。
    Ready { buckets: usize, peaks: Vec<(f32, f32)> },
}

impl WaveformState {
    /// 今の状態が持つ bucket 数(未着手なら `None`)。[`plan`] が使う。
    fn buckets(&self) -> Option<usize> {
        match self {
            WaveformState::NotRequested => None,
            WaveformState::Loading { buckets } | WaveformState::Ready { buckets, .. } => {
                Some(*buckets)
            }
        }
    }
}

/// [`plan`] の結果。**発火そのものはしない**(純関数は I/O を持てない) —
/// 呼び出し側が `Fetch` を見たら実際の非同期取得を起こす(次波の結線)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformAction {
    /// 何もしない(音声が無い/既に最新の bucket 数で取得中・取得済み)。
    None,
    /// この bucket 数で(再)取得を発火すべき。
    Fetch(usize),
}

/// 今の状態・クリップ画面幅・音声の有無から、この frame で取るべき行動を
/// 決める純関数(発注書3の「投影」の中心)。`has_audio == false`(音声を
/// 持たないクリップ)なら常に `None` — 音声の無いクリップへ波形を要求しない。
pub fn plan(state: &WaveformState, clip_width_px: f32, has_audio: bool) -> WaveformAction {
    if !has_audio {
        return WaveformAction::None;
    }
    let required = required_buckets(clip_width_px);
    if needs_recompute(state.buckets(), required) {
        WaveformAction::Fetch(required)
    } else {
        WaveformAction::None
    }
}

/// 波形1本分の描画データ(画面1列ぶん)。`x`/`y_top`/`y_bottom` は呼び出し側の
/// `rect` と同じ絶対座標系(`canvas::Frame` がそのまま使える想定 — 次波の
/// 配線で追加のオフセット計算を挟まないため)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveformSegment {
    pub x: f32,
    pub y_top: f32,
    pub y_bottom: f32,
}

/// peaks(min, max のバケット列)をクリップの画面矩形 `rect` へ写す純関数
/// (発注書1「peaks をクリップの画面幅に写し、中央線からの上下として描く」)。
/// 1画面 px = 1 出力列。`y_top`/`y_bottom` は `rect` の中央線(`rect.y +
/// rect.height / 2`)からの上下 — 振幅は `[-1, 1]` へ clamp してから
/// `rect.height / 2` を掛ける(壊れた/規格外の peaks 値で矩形の外へ
/// はみ出させない)。
///
/// `peaks.len()` と出力列数(= `rect.width` の切り捨て)が一致しなくても
/// 動く — 取得完了までの間にリサイズ/ズームされていた場合の安全側として、
/// 各列は最寄りの bucket(`column * peaks.len() / width`)を選ぶ。
///
/// 退化(`peaks` が空、`rect.width <= 0`)は空の `Vec`(描かない、panic しない
/// — 発注書5「退化(0 bucket/無音)」)。
pub fn waveform_segments(peaks: &[(f32, f32)], rect: Rectangle) -> Vec<WaveformSegment> {
    let width = if rect.width.is_finite() {
        rect.width.floor().max(0.0) as usize
    } else {
        0
    };
    if width == 0 || peaks.is_empty() {
        return Vec::new();
    }
    let center = rect.y + rect.height / 2.0;
    let half = rect.height / 2.0;
    let mut out = Vec::with_capacity(width);
    for column in 0..width {
        let index = (column * peaks.len() / width).min(peaks.len() - 1);
        let (min, max) = peaks[index];
        let max = max.clamp(-1.0, 1.0);
        let min = min.clamp(-1.0, 1.0);
        out.push(WaveformSegment {
            x: rect.x + column as f32,
            // screen y は下方向が正 — 正の振幅(max)は中央線より**上**(y が
            // 小さい方)、負の振幅(min)は**下**(y が大きい方)へ振れる。
            y_top: center - max * half,
            y_bottom: center - min * half,
        });
    }
    out
}

/// [`WaveformState`] から直接描画データを作る薄いラッパー。`Ready` 以外
/// (`NotRequested`/`Loading`)は空 — 「まだ無い/取得中」の顔(プレースホルダ
/// テキスト等)を出すかどうかは呼び出し側 canvas の判断で、この関数自体は
/// データが無ければ何も描かない(装飾を発明しない)。
pub fn waveform_state_segments(state: &WaveformState, rect: Rectangle) -> Vec<WaveformSegment> {
    match state {
        WaveformState::Ready { peaks, .. } => waveform_segments(peaks, rect),
        WaveformState::NotRequested | WaveformState::Loading { .. } => Vec::new(),
    }
}

/// `data`(tokens `Colors::data` ロール)と `bar_color`(クリップ色)を
/// 混ぜる比率。**波形はデータであって装飾ではない**(発注書2、裁定179の
/// 「状態の器」とは別カテゴリ)ので `data` を主色にしつつ、クリップの識別色
/// (`bar_color`)を完全には消さない程度に混ぜる。裁定の無い初回実装値 —
/// `canvas::loop_band_height` と同じ「実窓で見てから直す」対象。
pub const WAVEFORM_DATA_MIX: f32 = 0.7;

/// 波形インク色 — tokens `data` ロールをクリップ色の上へ明度差として重ねる
/// (発注書2「色は tokens の data ロール系・クリップ色の上に重ねる明度差で」)。
/// `alpha` は `data` のものをそのまま使う(クリップ色の透明度と混ぜない —
/// 波形の可視性は色ロール自身が持つ既定の濃さで決まる)。
pub fn waveform_ink(data: Color, bar_color: Color) -> Color {
    Color {
        r: lerp(bar_color.r, data.r, WAVEFORM_DATA_MIX),
        g: lerp(bar_color.g, data.g, WAVEFORM_DATA_MIX),
        b: lerp(bar_color.b, data.b, WAVEFORM_DATA_MIX),
        a: data.a,
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------
    // required_buckets — 画面幅→bucket 数の写像・退化
    // ---------------------------------------------------------------------

    #[test]
    fn required_buckets_is_zero_for_degenerate_widths() {
        assert_eq!(required_buckets(0.0), 0, "幅0は要求しない");
        assert_eq!(required_buckets(-10.0), 0, "負の幅は要求しない");
        assert_eq!(required_buckets(f32::NAN), 0, "NaN は要求しない");
        assert_eq!(required_buckets(f32::INFINITY), 0, "非有限は要求しない");
    }

    #[test]
    fn required_buckets_rounds_up_to_the_quantum() {
        assert_eq!(required_buckets(1.0), BUCKET_QUANTUM, "小さい幅でも最小の量子まで丸まる");
        assert_eq!(
            required_buckets(BUCKET_QUANTUM as f32 * TARGET_PX_PER_BUCKET),
            BUCKET_QUANTUM,
            "ちょうど1量子ぶんの幅は1量子のまま"
        );
        assert_eq!(
            required_buckets(BUCKET_QUANTUM as f32 * TARGET_PX_PER_BUCKET + 1.0),
            BUCKET_QUANTUM * 2,
            "1量子を1pxでも超えたら次の量子へ切り上がる"
        );
    }

    #[test]
    fn required_buckets_grows_monotonically_with_width() {
        let mut previous = required_buckets(1.0);
        for width in [50.0, 200.0, 800.0, 3200.0, 20_000.0] {
            let current = required_buckets(width);
            assert!(current >= previous, "幅が増えたのに要求 bucket 数が減った: {width}px");
            previous = current;
        }
    }

    // ---------------------------------------------------------------------
    // needs_recompute — ヒステリシス(量子化された値の一致判定)
    // ---------------------------------------------------------------------

    #[test]
    fn needs_recompute_is_false_when_required_is_zero() {
        assert!(!needs_recompute(None, 0), "要求すべき数が無ければ再計算しない");
        assert!(!needs_recompute(Some(64), 0), "要求すべき数が無ければ再計算しない");
    }

    #[test]
    fn needs_recompute_is_true_on_first_request() {
        assert!(needs_recompute(None, 32), "未着手からは常に再計算が要る");
    }

    #[test]
    fn needs_recompute_ignores_a_matching_current_value() {
        assert!(!needs_recompute(Some(64), 64), "同じ bucket 数なら再計算しない");
    }

    #[test]
    fn needs_recompute_fires_only_when_the_quantized_value_changes() {
        // 同じ量子内(required_buckets が同じ値を返す)幅の変化は、
        // needs_recompute 自体には反映されない(呼び出し側が同じ required を
        // 渡す限り false のまま) — ヒステリシスは required_buckets の量子化が
        // 担い、needs_recompute は単純な不一致判定に徹する。
        let a = required_buckets(10.0);
        let b = required_buckets(20.0);
        assert_eq!(a, b, "同じ量子に収まる幅は同じ bucket 数を返す(前提の確認)");
        assert!(!needs_recompute(Some(a), b));

        let c = required_buckets(100_000.0);
        assert_ne!(a, c, "十分離れた幅は別の量子になる(前提の確認)");
        assert!(needs_recompute(Some(a), c));
    }

    // ---------------------------------------------------------------------
    // plan — 3状態 × has_audio の投影
    // ---------------------------------------------------------------------

    #[test]
    fn plan_never_fetches_for_a_clip_without_audio() {
        assert_eq!(
            plan(&WaveformState::NotRequested, 500.0, false),
            WaveformAction::None,
            "音声の無いクリップは要求しない"
        );
        assert_eq!(
            plan(&WaveformState::Ready { buckets: 64, peaks: vec![(0.0, 0.0)] }, 500.0, false),
            WaveformAction::None,
        );
    }

    #[test]
    fn plan_fetches_on_first_sight_of_an_audio_clip() {
        let required = required_buckets(500.0);
        assert_eq!(
            plan(&WaveformState::NotRequested, 500.0, true),
            WaveformAction::Fetch(required),
            "未着手の音声クリップは最初のフレームで要求すべき"
        );
    }

    #[test]
    fn plan_does_nothing_while_loading_the_same_bucket_count() {
        let required = required_buckets(500.0);
        let state = WaveformState::Loading { buckets: required };
        assert_eq!(plan(&state, 500.0, true), WaveformAction::None, "取得中に同じ要求を重複発火しない");
    }

    #[test]
    fn plan_does_nothing_once_ready_at_the_same_bucket_count() {
        let required = required_buckets(500.0);
        let state = WaveformState::Ready { buckets: required, peaks: vec![(0.0, 0.0); required] };
        assert_eq!(plan(&state, 500.0, true), WaveformAction::None, "既に最新なら再取得しない");
    }

    #[test]
    fn plan_refetches_when_the_zoomed_width_crosses_a_quantum() {
        let narrow = required_buckets(10.0);
        let state = WaveformState::Ready { buckets: narrow, peaks: vec![(0.0, 0.0); narrow] };
        let wide_required = required_buckets(100_000.0);
        assert_ne!(narrow, wide_required, "テスト前提: 十分ズームすれば別の量子になる");
        assert_eq!(
            plan(&state, 100_000.0, true),
            WaveformAction::Fetch(wide_required),
            "大きくズームしたら新しい bucket 数で再取得すべき"
        );
    }

    // ---------------------------------------------------------------------
    // waveform_segments — peaks→画面座標の写像・退化
    // ---------------------------------------------------------------------

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle { x, y, width, height }
    }

    #[test]
    fn waveform_segments_is_empty_for_zero_buckets() {
        let out = waveform_segments(&[], rect(0.0, 0.0, 100.0, 20.0));
        assert!(out.is_empty(), "0 bucket(空の peaks)は何も描かない");
    }

    #[test]
    fn waveform_segments_is_empty_for_a_degenerate_rect() {
        let peaks = vec![(-0.5, 0.5), (-0.2, 0.8)];
        assert!(waveform_segments(&peaks, rect(0.0, 0.0, 0.0, 20.0)).is_empty(), "幅0は描かない");
        assert!(waveform_segments(&peaks, rect(0.0, 0.0, -5.0, 20.0)).is_empty(), "負の幅は描かない");
        assert!(
            waveform_segments(&peaks, rect(0.0, 0.0, f32::NAN, 20.0)).is_empty(),
            "非有限の幅は描かない"
        );
    }

    #[test]
    fn silence_produces_a_flat_line_on_the_center() {
        // 発注書5「無音」の退化: 全 bucket が (0.0, 0.0) なら、全列が中央線
        // ちょうどに乗る(上下どちらにも振れない — まだ読めていないのか本当に
        // 無音なのかを区別できる線として意味を持つ、旧世界 egui 版と同じ姿勢)。
        let peaks = vec![(0.0, 0.0); 8];
        let r = rect(10.0, 100.0, 8.0, 20.0);
        let out = waveform_segments(&peaks, r);
        assert_eq!(out.len(), 8);
        let center = r.y + r.height / 2.0;
        for segment in &out {
            assert_eq!(segment.y_top, center, "無音の上端が中央線からずれている");
            assert_eq!(segment.y_bottom, center, "無音の下端が中央線からずれている");
        }
    }

    #[test]
    fn full_scale_peaks_reach_the_rect_edges() {
        // (min, max) = (-1, 1) は矩形の上端/下端ちょうどまで振れる。
        let peaks = vec![(-1.0, 1.0)];
        let r = rect(0.0, 0.0, 4.0, 20.0);
        let out = waveform_segments(&peaks, r);
        for segment in &out {
            assert_eq!(segment.y_top, r.y, "最大振幅が矩形の上端に届いていない");
            assert_eq!(segment.y_bottom, r.y + r.height, "最小振幅が矩形の下端に届いていない");
        }
    }

    #[test]
    fn out_of_range_peak_values_are_clamped_inside_the_rect() {
        // 壊れた/規格外の peaks(|value| > 1)でも矩形の外へはみ出さない。
        let peaks = vec![(-5.0, 5.0)];
        let r = rect(0.0, 0.0, 2.0, 20.0);
        let out = waveform_segments(&peaks, r);
        for segment in &out {
            assert!(segment.y_top >= r.y && segment.y_top <= r.y + r.height);
            assert!(segment.y_bottom >= r.y && segment.y_bottom <= r.y + r.height);
        }
    }

    #[test]
    fn column_count_follows_the_rect_width_not_the_bucket_count() {
        // peaks の本数と出力列数が食い違っても(取得完了までにリサイズ/ズーム
        // された場合の安全側)、出力列数は常に rect の幅(切り捨て)。
        let peaks = vec![(-0.3, 0.3); 3];
        let r = rect(0.0, 0.0, 10.0, 20.0);
        let out = waveform_segments(&peaks, r);
        assert_eq!(out.len(), 10, "出力列数は rect.width の切り捨てであるべき");
    }

    #[test]
    fn bucket_selection_is_monotonic_across_columns() {
        // 列が進むほど選ばれる bucket の添字は後退しない(飛び先が乱れない)。
        let peaks: Vec<(f32, f32)> = (0..20).map(|i| (0.0, i as f32 / 20.0)).collect();
        let r = rect(0.0, 0.0, 200.0, 20.0);
        let out = waveform_segments(&peaks, r);
        let center = r.y + r.height / 2.0;
        let mut last_amplitude = f32::NEG_INFINITY;
        for segment in out.iter().step_by(10) {
            let amplitude = center - segment.y_top; // max 側の実効振幅(正)
            assert!(
                amplitude + 1e-6 >= last_amplitude,
                "振幅が単調増加の peaks から逆行して選ばれている"
            );
            last_amplitude = amplitude;
        }
    }

    #[test]
    fn waveform_state_segments_is_empty_unless_ready() {
        let r = rect(0.0, 0.0, 10.0, 20.0);
        assert!(waveform_state_segments(&WaveformState::NotRequested, r).is_empty());
        assert!(waveform_state_segments(&WaveformState::Loading { buckets: 32 }, r).is_empty());
        let ready = WaveformState::Ready { buckets: 4, peaks: vec![(-0.1, 0.1); 4] };
        assert!(!waveform_state_segments(&ready, r).is_empty(), "Ready なら描く");
    }

    // ---------------------------------------------------------------------
    // waveform_ink — data ロール×クリップ色
    // ---------------------------------------------------------------------

    #[test]
    fn waveform_ink_leans_toward_the_data_role_but_keeps_some_bar_color() {
        let data = Color::from_rgb(0.0, 1.0, 0.0);
        let bar = Color::from_rgb(1.0, 0.0, 0.0);
        let ink = waveform_ink(data, bar);
        assert!(ink.g > 0.5, "data ロールが主色になっていない: {ink:?}");
        assert!(ink.r > 0.0, "クリップ色が完全に消えている: {ink:?}");
        assert_eq!(ink.a, data.a, "alpha は data ロールのものを使うべき");
    }

    #[test]
    fn waveform_ink_matches_data_exactly_when_bar_color_equals_data() {
        let data = Color::from_rgb(0.4706, 0.7098, 0.6902);
        let ink = waveform_ink(data, data);
        assert_eq!(ink, data, "同色同士の混色は元の色に一致するべき");
    }
}
