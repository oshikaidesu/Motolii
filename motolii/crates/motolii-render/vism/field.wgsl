@description("The layer becomes a place that pulls, swirls, pushes or blows everything else in the same box. Gravity, attractor, vortex and wind are one thing on two dials with no modes between them; the solver is Rapier")
@scope("room")
@physics("ROOM", "ancestor")
@physics("WALLS", "follow")
@physics("KEEP", "box")
@physics("TIME", "gather")
@physics("SUBSTEPS", 4)
@physics("NUM_SOLVER_ITERATIONS", 12)
@physics("MAX_CCD_SUBSTEPS", 4)
@physics("NORMALIZED_ALLOWED_LINEAR_ERROR", 0.0005)
@physics("SOFT_CCD", 2.0)
@physics("CORRECTIVE", 60.0)

@label("Turn") @range(-180.0, 180.0)
override turn: f32 = 0.0;
@label("Spread") @range(0.0, 1.0)
override spread: f32 = 0.0;
@label("Angle") @range(-360.0, 360.0)
override angle: f32 = 90.0;
@label("Strength") @range(-100000.0, 100000.0)
override strength: f32 = 400.0;
@label("Reach") @range(0.0, 100000.0) @reach
override reach: f32 = 0.0;
@label("Hold") @range(0.0, 1.0)
override hold: f32 = 0.0;
@label("Gather") @range(0.0, 1.0)
override gather: f32 = 0.0;

// PHYSICS は嘘と解き手の欄で、世界(Rust)は部品だけを持つ: ROOM = 部屋は場のある一番近い先祖("ancestor")か
// 直の親("parent")。WALLS = 壁は箱に付いて行く("follow")か組んだ時のまま("fixed")。KEEP = 箱の外へ出た分を
// 戻す("box")か Rapier に任せる("none")。TIME = 欄 Gather で終わりから読める("gather")か前向きだけ("forward")。
// SUBSTEPS は 1 コマの歩数。NUM_SOLVER_ITERATIONS / MAX_CCD_SUBSTEPS / NORMALIZED_ALLOWED_LINEAR_ERROR は Rapier の
// IntegrationParameters と同じ名前(書かなければ Rapier の既定 4 / 1 / 0.001)。SOFT_CCD は予測接触 m、CORRECTIVE は 1 歩の押し戻し。
// 場の欄は意図だけ。動かすのは外の解き手(Rapier): 一様な分は重力、元からの分は引き寄せ・
// 押し出し・渦、Hold は家(レイアウトの場所)へのばね。Gather は向き: 0 は構図から散る、1 は構図へ集まる
// (同じ解きを終わりから逆に読む — 終わりは構図、提案 2026-09-16)。ここの本体は棚に載せるための形だけで、
// 絵は engine/physics.rs が Rapier に訳して出す(提案 2026-09-16)。
fn block(k: u32) -> Offset {
    return NO_OFFSET;
}
