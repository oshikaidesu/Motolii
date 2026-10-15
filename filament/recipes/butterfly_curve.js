// Butterfly Curve: Canvalry's Butterfly Curve Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill 6437ff
// set size 0.03
// set copies 1000
// say copy_fill 6437ff ff24e0 ffffff
// --track@n0:Scale,0,300,100,0.6
// --track@n1:Point Count,1,1000,1000,1

/*
  Butterfly Curve Distribution (Temple Fay)
  Distributes points along the Temple Fay butterfly curve in polar form:
    r = e^sin(t) - 2*cos(4t) + sin((2t - PI) / 24)^5
    x = r * sin(t) * scale
    y = -r * cos(t) * scale

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Scale                (default: 100)
    n1  Point Count          (default: 1000)
*/

function butterflyCurveDistribution() {
    var scale = (typeof n0 !== "undefined") ? n0 : 100;
    var count = (typeof n1 !== "undefined") ? Math.max(Math.round(n1), 1) : 1000;

    var cloud = new cavalry.PointCloud();
    var PI = Math.PI;

    for (var i = 0; i < count; i += 1) {
        var t = 2 * PI * i / count;

        var sinT5 = Math.sin((2 * t - PI) / 24);
        var r = Math.exp(Math.sin(t)) - 2 * Math.cos(4 * t) + sinT5 * sinT5 * sinT5 * sinT5 * sinT5;

        var px = r * Math.sin(t) * scale;
        var py = -r * Math.cos(t) * scale;

        cloud.appendPoint(new cavalry.Point(px, py));
    }

    return cloud;
}

butterflyCurveDistribution();
