// Superellipse: Canvalry's Superellipse Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill ff3366
// set size 0.03
// set copies 1000
// say copy_fill ff3366 ffd166 3a86ff
// --track@n0:Width a,0,900,300,1.8
// --track@n1:Height b,0,900,300,1.8
// --track@n2:Exponent n,0,12,4,0.024
// --track@n3:Point Count,1,1000,500,1

/*
  Superellipse Distribution (Squircle / Lame Curve)
  Distributes points along a superellipse boundary: |x/a|^n + |y/b|^n = 1.

  n=2 is an ellipse, n=4 is a squircle, n=1 is a diamond,
  large n approaches a rectangle.

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Width a              (default: 300)
    n1  Height b             (default: 300)
    n2  Exponent n           (default: 4)   the shape parameter
    n3  Point Count          (default: 500)

  Presets: n=1 (diamond), n=2 (ellipse), n=4 (squircle),
           n=0.5 (astroid-like), n=20 (near-rectangle)
*/

function superellipseDistribution() {
    var a = (typeof n0 !== "undefined") ? n0 : 300;
    var b = (typeof n1 !== "undefined") ? n1 : 300;
    var exp = (typeof n2 !== "undefined") ? n2 : 4;
    var count = (typeof n3 !== "undefined") ? Math.max(Math.round(n3), 1) : 500;

    var cloud = new cavalry.PointCloud();

    if (Math.abs(exp) < 0.01) { exp = 0.01; }
    var invExp = 2 / exp;

    for (var i = 0; i < count; i += 1) {
        var t = 2 * Math.PI * i / count;
        var cosT = Math.cos(t);
        var sinT = Math.sin(t);

        var signX = cosT >= 0 ? 1 : -1;
        var signY = sinT >= 0 ? 1 : -1;

        var px = signX * a * Math.pow(Math.abs(cosT), invExp);
        var py = signY * b * Math.pow(Math.abs(sinT), invExp);

        cloud.appendPoint(new cavalry.Point(px, py));
    }

    return cloud;
}

superellipseDistribution();
