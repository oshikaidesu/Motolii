// Lissajous: Canvalry's Lissajous Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill 00f5d4
// set size 0.03
// set copies 1000
// say copy_fill 00f5d4 9b5de5 f15bb5
// key js_n4 0 0
// key js_n4 8 360
// --track@n0:Width A,0,900,300,1.8
// --track@n1:Height B,0,900,300,1.8
// --track@n2:Frequency A,0,9,3,1
// --track@n3:Frequency B,0,6,2,1
// --track@n4:Phase delta,0,360,90,1
// --track@n5:Point Count,1,1000,500,1

/*
  Lissajous Distribution
  Distributes points along a Lissajous curve:
    x = A * sin(a * t + delta)
    y = B * sin(b * t)
  The frequency ratio a:b and phase delta produce figures-of-eight, knots,
  and complex woven patterns.

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Width A              (default: 300)
    n1  Height B             (default: 300)
    n2  Frequency A          (default: 3)
    n3  Frequency B          (default: 2)
    n4  Phase delta degrees  (default: 90)
    n5  Point Count          (default: 500)

  Try: A=3 B=2 delta=90 (figure-eight), A=3 B=4 delta=45 (woven knot),
       A=5 B=4 delta=90 (complex weave)
*/

function lissajousDistribution() {
    var ampA = (typeof n0 !== "undefined") ? n0 : 300;
    var ampB = (typeof n1 !== "undefined") ? n1 : 300;
    var freqA = (typeof n2 !== "undefined") ? n2 : 3;
    var freqB = (typeof n3 !== "undefined") ? n3 : 2;
    var delta = (typeof n4 !== "undefined") ? n4 * Math.PI / 180 : Math.PI / 2;
    var count = (typeof n5 !== "undefined") ? Math.max(Math.round(n5), 1) : 500;

    var cloud = new cavalry.PointCloud();

    for (var i = 0; i < count; i += 1) {
        var t = 2 * Math.PI * i / count;
        var px = ampA * Math.sin(freqA * t + delta);
        var py = ampB * Math.sin(freqB * t);

        cloud.appendPoint(new cavalry.Point(px, py));
    }

    return cloud;
}

lissajousDistribution();
