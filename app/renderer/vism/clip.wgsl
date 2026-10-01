/*{
  "ID": "motolii.clip",
  "LABEL": "Clip",
  "STAGE": "clip",
  "DESCRIPTION": "世界の平面で切る。層の中心から、層の軸の向きに Offset px の所。板(2D)・点群・網が同じ平面に従う",
  "INPUTS": [
    { "NAME": "axis", "LABEL": "Axis", "TYPE": "long", "DEFAULT": 0, "LABELS": ["+X", "-X", "+Y", "-Y", "+Z", "-Z"] },
    { "NAME": "offset", "LABEL": "Offset", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TRANSLATION" },
    { "NAME": "cap", "LABEL": "Cap", "TYPE": "bool", "DEFAULT": 1 }
  ]
}*/

// shader は無い。切る式は fork の 1 箇所(re_renderer shader/utils/clip.wgsl)にあり、
// 板・点群・網がそれを読む。この file は棚の札と欄の宣言。
