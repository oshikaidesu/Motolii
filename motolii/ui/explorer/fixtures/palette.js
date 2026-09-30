// A small piece with a real palette: the colours the Colors shelf finds "used here".
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101114" });
const colours = ["#3155FF", "#FF4FA3", "#C7F000", "#7B3FF2", "#0C0D10", "#FF6A13", "#2EE6C8", "#FFE14D", "#C4CCD8", "#00C2FF"];
colours.forEach((c, i) => (i % 3 === 0 ? ellipse() : rectangle()).name(`Swatch ${i + 1}`).fill(c));
