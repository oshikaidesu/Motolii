// Names longer than any column: English and Japanese, mixed kinds.
comp({ width: 1920, height: 1080, fps: 30, seconds: 10, background: "#101114" });
const names = [
  "An extremely long layer name that keeps going past any sensible column width",
  "Lower third / speaker name, role and organisation (final, approved v7)",
  "夜の公園で待ち合わせをしている二人の後ろを通り過ぎる自転車の影",
  "字幕 — 第三章「帰り道」の二行目、句読点と長音ー込み",
  "Background plate (graded, final v3) — do not move",
  "オープニングタイトル・ロゴアニメーション（差し替え版）",
];
names.forEach((n, i) => { const l = (i % 2 ? ellipse() : rectangle()).name(n); l.time(i * 0.3, 4 + i); });
