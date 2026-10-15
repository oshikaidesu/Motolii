// Sphere: Canvalry's Sphere Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill 8ecae6
// set size 0.03
// set copies 1000
// say copy_fill 8ecae6 219ebc ffb703
// key js_n3 0 0
// key js_n3 8 360
// --track@n0:Radius,0,900,300,1.8
// --track@n1:Point Count,1,1000,100,1
// --track@n2:Rotation X,-360,360,0,1
// --track@n3:Rotation Y,-360,360,0,1
// --track@n4:Rotation Z,-360,360,0,1
// --track@n5:Perspective,0,3000,800,6.0
// --check@n6:Cull Backfaces,0

/*
  Sphere Distribution
  Distributes points on the surface of a sphere using the Fibonacci/golden spiral
  algorithm, with 3D rotation, CSS-style perspective projection, and backface culling.

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Radius              (default: 300)
    n1  Point Count         (default: 100)
    n2  Rotation X degrees  (default: 0)
    n3  Rotation Y degrees  (default: 0)
    n4  Rotation Z degrees  (default: 0)
    n5  Perspective          (default: 800, set to 0 for orthographic)
    n6  Cull Backfaces       (0 = off, 1 = on)
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

function smoothstep(t) {
    if (t <= 0) { return 0; }
    if (t >= 1) { return 1; }
    return t * t * (3 - 2 * t);
}

function projectPoint(x, y, z, perspective) {
    if (perspective > 0) {
        var scale = perspective / (perspective - z);
        if (scale <= 0) { scale = 0.001; }
        return { x: x * scale, y: y * scale, scale: scale };
    }
    return { x: x, y: y, scale: 1 };
}

function sphereDistribution() {
    var radius = (typeof n0 !== "undefined") ? n0 : 300;
    var count = (typeof n1 !== "undefined") ? Math.max(Math.round(n1), 1) : 100;
    var rx = (typeof n2 !== "undefined") ? n2 * Math.PI / 180 : 0;
    var ry = (typeof n3 !== "undefined") ? n3 * Math.PI / 180 : 0;
    var rz = (typeof n4 !== "undefined") ? n4 * Math.PI / 180 : 0;
    var perspective = (typeof n5 !== "undefined") ? n5 : 800;
    var cullBackfaces = (typeof n6 !== "undefined") ? (n6 >= 1) : false;

    var cloud = new cavalry.PointCloud();
    var goldenAngle = Math.PI * (3 - Math.sqrt(5));

    for (var i = 0; i < count; i += 1) {
        var lat = 1 - (2 * i / (count - 1 || 1));
        var ringRadius = Math.sqrt(1 - lat * lat);
        var theta = goldenAngle * i;

        var px = Math.cos(theta) * ringRadius * radius;
        var py = lat * radius;
        var pz = Math.sin(theta) * ringRadius * radius;

        var rotated = rotatePoint3D(px, py, pz, rx, ry, rz);

        var cullFactor = 1;
        if (cullBackfaces) {
            var depth = rotated.z / (radius || 1);
            cullFactor = smoothstep(depth / 0.3);
        }

        var projected = projectPoint(rotated.x, rotated.y, rotated.z, perspective);
        var s = projected.scale * cullFactor;
        cloud.appendPoint(new cavalry.Point(projected.x, projected.y));
        cloud.appendSize(new cavalry.Point(s, s));
    }

    return cloud;
}

sphereDistribution();
