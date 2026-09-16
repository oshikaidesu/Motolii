/*{
  "ID": "motolii.field",
  "LABEL": "Field",
  "STAGE": "block",
  "SCOPE": "room",
  "REACH": "reach",
  "PHYSICS": {
    "ROOM": "ancestor",
    "WALLS": "follow",
    "KEEP": "box",
    "TIME": "gather",
    "SUBSTEPS": 4,
    "ITERATIONS": 12,
    "SOFT_CCD": 2.0,
    "CORRECTIVE": 60.0
  },
  "DESCRIPTION": "The layer becomes a place that pulls, swirls, pushes or blows everything else in the same box. Gravity, attractor, vortex and wind are one thing on two dials with no modes between them; the solver is Rapier",
  "INPUTS": [
    { "NAME": "turn", "LABEL": "Turn", "TYPE": "float", "DEFAULT": 0.0, "MIN": -180.0, "MAX": 180.0 },
    { "NAME": "spread", "LABEL": "Spread", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 90.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 400.0, "MIN": -100000.0, "MAX": 100000.0 },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100000.0 },
    { "NAME": "hold", "LABEL": "Hold", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "gather", "LABEL": "Gather", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/
// PHYSICS は嘘と解き手の欄で、世界(Rust)は部品だけを持つ: ROOM = 部屋は場のある一番近い先祖("ancestor")か
// 直の親("parent")。WALLS = 壁は箱に付いて行く("follow")か組んだ時のまま("fixed")。KEEP = 箱の外へ出た分を
// 戻す("box")か Rapier に任せる("none")。TIME = 欄 Gather で終わりから読める("gather")か前向きだけ("forward")。
// SUBSTEPS/ITERATIONS/SOFT_CCD/CORRECTIVE は Rapier の欄(1 コマの歩数・反復・予測接触 m・1 歩の押し戻し)。
// 場の欄は意図だけ。動かすのは外の解き手(Rapier): 一様な分は重力、元からの分は引き寄せ・
// 押し出し・渦、Hold は家(レイアウトの場所)へのばね。Gather は向き: 0 は構図から散る、1 は構図へ集まる
// (同じ解きを終わりから逆に読む — 終わりは構図、提案 2026-09-16)。ここの本体は棚に載せるための形だけで、
// 絵は engine/physics.rs が Rapier に訳して出す(提案 2026-09-16)。
fn block(k: u32, p: BlockParams) -> Offset {
    return NO_OFFSET;
}
