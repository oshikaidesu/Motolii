/*{
  "ID": "motolii.field",
  "LABEL": "Field",
  "STAGE": "block",
  "SCOPE": "room",
  "REACH": "reach",
  "DESCRIPTION": "The layer becomes a place that pulls, swirls, pushes or blows everything else in the same box. Gravity, attractor, vortex and wind are one thing on two dials with no modes between them; the solver is Rapier",
  "INPUTS": [
    { "NAME": "turn", "LABEL": "Turn", "TYPE": "float", "DEFAULT": 0.0, "MIN": -180.0, "MAX": 180.0 },
    { "NAME": "spread", "LABEL": "Spread", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 90.0, "MIN": -360.0, "MAX": 360.0 },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 400.0, "MIN": -100000.0, "MAX": 100000.0 },
    { "NAME": "reach", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 100000.0 },
    { "NAME": "hold", "LABEL": "Hold", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/
// 場の欄は意図だけ。動かすのは外の解き手(Rapier): 一様な分は重力、元からの分は引き寄せ・
// 押し出し・渦、Hold は家(レイアウトの場所)へのばね。ここの本体は棚に載せるための形だけで、
// 絵は engine/physics.rs が Rapier に訳して出す(提案 2026-09-16)。
fn block(k: u32, p: BlockParams) -> Offset {
    return NO_OFFSET;
}
