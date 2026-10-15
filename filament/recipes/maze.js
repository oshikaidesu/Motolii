// Maze: a grid of diagonals each slanted one way or the other at random (10 PRINT, a copy turned 0 or 90 degrees), drawn afresh every so often
// make shape line
// say fill ffffff
// set copies 1000
// say copy_fill 6437ff ff24e0
// --track@cells:Cells,4,40,32,1
// --track@spread:Spread,0.5,12,5.6,0.01
// --track@seed:Seed,0,999,1,1
// --track@every:Afresh,0,4,0.5,0.01

// Each copy a cell of a square grid, as many across as Cells and as many down as fill the picture (16 to 9) or the copies; a line as long as the cell's
// diagonal, slanted / or \ by a coin the seed and its number toss, tossed again every Afresh seconds (0: never).
function each(i, n, t, k) {
  const cols = Math.max(1, Math.round(k.cells)), rows = Math.min(Math.floor(n / cols), Math.round(cols * 9 / 16)), cell = k.spread / cols;
  if (i >= cols * rows) return { size: 0 };
  const c = i % cols, r = Math.floor(i / cols);
  const epoch = k.every > 0 ? Math.floor(t / k.every) : 0;
  const flip = cavalry.random(0, 1, k.seed + epoch * 7919, i) < 0.5;
  return { x: (c - (cols - 1) / 2) * cell, y: ((rows - 1) / 2 - r) * cell, size: cell * Math.SQRT2, rotate: flip ? 45 : 135 };
}
