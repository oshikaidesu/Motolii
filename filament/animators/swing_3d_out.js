// 3D Swing Out: each letter tips over flat, one after another, as the thing ends
// (Cavalry's Animator of that name, as its shape is: a value of each copy from one number to another over a length, a way, a delay across them)
// --track@length:Length,0.05,4,0.67,0.01
// --track@delay:Delay,0,4,0.33,0.01

function each(i, n, t, k) { return animate(i, n, t, k, { what: 'rotate_x', from: 0, to: 90, way: 'VerySlowOut', out: true }); }
