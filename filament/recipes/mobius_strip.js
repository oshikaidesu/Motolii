// Mobius Strip: Canvalry's Mobius Strip Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill 8ecae6
// set size 0.03
// set copies 1000
// say copy_fill 8ecae6 219ebc ffb703
// key js_n4 0 0
// key js_n4 8 360
// --track@n0:Radius R,0,600,200,1.2
// --track@n1:Half-width w,0,180,60,0.36
// --track@n2:Point Count,1,1000,300,1
// --track@n3:Rotation X,-360,360,0,1
// --track@n4:Rotation Y,-360,360,0,1
// --track@n5:Rotation Z,-360,360,0,1
// --track@n6:Perspective,0,3000,800,6.0

/*
  Mobius Strip Distribution
  Distributes points on a Mobius strip surface -- a band with a half-twist.

  x = (R + v * cos(u/2)) * cos(u)
  y = (R + v * cos(u/2)) * sin(u)
  z = v * sin(u/2)

  With 3D rotation and CSS-style perspective projection.

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Radius R             (default: 200)
    n1  Half-width w         (default: 60)
    n2  Point Count          (default: 300)
    n3  Rotation X degrees   (default: 0)
    n4  Rotation Y degrees   (default: 0)
    n5  Rotation Z degrees   (default: 0)
    n6  Perspective           (default: 800, set to 0 for orthographic)
*/

function rotatePoint3D(x, y, z, rx, ry, rz) {
    var cosX = Math.cos(rx), sinX = Math.sin(rx);
    var y1 = y * cosX - z * sinX;
    var z1 = y * sinX + z * cosX;

    var cosY = Math.cos(ry), sinY = Math.sin(ry);
    var x2 = x * cosY + z1 * sinY;
    var z2 = -x * sinY + z1 * cosY;

    var cosZ = Math.cos(rz), sinZ = Math.sin(rz);
    var x3 = x2 * cosZ - y1 * sinZ;
    var y3 = x2 * sinZ + y1 * cosZ;

    return { x: x3, y: y3, z: z2 };
}

function projectPoint(x, y, z, perspective) {
    if (perspective > 0) {
        var scale = perspective / (perspective - z);
        if (scale <= 0) { scale = 0.001; }
        return { x: x * scale, y: y * scale, scale: scale };
    }
    return { x: x, y: y, scale: 1 };
}

function mobiusStripDistribution() {
    var R = (typeof n0 !== "undefined") ? n0 : 200;
    var halfW = (typeof n1 !== "undefined") ? n1 : 60;
    var count = (typeof n2 !== "undefined") ? Math.max(Math.round(n2), 1) : 300;
    var rx = (typeof n3 !== "undefined") ? n3 * Math.PI / 180 : 0;
    var ry = (typeof n4 !== "undefined") ? n4 * Math.PI / 180 : 0;
    var rz = (typeof n5 !== "undefined") ? n5 * Math.PI / 180 : 0;
    var perspective = (typeof n6 !== "undefined") ? n6 : 800;

    var cloud = new cavalry.PointCloud();

    var uSteps = Math.max(Math.round(Math.sqrt(count * 3)), 3);
    var vSteps = Math.max(Math.round(count / uSteps), 1);

    for (var i = 0; i < uSteps; i += 1) {
        var u = 2 * Math.PI * i / uSteps;
        var halfU = u / 2;
        var cosU = Math.cos(u);
        var sinU = Math.sin(u);
        var cosHalfU = Math.cos(halfU);
        var sinHalfU = Math.sin(halfU);

        for (var j = 0; j < vSteps; j += 1) {
            var v = -halfW + 2 * halfW * j / (vSteps - 1 || 1);

            var px = (R + v * cosHalfU) * cosU;
            var py = (R + v * cosHalfU) * sinU;
            var pz = v * sinHalfU;

            var rotated = rotatePoint3D(px, py, pz, rx, ry, rz);
            var projected = projectPoint(rotated.x, rotated.y, rotated.z, perspective);
            cloud.appendPoint(new cavalry.Point(projected.x, projected.y));
            cloud.appendSize(new cavalry.Point(projected.scale, projected.scale));
        }
    }

    return cloud;
}

mobiusStripDistribution();
