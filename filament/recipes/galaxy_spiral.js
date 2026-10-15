// Galaxy Spiral: Canvalry's Galaxy Spiral Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill 00f5d4
// set size 0.03
// set copies 1000
// say copy_fill 00f5d4 9b5de5 f15bb5
// --track@n0:Scale,0,900,300,1.8
// --track@n1:Point Count,1,1000,500,1
// --track@n2:Arm Count,1,1000,3,1
// --track@n3:Tightness,0,1,0.3,0.002
// --track@n4:Scatter,0,90,30,0.18
// --track@n5:Seed,0,126,42,1

/*
  Galaxy Spiral Distribution
  Distributes points along logarithmic spiral arms with random scatter,
  producing a spiral galaxy or swirl pattern.

  Each arm follows r = a * e^(b * theta), with points randomly offset
  from the arm path for a natural look.

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Scale                (default: 300)
    n1  Point Count          (default: 500)
    n2  Arm Count            (default: 3)
    n3  Tightness            (default: 0.3)  how tightly wound the arms are
    n4  Scatter              (default: 30)   random offset from arm path
    n5  Seed                 (default: 42)
*/

function galaxySpiralDistribution() {
    var scale = (typeof n0 !== "undefined") ? n0 : 300;
    var count = (typeof n1 !== "undefined") ? Math.max(Math.round(n1), 1) : 500;
    var arms = (typeof n2 !== "undefined") ? Math.max(Math.round(n2), 1) : 3;
    var tightness = (typeof n3 !== "undefined") ? n3 : 0.3;
    var scatter = (typeof n4 !== "undefined") ? n4 : 30;
    var seed = (typeof n5 !== "undefined") ? n5 : 42;

    var cloud = new cavalry.PointCloud();
    var armAngle = 2 * Math.PI / arms;
    var perArm = Math.floor(count / arms);

    for (var arm = 0; arm < arms; arm += 1) {
        var baseAngle = arm * armAngle;

        for (var i = 0; i < perArm; i += 1) {
            var t = i / (perArm - 1 || 1);
            var theta = t * 4 * Math.PI;
            var r = scale * t;

            var spiralAngle = baseAngle + theta * tightness;

            var idx = arm * perArm + i;
            var scatterX = cavalry.random(-scatter, scatter, seed, idx * 2);
            var scatterY = cavalry.random(-scatter, scatter, seed, idx * 2 + 1);

            var px = r * Math.cos(spiralAngle) + scatterX;
            var py = r * Math.sin(spiralAngle) + scatterY;

            cloud.appendPoint(new cavalry.Point(px, py));
        }
    }

    return cloud;
}

galaxySpiralDistribution();
