// A small piece with a real palette: the colours the Colors shelf finds "used here".
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101114" });
const colours = ["#F4F1EA", "#161616", "#D9C7A7", "#C8553D", "#E0A458", "#6B8F71", "#2F5D62", "#3C4F76", "#9A8FB3", "#D98E8E"];
colours.forEach((c, i) => (i % 3 === 0 ? ellipse() : rectangle()).name(`Swatch ${i + 1}`).fill(c));
