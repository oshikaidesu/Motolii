//! owns: timeline 波形表示の**意味側**(peaks 抽出)。表示(次波)はこのAPIを叩くだけ。
//!
//! ## 何をするか
//! `waveform_peaks(path, buckets)` は音声を先頭から末尾まで読み、`buckets` 個の
//! バケットそれぞれについて (min, max) を返す。1バケット=時間軸上の等間隔区間
//! (総尺 / buckets)。モノラルへ折り畳んでから min/max を取る(ステレオのLとRを
//! 別々に見せる意味論はまだ無い — 必要になったら (min,max) を2組返す形へ拡張)。
//!
//! ## デコード経路(既存を再利用)
//! [`decode::FrameReader`](crate::FrameReader) と同じ「ffmpegサイドカーへ
//! rawフォーマット出力させ、stdoutをパイプで読む」形をそのまま踏襲する。
//! 音声は `-map 0:a:0 -vn -ac 2 -ar 48000 -f f32le -` で
//! **48kHz・interleaved stereo・f32le** に正規化してから読む
//! (48kHzは[`crate::write_f32le_wav_stereo_48k`]と同じ正準レートを再利用 — 発明しない)。
//! チャンネル折り畳みはffmpegの `-ac 1` に任せない: 5.1等の重み付きダウンミックス
//! アルゴリズムはffmpegのビルド・バージョンで挙動が変わり得るため、必ず
//! **L/Rを2chで受けてこちら側で単純平均**する(モノラル素材はffmpegが自動で両chへ
//! 複製するので L==R となり、平均しても値は変わらない)。
//!
//! ## ストリーミングで畳む(全デコードを溜めない)
//! 総サンプル数は事前に `probe_container` の尺(container duration)× 48000 から
//! 見積もり、サンプル位置→バケット番号を `sample_index * buckets / total_samples`
//! で都度計算する。ffmpegのstdoutは固定サイズのチャンク([`READ_CHUNK_BYTES`])で
//! 読み、フレーム境界(8byte = f32×2ch)をまたいだ端数だけを次回読み出しへ持ち越す
//! (`pending`)。長尺素材でもメモリに乗るのはチャンクぶんだけ。
//! 実デコード長が見積もりとズレても(丸め・コンテナのpadding等)安全:
//! - 見積もりより**多い**サンプルは最終バケットへ吸収する(`.min(buckets-1)`)。
//! - 見積もりより**少ない**場合、触れられなかった末尾バケットは無音 `(0.0, 0.0)` に
//!   フォールバックする。
//!
//! ## キャッシュ
//! 同一 `path` + `buckets` の再計算を避けるプロセス内グローバルキャッシュを持つ。
//! - キー: `path` + ファイルの `len`/`mtime`(ファイルが差し替わったら自動的に
//!   別キーになり、古い結果を返さない — 明示的な invalidation APIは無い)
//! - 上限: [`WAVEFORM_CACHE_CAP`] 件(既定64)。超過時は最も長くtouchされていない
//!   1件を追い出す(真のLRU。`get`/`put`のたびに対象キーを使用順リストの末尾へ
//!   移動させる線形実装 — 上限が小さいので許容)。
//! - スコープ: プロセス内のみ。ディスク永続化はしない(next/engine層はUIの
//!   ライフタイムを知らないため、永続キャッシュの追い出しタイミングはUI/表示側の
//!   関心事として持ち越す)。

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use crate::probe::{probe_container, select_audio_stream};
use crate::{read_child_stderr, MediaError, Result};

/// waveform 抽出用に正規化するサンプルレート(Hz)。
/// [`crate::write_f32le_wav_stereo_48k`] と同じ正準値を再利用する。
pub const WAVEFORM_SAMPLE_RATE: u32 = 48_000;

/// キャッシュの最大保持件数(LRU追い出し)。
pub const WAVEFORM_CACHE_CAP: usize = 64;

/// interleaved stereo f32le の1フレーム(L+R)のバイト数。
const FRAME_BYTES: usize = 8;

/// ffmpeg stdout を読む1回ぶんのチャンクサイズ。`FRAME_BYTES`の倍数である必要は
/// ない(端数は`pending`へ持ち越す)が、システムコール回数を減らすため大きめにする。
const READ_CHUNK_BYTES: usize = 64 * 1024;

/// 指定音声(またはコンテナ内の先頭音声トラック)を `buckets` 個の (min, max) へ
/// 折り畳む。モノラル折り畳み・ストリーミング処理は module doc 参照。
///
/// `buckets == 0` は誤用としてエラーにする。戻り値の長さは常に `buckets`
/// (末尾が触れられなかった場合は無音 `(0.0, 0.0)` で埋める)。
pub fn waveform_peaks(path: impl AsRef<Path>, buckets: usize) -> Result<Vec<(f32, f32)>> {
    let path = path.as_ref();
    if buckets == 0 {
        return Err(MediaError::Probe(
            "waveform_peaks: buckets must be > 0".into(),
        ));
    }

    let key = cache_key(path, buckets)?;

    if let Some(hit) = cache().lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return Ok(hit);
    }

    let peaks = compute_waveform_peaks(path, buckets)?;

    cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .put(key, peaks.clone());

    Ok(peaks)
}

fn compute_waveform_peaks(path: &Path, buckets: usize) -> Result<Vec<(f32, f32)>> {
    #[cfg(test)]
    WAVEFORM_COMPUTE_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let container = probe_container(path)?;
    // 存在確認だけが目的(先頭音声トラック=ordinal 0)。実デコードは
    // ffmpegの `-map 0:a:0` に任せる — 二重にデコード経路を持たない。
    let _ = select_audio_stream(&container, 0)?;
    let duration = container
        .duration
        .ok_or_else(|| MediaError::Probe("waveform_peaks: unknown container duration".into()))?;

    let total_samples =
        ((duration.as_seconds_f64() * WAVEFORM_SAMPLE_RATE as f64).round() as u64).max(1);

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-v", "error", "-nostdin"]);
    cmd.arg("-i").arg(path);
    cmd.args([
        "-map",
        "0:a:0",
        "-vn",
        "-ac",
        "2",
        "-ar",
        &WAVEFORM_SAMPLE_RATE.to_string(),
        "-f",
        "f32le",
        "-",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => MediaError::ToolNotFound("ffmpeg"),
        _ => MediaError::Io(e),
    })?;

    let mut acc: Vec<(f32, f32)> = vec![(f32::INFINITY, f32::NEG_INFINITY); buckets];
    let mut sample_index: u64 = 0;
    let mut pending: Vec<u8> = Vec::with_capacity(FRAME_BYTES);

    {
        let stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| MediaError::Ffmpeg("waveform reader stdout not piped".into()))?;
        let mut buf = vec![0u8; READ_CHUNK_BYTES];
        loop {
            let n = match stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(MediaError::Io(e)),
            };
            let mut data = &buf[..n];

            if !pending.is_empty() {
                let need = FRAME_BYTES - pending.len();
                let take = need.min(data.len());
                pending.extend_from_slice(&data[..take]);
                data = &data[take..];
                if pending.len() < FRAME_BYTES {
                    continue;
                }
                fold_frame(&pending, &mut acc, &mut sample_index, total_samples, buckets);
                pending.clear();
            }

            let full_frames = data.len() / FRAME_BYTES;
            for frame in data[..full_frames * FRAME_BYTES].chunks_exact(FRAME_BYTES) {
                fold_frame(frame, &mut acc, &mut sample_index, total_samples, buckets);
            }
            let rem = &data[full_frames * FRAME_BYTES..];
            if !rem.is_empty() {
                pending.extend_from_slice(rem);
            }
        }
    }

    wait_ffmpeg_success(&mut child)?;

    // 見積もりより短く終わった場合、触れられなかった末尾バケットは無音扱い。
    for bucket in &mut acc {
        if bucket.0.is_infinite() {
            *bucket = (0.0, 0.0);
        }
    }

    Ok(acc)
}

#[inline]
fn fold_frame(
    frame: &[u8],
    acc: &mut [(f32, f32)],
    sample_index: &mut u64,
    total_samples: u64,
    buckets: usize,
) {
    debug_assert_eq!(frame.len(), FRAME_BYTES);
    let l = f32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]);
    let r = f32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]);
    let mono = (l + r) * 0.5;

    let idx = (*sample_index * buckets as u64) / total_samples;
    let idx = (idx as usize).min(buckets - 1);

    let entry = &mut acc[idx];
    if mono < entry.0 {
        entry.0 = mono;
    }
    if mono > entry.1 {
        entry.1 = mono;
    }
    *sample_index += 1;
}

fn wait_ffmpeg_success(child: &mut Child) -> Result<()> {
    let mut err = String::new();
    if let Some(stderr) = child.stderr.as_mut() {
        err = read_child_stderr(stderr)?;
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(MediaError::Ffmpeg(err));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// キャッシュ(同 path+buckets の再計算回避)
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    path: PathBuf,
    len: u64,
    mtime_nanos: u128,
    buckets: usize,
}

fn cache_key(path: &Path, buckets: usize) -> Result<CacheKey> {
    let metadata = std::fs::metadata(path)?;
    let mtime_nanos = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    Ok(CacheKey {
        path: path.to_path_buf(),
        len: metadata.len(),
        mtime_nanos,
        buckets,
    })
}

/// 単純なLRU: `order`の末尾ほど新しく使われた。`WAVEFORM_CACHE_CAP`超過時は
/// 先頭(=最も長く未使用)を1件だけ追い出す。
struct WaveformCache {
    entries: HashMap<CacheKey, Vec<(f32, f32)>>,
    order: Vec<CacheKey>,
}

impl WaveformCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: Vec::new(),
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let k = self.order.remove(pos);
            self.order.push(k);
        }
    }

    fn get(&mut self, key: &CacheKey) -> Option<Vec<(f32, f32)>> {
        let hit = self.entries.get(key).cloned();
        if hit.is_some() {
            self.touch(key);
        }
        hit
    }

    fn put(&mut self, key: CacheKey, value: Vec<(f32, f32)>) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), value);
            self.touch(&key);
            return;
        }
        if self.entries.len() >= WAVEFORM_CACHE_CAP && !self.order.is_empty() {
            let evict = self.order.remove(0);
            self.entries.remove(&evict);
        }
        self.order.push(key.clone());
        self.entries.insert(key, value);
    }
}

fn cache() -> &'static Mutex<WaveformCache> {
    static CACHE: OnceLock<Mutex<WaveformCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(WaveformCache::new()))
}

#[cfg(test)]
pub(crate) static WAVEFORM_COMPUTE_CALLS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
mod tests {
    use super::*;

    fn write_wav(path: &Path, interleaved: &[f32]) {
        crate::write_f32le_wav_stereo_48k(path, interleaved).unwrap();
    }

    fn silent_wav(path: &Path, seconds: f64) {
        let frames = (seconds * WAVEFORM_SAMPLE_RATE as f64) as usize;
        let interleaved = vec![0.0f32; frames * 2];
        write_wav(path, &interleaved);
    }

    fn sine_wav(path: &Path, seconds: f64, freq_hz: f64, amplitude: f32) {
        let frames = (seconds * WAVEFORM_SAMPLE_RATE as f64) as usize;
        let mut interleaved = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            let t = i as f64 / WAVEFORM_SAMPLE_RATE as f64;
            let s = (amplitude as f64 * (2.0 * std::f64::consts::PI * freq_hz * t).sin()) as f32;
            interleaved.push(s);
            interleaved.push(s);
        }
        write_wav(path, &interleaved);
    }

    /// 1サンプルだけ非ゼロの「クリック」。位置は `at_frame`。
    fn click_wav(path: &Path, seconds: f64, at_frame: usize, amplitude: f32) {
        let frames = (seconds * WAVEFORM_SAMPLE_RATE as f64) as usize;
        let mut interleaved = vec![0.0f32; frames * 2];
        interleaved[at_frame * 2] = amplitude;
        interleaved[at_frame * 2 + 1] = amplitude;
        write_wav(path, &interleaved);
    }

    #[test]
    fn buckets_zero_is_rejected() {
        let err = waveform_peaks("missing.wav", 0).unwrap_err();
        assert!(matches!(err, MediaError::Probe(_)));
    }

    #[test]
    fn silence_yields_all_zero_peaks_with_requested_bucket_count() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("waveform-silence");
        let path = dir.join("silence.wav");
        silent_wav(&path, 2.0);

        let peaks = waveform_peaks(&path, 40).unwrap();
        assert_eq!(peaks.len(), 40, "bucket count must match request exactly");
        for (i, (min, max)) in peaks.iter().enumerate() {
            assert!(
                min.abs() < 1e-4 && max.abs() < 1e-4,
                "bucket {i}: expected silence, got ({min}, {max})"
            );
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sine_peaks_are_symmetric_and_near_amplitude() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("waveform-sine");
        let path = dir.join("sine.wav");
        // 100Hz・2秒・振幅0.8。bucket幅0.1s(20buckets)なら1bucketに10周期入り、
        // 端の位相効果を吸収できる。
        sine_wav(&path, 2.0, 100.0, 0.8);

        let peaks = waveform_peaks(&path, 20).unwrap();
        assert_eq!(peaks.len(), 20);
        for (i, (min, max)) in peaks.iter().enumerate() {
            // 対称性: ゼロ平均の正弦波はどのbucketでも min ≈ -max。
            assert!(
                (min + max).abs() < 0.05,
                "bucket {i}: expected symmetric peaks around 0, got min={min} max={max}"
            );
            // 振幅に近い値が出ていること(丸め・端数の余地で少し緩める)。
            assert!(
                (max - 0.8).abs() < 0.05,
                "bucket {i}: expected max near 0.8, got {max}"
            );
            assert!(
                (min + 0.8).abs() < 0.05,
                "bucket {i}: expected min near -0.8, got {min}"
            );
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn click_position_maps_to_monotonic_time_bucket() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("waveform-click");
        let seconds = 4.0;
        let total_frames = (seconds * WAVEFORM_SAMPLE_RATE as f64) as usize;
        let buckets = 8usize;

        // クリックをbucket境界の内側に置く(境界ちょうどは丸めで隣接bucketへ
        // ずれ得るため、各bucketの中央に置いて一意に判定できるようにする)。
        let mut prev_idx: Option<usize> = None;
        for b in 0..buckets {
            let center_frac = (b as f64 + 0.5) / buckets as f64;
            let at_frame = ((center_frac * total_frames as f64) as usize).min(total_frames - 1);

            let path = dir.join(format!("click_{b}.wav"));
            click_wav(&path, seconds, at_frame, 0.9);

            let peaks = waveform_peaks(&path, buckets).unwrap();
            assert_eq!(peaks.len(), buckets);

            let hit_idx = peaks
                .iter()
                .position(|(min, max)| max.abs() > 0.5 || min.abs() > 0.5)
                .unwrap_or_else(|| panic!("no bucket registered the click for b={b}"));
            assert_eq!(
                hit_idx, b,
                "click placed at bucket {b} (frame {at_frame}) was recorded in bucket {hit_idx}"
            );

            // 時間軸の単調性: 前より後ろに置いたクリックは、検出bucketも
            // 後退しない(同値以上)。
            if let Some(prev) = prev_idx {
                assert!(hit_idx >= prev, "bucket index must not decrease with time");
            }
            prev_idx = Some(hit_idx);

            // 他のbucketは無音のまま(クリック1点だけが局在すること)。
            for (i, (min, max)) in peaks.iter().enumerate() {
                if i == hit_idx {
                    continue;
                }
                assert!(
                    min.abs() < 1e-4 && max.abs() < 1e-4,
                    "bucket {i} should stay silent, got ({min}, {max})"
                );
            }
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn repeated_calls_hit_the_cache() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("waveform-cache");
        let path = dir.join("cache.wav");
        silent_wav(&path, 0.5);

        let before = WAVEFORM_COMPUTE_CALLS.load(std::sync::atomic::Ordering::SeqCst);
        let first = waveform_peaks(&path, 10).unwrap();
        let after_first = WAVEFORM_COMPUTE_CALLS.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(after_first, before + 1, "first call must compute");

        let second = waveform_peaks(&path, 10).unwrap();
        let after_second = WAVEFORM_COMPUTE_CALLS.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(after_second, after_first, "second call must hit cache (no recompute)");
        assert_eq!(first, second);

        // buckets を変えるとキーが変わり再計算される。
        let _ = waveform_peaks(&path, 11).unwrap();
        let after_third = WAVEFORM_COMPUTE_CALLS.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(
            after_third,
            after_second + 1,
            "different bucket count must not hit the same cache entry"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cache_evicts_least_recently_used_beyond_cap() {
        let mut cache = WaveformCache::new();
        let make_key = |n: usize| CacheKey {
            path: PathBuf::from(format!("f{n}.wav")),
            len: 1,
            mtime_nanos: 0,
            buckets: 10,
        };

        for n in 0..WAVEFORM_CACHE_CAP {
            cache.put(make_key(n), vec![(0.0, 0.0)]);
        }
        assert_eq!(cache.entries.len(), WAVEFORM_CACHE_CAP);

        // key 0 を触れておく(LRU末尾へ移動 → 追い出し対象から外れる)。
        assert!(cache.get(&make_key(0)).is_some());

        // 新規キーを1件入れると、直近未使用の key 1 が追い出される。
        cache.put(make_key(WAVEFORM_CACHE_CAP), vec![(0.0, 0.0)]);
        assert_eq!(cache.entries.len(), WAVEFORM_CACHE_CAP);
        assert!(cache.entries.contains_key(&make_key(0)), "recently touched entry survives");
        assert!(
            !cache.entries.contains_key(&make_key(1)),
            "least-recently-used entry must be evicted"
        );
        assert!(cache.entries.contains_key(&make_key(WAVEFORM_CACHE_CAP)));
    }
}
