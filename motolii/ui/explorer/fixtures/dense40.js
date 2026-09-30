// 40 layers for looking at the Timeline's density: long names, Japanese names, groups, keys, short and long bars.
comp({ width: 1920, height: 1080, fps: 30, seconds: 12, background: "#101114" });
const names = ["Title", "サブタイトル — 夜の公園で待ち合わせ", "Background plate (graded, final v3)", "Logo", "光の粒", "Lower third / speaker name and role",
  "Grain", "字幕 01", "字幕 02", "字幕 03", "Card A", "Card B", "Card C", "Vignette", "An extremely long layer name that keeps going past any sensible column width",
  "Star", "Ring", "ボタン", "Arrow", "Cursor"];
const make = [() => rectangle(), () => ellipse(), () => text("Motolii"), () => globalThis.star(), () => rectangle()];
const all = [];
for (let i = 0; i < 40; i++) {
  const l = make[i % make.length]();
  l.name(names[i % names.length] + (i >= names.length ? ` ${i}` : ""));
  const start = (i * 7) % 90, len = 20 + ((i * 37) % 300);
  l.time(start / 30, len / 30);
  if (i % 3 === 0) l.keys("Position", [[start / 30, [200 + i * 30, 300]], [(start + len / 2) / 30, [900, 500 + i]], [(start + len - 1) / 30, [1500, 700]]]);
  if (i % 4 === 1) l.keys("Opacity", [[start / 30, 0], [(start + 10) / 30, 1]]);
  all.push(l);
}
group(all[4], all[5], all[6]).name("Particles group");
