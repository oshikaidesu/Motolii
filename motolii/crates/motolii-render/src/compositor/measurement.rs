use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameMeasurement {
    pub total_us: u64,
    pub resolve_us: u64,
    pub text_us: u64,
    pub shape_us: u64,
    pub camera_us: u64,
    pub layer_build_us: u64,
    pub textures_us: u64,
    pub inputs_us: u64,
    pub accumulate_us: u64,
    pub outline_us: u64,
    pub finalize_us: u64,
    pub prepare_us: u64,
    pub submit_us: u64,
    pub wait_us: u64,
    pub readback_us: u64,
    pub final_submission_gpu_us: Option<f64>,
    pub gpu_status: &'static str,
}

impl FrameMeasurement {
    /// 1 コマの持ち主。**名前と順はここだけ** — 数える側も出す側もこの並びを読む。
    /// 合計 1 つにしない(`reference/owned-budget.tsv` の法): まとめると誰が食ったか言えなくなる。
    pub const OWNERS: [&'static str; 11] = [
        "resolve", "text", "shape", "camera", "layers",
        "textures", "inputs", "accumulate", "outline", "finalize",
        "other",
    ];

    /// `OWNERS` と同じ並びの取り分。最後の `other` は、割り当てた段の外に落ちた分
    /// (どの持ち主にも属していない時間が見えるように、引き算で残す)。
    pub fn owners(&self) -> [u64; 11] {
        let named = self.resolve_us + self.text_us + self.shape_us + self.camera_us
            + self.layer_build_us + self.textures_us + self.inputs_us
            + self.accumulate_us + self.outline_us + self.finalize_us;
        [
            self.resolve_us, self.text_us, self.shape_us, self.camera_us, self.layer_build_us,
            self.textures_us, self.inputs_us, self.accumulate_us, self.outline_us, self.finalize_us,
            self.total_us.saturating_sub(named),
        ]
    }

    /// 段の中に含まれている内訳の名前。再生中の `wait`・`readback` は **0 でなければ契約違反**。
    pub const INCLUDED: [&'static str; 3] = ["submit", "wait", "readback"];

    /// `INCLUDED` と同じ並びの取り分。
    pub fn included(&self) -> [u64; 3] {
        [self.submit_us, self.wait_us, self.readback_us]
    }
}
