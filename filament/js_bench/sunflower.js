// A recipe's script as a person or an LLM writes it: modern JavaScript, the copies' values from their number.
const golden = Math.PI * (3 - Math.sqrt(5));
class Flower {
  constructor({ count = 2000, spread = 2.2, size = 0.04 } = {}) { Object.assign(this, { count, spread, size }); }
  each(i, t) {
    const r = this.spread * Math.sqrt((i + 0.5) / this.count), a = i * golden + t * 0.6;
    const [x, y] = [r * Math.cos(a), r * Math.sin(a)];
    const grow = 1 + 0.5 * Math.sin(t * 3 - r * 4);
    return { x, y, size: this.size * grow * (0.4 + r), hue: (i * 0.618) % 1 };
  }
  all(t) {   // one call, every copy: flat numbers, four a copy
    const out = new Float32Array(this.count * 4);
    for (let i = 0; i < this.count; i++) { const c = this.each(i, t); out.set([c.x, c.y, c.size, c.hue], i * 4); }
    return out;
  }
}
globalThis.flower = new Flower();
globalThis.features = `${[1, 2].map((n) => n * 2).at(-1)}|${({ a: { b: 7 } }).a?.b ?? 0}|${[...'ab'].length}|${Object.entries({ k: 1 }).flat().join()}|${10n ** 3n}`;
