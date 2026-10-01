// Making way in depth: a field of small solid blocks on a tilted table. A big block comes down through the table
// and slides across; every block it reaches steps aside — up, out, around — by its own declared distance (Margin).
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#0c0d12" });

const colors = ["#F2EDE4", "#E4572E", "#F3B61F", "#2E5EAA", "#7FA37A"];
const blocks = [];
for (let row = 0; row < 5; row++) {
  for (let col = 0; col < 9; col++) {
    const b = roundedRectangle({ name: `Block ${row}-${col}` }).fill(colors[(row * 3 + col) % colors.length]);
    b.set("Scale", [0.55, 0.55]).set("Depth", 110).set("Margin", 10)
      .set("Position", [(col - 4) * 165, (row - 2) * 165]).set("Position Z", 0)
      .set("Transition Duration", 0.5).set("Transition Easing", "Ease In Out");
    blocks.push(b);
  }
}

const big = roundedRectangle({ name: "Big" }).fill("#E4572E");
big.set("Scale", [1.7, 1.7]).set("Depth", 340).set("Margin", 16).set("Flex Shrink", 0);
big.keys("Position", [[0, [0, 0], "Hold"], [2.6, [0, 0], "Bezier"], [4.6, [600, -130], "Bezier"], [6, [600, -130]]]);
big.keys("Position Z", [[0, -1400, "Bezier"], [2.2, -60, "Hold"]]);
big.keys("Rotation", [[2.4, 0, "Bezier"], [4.8, 90]]);

const table = group(...blocks, big).name("Table");
table.set("Position", [900, 560]).set("Scale", [0.78, 0.78]).set("Tilt X", 50).set("Rotation", -14);
for (const layer of [table, ...blocks, big]) layer.projection("3D");
