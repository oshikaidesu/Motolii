// A small piece with a real palette: the colours the Colors shelf finds "used here".
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#101114" });
const colours = ["#F2EDE4", "#E4572E", "#F3B61F", "#2E5EAA", "#76B041", "#8E6C8A", "#161616", "#FF7AA2", "#3BCEAC", "#0EAD69"];
colours.forEach((c, i) => (i % 3 === 0 ? ellipse() : rectangle()).name(`Swatch ${i + 1}`).fill(c));
