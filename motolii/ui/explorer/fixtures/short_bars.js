// Very short layers (a few frames), where a bar is thinner than a grip.
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#101114" });
[1, 2, 3, 5, 8, 12, 20].forEach((frames, i) => {
  const l = ellipse().name(`${frames} frame${frames > 1 ? "s" : ""}`);
  l.time((10 + i * 17) / 30, frames / 30);
  if (frames > 4) l.keys("Opacity", [[(10 + i * 17) / 30, 0], [(10 + i * 17 + frames - 1) / 30, 1]]);
});
