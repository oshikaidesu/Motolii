// Sunflower: dots laid a golden angle apart, further out as the root of their number, slowly turning, a pulse running outwards
// make shape circle
// say fill ffd166
// set size 0.04
// set copies 800
// say copy_fill ffd166 ff3366 1b2fd8
// --track@spread:Spread,0,5,1.3,0.01
// --track@grow:Grow,0,3,1,0.01
// --track@turn:Turn,-2,2,0.3,0.01
// --check@pulse:Pulse,1
// --select@order:Order,in,out

// A sunflower: each copy a golden angle round from the one before, further out as the square root of its number
// (Vogel's model, as Canvalry's Sunflower lays a Duplicator's points), slowly turning, a pulse running outwards.
const golden = Math.PI * (3 - Math.sqrt(5));

function each(i, n, t, k) {
  const j = k.order === 'out' ? n - 1 - i : i;
  const r = k.spread * Math.sqrt((j + 0.5) / n);
  const a = j * golden + t * k.turn;
  const pulse = k.pulse ? 1 + 0.5 * Math.sin(t * 3 - r * 4) : 1;
  return { x: r * Math.cos(a), y: r * Math.sin(a), size: k.grow * pulse * (0.4 + r), rotate: (a * 180) / Math.PI };
}
