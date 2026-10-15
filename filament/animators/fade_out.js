// Fade Out: each letter goes clear, one after another, to be gone as the thing ends
// (Cavalry's Animator of that name, as its shape is: a value of each copy from one number to another over a length, a way, a delay across them)
// --track@length:Length,0.05,4,0.67,0.01
// --track@delay:Delay,0,4,0.33,0.01

function each(i, n, t, k) { return animate(i, n, t, k, { what: 'opacity', from: 1, to: 0, way: 'SlowOut', out: true }); }
