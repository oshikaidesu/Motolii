// The periodic table (three.js css3d, 2011) as one block: 118 symbols at one point, Formations puts them in a table,
// a circle, a helix, a grid, and walks through them on the clock. Two hands: the words, the block.
comp({ width: 1920, height: 1080, fps: 30, seconds: 17, background: "#0B0F12" });
const SYM = "H He Li Be B C N O F Ne Na Mg Al Si P S Cl Ar K Ca Sc Ti V Cr Mn Fe Co Ni Cu Zn Ga Ge As Se Br Kr Rb Sr Y Zr Nb Mo Tc Ru Rh Pd Ag Cd In Sn Sb Te I Xe Cs Ba La Ce Pr Nd Pm Sm Eu Gd Tb Dy Ho Er Tm Yb Lu Hf Ta W Re Os Ir Pt Au Hg Tl Pb Bi Po At Rn Fr Ra Ac Th Pa U Np Pu Am Cm Bk Cf Es Fm Md No Lr Rf Db Sg Bh Hs Mt Ds Rg Cn Nh Fl Mc Lv Ts Og".split(" ");
const C = ["#7FE7D8", "#F0ECE3", "#E8442E", "#E9C46A"];
const things = SYM.map((s, k) => text(s, { name: `E${k}`, Position: [960, 540] }).fill(C[k % 4]).font("Helvetica Neue").set("Scale", [0.34, 0.34]));
things.forEach((t) => t.effect("Formations", { "Hold": 2.6, "Travel": 1.4, "Cell": 84, "Radius": 430, "Stagger": 0.004 }));
text("table · circle · helix · grid — 118 words, one block, no keys", { Position: [960, 1040] }).fill("#F0ECE3").font("Helvetica Neue").set("Scale", [0.22, 0.22]);
