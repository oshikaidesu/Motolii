// Poster: the picture cut again and again into a grid of unequal cells, each cell a square, a circle or a triangle in one of a few colours, cut afresh every so often
// make shape rect
// say fill ff3366
// set copies 64
// say copy_fill ff3366 ffd166 1b2fd8 ffffff
// say copy_fill_way cycle
// then shape circle
// say script this
// set copies 64
// say fill ffd166
// say copy_fill 1b2fd8 ffffff ff3366 ffd166
// say copy_fill_way cycle
// set js_which 1
// then shape polygon
// say script this
// set points 3
// set copies 64
// say fill 1b2fd8
// say copy_fill ffd166 ff3366 ffffff 1b2fd8
// say copy_fill_way cycle
// set js_which 2
// --track@cells:Cells,1,64,22,1
// --track@wide:Width,0.5,12,5.4,0.01
// --track@fill:Fill,0.1,1.2,0.86,0.01
// --track@seed:Seed,0,999,3,1
// --track@every:Afresh,0,8,1,0.01
// --track@which:Which,0,2,0,1

// The cutting (the same for all three of its things, from the seed): the picture a rectangle; again and again the
// largest cell cut across its longer side somewhere between a third and two thirds; then each cell a shape by a coin
// of three sides. Each thing draws the cells whose coin is its own (Which: 0 squares, 1 circles, 2 triangles), as large
// as the cell's shorter side allows, a triangle turned a quarter at random.
function cells(k, epoch) {
  const W = k.wide, H = (k.wide * 9) / 16, all = [{ x: -W / 2, y: -H / 2, w: W, h: H }];
  const seed = k.seed + epoch * 7919;
  for (let c = 1; c < Math.min(64, Math.round(k.cells)); c++) {
    let at = 0;
    for (let i = 1; i < all.length; i++) if (all[i].w * all[i].h > all[at].w * all[at].h) at = i;
    const a = all[at], cut = cavalry.random(1 / 3, 2 / 3, seed, c);
    if (a.w >= a.h) all.splice(at, 1, { x: a.x, y: a.y, w: a.w * cut, h: a.h }, { x: a.x + a.w * cut, y: a.y, w: a.w * (1 - cut), h: a.h });
    else all.splice(at, 1, { x: a.x, y: a.y, w: a.w, h: a.h * cut }, { x: a.x, y: a.y + a.h * cut, w: a.w, h: a.h * (1 - cut) });
  }
  return all;
}
let kept = null, keptKey = '';
function each(i, n, t, k) {
  const epoch = k.every > 0 ? Math.floor(t / k.every) : 0, key = [k.cells, k.wide, k.seed, epoch].join(' ');
  if (key !== keptKey) (kept = cells(k, epoch)), (keptKey = key);
  const c = kept[i];
  const seed = k.seed + epoch * 7919;
  if (!c || Math.floor(cavalry.random(0, 3, seed + 1, i)) !== Math.round(k.which)) return { size: 0 };
  const side = Math.min(c.w, c.h) * k.fill;
  return { x: c.x + c.w / 2, y: c.y + c.h / 2, size: side, rotate: k.which == 2 ? 90 * Math.floor(cavalry.random(0, 4, seed + 2, i)) : 0 };
}
