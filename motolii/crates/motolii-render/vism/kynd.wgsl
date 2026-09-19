// 関数の棚(module `package::kynd`): kynd の MotionToolKit(The Book of Shaders のギャラリー、GLSL 13 本)の芯。
// 時刻の窓 → ease → mix、行きと帰りの引き算、拍の矩形。名前は出典のまま。出典: docs/reviews/2026-09-18-motion-code-survey.md

/// 時刻 t が begin..end のどこか(0..1、外は 0 か 1)。smoothstep の直線版。
fn linearstep(begin: f32, end: f32, t: f32) -> f32 {
    return clamp((t - begin) / max(end - begin, 1e-6), 0.0, 1.0);
}

/// 上がって(upBegin..upEnd)下がる(downBegin..downEnd)。行きと帰りの引き算。
fn linearstepUpDown(upBegin: f32, upEnd: f32, downBegin: f32, downEnd: f32, t: f32) -> f32 {
    return linearstep(upBegin, upEnd, t) - linearstep(downBegin, downEnd, t);
}

/// begin..end の間だけ 1(拍の矩形)。
fn stepUpDown(begin: f32, end: f32, t: f32) -> f32 {
    return step(begin, t) - step(end, t);
}

/// 時計回りに拭く: 中心からの角度が t(0..1)より小さい所が 1。
fn clockWipe(p: vec2f, t: f32) -> f32 {
    let a = atan2(p.x, -p.y) / 6.2831853 + 0.5;
    return step(a, t);
}
