// Honeycomb: Canvalry's Honeycomb Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill ff3366
// set size 0.03
// set copies 1000
// say copy_fill ff3366 ffd166 3a86ff
// --track@n0:Cell Size,0,150,50,0.3
// --track@n1:Columns,0,36,12,1
// --track@n2:Rows,0,30,10,1

/*
  Honeycomb Distribution (Hex Grid)
  Hexagonal grid where every other row is offset by half a cell width.
  Clean, geometric tessellation.

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Cell Size            (default: 50)   distance between centres
    n1  Columns              (default: 12)
    n2  Rows                 (default: 10)
*/

function honeycombDistribution() {
    var cellSize = (typeof n0 !== "undefined") ? n0 : 50;
    var cols = (typeof n1 !== "undefined") ? Math.max(Math.round(n1), 1) : 12;
    var rows = (typeof n2 !== "undefined") ? Math.max(Math.round(n2), 1) : 10;

    var cloud = new cavalry.PointCloud();

    var rowHeight = cellSize * Math.sqrt(3) / 2;
    var totalW = (cols - 1) * cellSize + cellSize / 2;
    var totalH = (rows - 1) * rowHeight;
    var offsetX = -totalW / 2;
    var offsetY = -totalH / 2;

    for (var row = 0; row < rows; row += 1) {
        var isOdd = row % 2 === 1;
        var shift = isOdd ? cellSize / 2 : 0;
        var py = offsetY + row * rowHeight;

        for (var col = 0; col < cols; col += 1) {
            var px = offsetX + col * cellSize + shift;
            cloud.appendPoint(new cavalry.Point(px, py));
        }
    }

    return cloud;
}

honeycombDistribution();
