// Lorenz Attractor: Canvalry's Lorenz Attractor Distribution as it is (MIT, Phill T, github.com/phillip-motion/Canvalry-scripts), only these head lines added (filament/recipes/from_canvalry.py)
// make shape circle
// say fill ffbe0b
// set size 0.03
// set copies 1000
// say copy_fill ffbe0b fb5607 ff006e
// key js_n7 0 0
// key js_n7 8 360
// --track@n0:Scale,0,30,10,0.06
// --track@n1:Iterations,0,6000,2000,12.0
// --track@n2:Step Size dt,0,1,0.005,0.002
// --track@n3:Sigma,0,30,10,0.06
// --track@n4:Rho,0,84,28,0.168
// --track@n5:Beta,0,8.001,2.667,0.016
// --track@n6:Rotation X,-360,360,0,1
// --track@n7:Rotation Y,-360,360,0,1
// --track@n8:Rotation Z,-360,360,0,1
// --track@n9:Perspective,0,3000,800,6.0

/*
  Lorenz Attractor Distribution
  Iterates the Lorenz system of differential equations to produce the iconic
  butterfly-shaped strange attractor. 3D with rotation and perspective.

  dx/dt = sigma * (y - x)
  dy/dt = x * (rho - z) - y
  dz/dt = x * y - beta * z

  Connect this JavaScript Utility to a Duplicator's Custom Distribution.

  Node attributes (add via the + button on the JavaScript tab):
    n0  Scale                (default: 10)
    n1  Iterations           (default: 2000)  number of points
    n2  Step Size dt         (default: 0.005)
    n3  Sigma                (default: 10)    rate of rotation
    n4  Rho                  (default: 28)    drives chaotic behaviour
    n5  Beta                 (default: 2.667) damping
    n6  Rotation X degrees   (default: 0)
    n7  Rotation Y degrees   (default: 0)
    n8  Rotation Z degrees   (default: 0)
    n9  Perspective           (default: 800, set to 0 for orthographic)

  The classic parameters (sigma=10, rho=28, beta=8/3) produce the
  butterfly shape. Try rho=99.96 for a more complex attractor, or
  sigma=14 for a tighter spiral.
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

function lorenzDistribution() {
    var scale = (typeof n0 !== "undefined") ? n0 : 10;
    var iterations = (typeof n1 !== "undefined") ? Math.max(Math.round(n1), 1) : 2000;
    var dt = (typeof n2 !== "undefined") ? n2 : 0.005;
    var sigma = (typeof n3 !== "undefined") ? n3 : 10;
    var rho = (typeof n4 !== "undefined") ? n4 : 28;
    var beta = (typeof n5 !== "undefined") ? n5 : 2.667;
    var rx = (typeof n6 !== "undefined") ? n6 * Math.PI / 180 : 0;
    var ry = (typeof n7 !== "undefined") ? n7 * Math.PI / 180 : 0;
    var rz = (typeof n8 !== "undefined") ? n8 * Math.PI / 180 : 0;
    var perspective = (typeof n9 !== "undefined") ? n9 : 800;

    var cloud = new cavalry.PointCloud();

    var lx = 0.1, ly = 0, lz = 0;

    for (var i = 0; i < iterations; i += 1) {
        var dx = sigma * (ly - lx);
        var dy = lx * (rho - lz) - ly;
        var dz = lx * ly - beta * lz;

        lx += dx * dt;
        ly += dy * dt;
        lz += dz * dt;

        var px = lx * scale;
        var py = ly * scale;
        var pz = (lz - rho) * scale;

        var rotated = rotatePoint3D(px, py, pz, rx, ry, rz);
        var projected = projectPoint(rotated.x, rotated.y, rotated.z, perspective);
        cloud.appendPoint(new cavalry.Point(projected.x, projected.y));
        cloud.appendSize(new cavalry.Point(projected.scale, projected.scale));
    }

    return cloud;
}

lorenzDistribution();
