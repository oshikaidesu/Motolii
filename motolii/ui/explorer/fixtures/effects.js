// A layer with several effects (a long parameter list) for the Inspector.
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#101114" });
const l = rectangle().name("Effected card");
for (const e of ["Hue/Saturation", "Glow", "Blur", "Drop Shadow"]) { try { l.effect(e); } catch (_) {} }
