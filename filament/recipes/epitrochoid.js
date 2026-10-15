// Epitrochoid: Canvalry's Epitrochoid Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill 6437ff
// set size 0.03
// set copies 1000
// say copy_fill 6437ff ff24e0 ffffff
// --track@n0:Fixed Radius R,0,600,200,1.2
// --track@n1:Rolling Radius r,0,240,80,0.48
// --track@n2:Pen Distance d,0,240,80,0.48
// --track@n3:Revolutions,0,30,10,1
// --track@n4:Point Count,1,1000,1000,1

/*
  Epitrochoid Distribution
  A circle of radius r rolling outside a fixed circle of radius R, with
  the pen at distance d from the rolling circle's centre.

  x = (R + r) * cos(t) - d * cos((R + r) * t / r)
  y = (R + r) * sin(t) - d * sin((R + r) * t / r)

  This is the complement to the Hypotrochoid (inner rolling).

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Fixed Radius R       (default: 200)
    n1  Rolling Radius r     (default: 80)
    n2  Pen Distance d       (default: 80)
    n3  Revolutions          (default: 10)
    n4  Point Count          (default: 1000)

  Presets:
    Cardioid:    R=100, r=100, d=100  (R/r = 1)
    Nephroid:    R=100, r=50,  d=50   (R/r = 2)
    3-lobed:     R=200, r=67,  d=67   (R/r = 3)
*/

function epitrochoidDistribution() {
    var R = (typeof n0 !== "undefined") ? n0 : 200;
    var r = (typeof n1 !== "undefined") ? n1 : 80;
    var d = (typeof n2 !== "undefined") ? n2 : 80;
    var revolutions = (typeof n3 !== "undefined") ? Math.max(n3, 0.1) : 10;
    var count = (typeof n4 !== "undefined") ? Math.max(Math.round(n4), 1) : 1000;

    var cloud = new cavalry.PointCloud();

    if (Math.abs(r) < 0.001) { r = 0.001; }
    var totalAngle = revolutions * 2 * Math.PI;
    var sum = R + r;

    for (var i = 0; i < count; i += 1) {
        var t = totalAngle * i / count;

        var px = sum * Math.cos(t) - d * Math.cos(sum * t / r);
        var py = sum * Math.sin(t) - d * Math.sin(sum * t / r);

        cloud.appendPoint(new cavalry.Point(px, py));
    }

    return cloud;
}

epitrochoidDistribution();
