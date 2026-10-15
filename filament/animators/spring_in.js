// Spring In: each letter drops in from above the picture and springs to its place, one after another
// (Cavalry's Animator of that name, as its shape is: a value of each copy from one number to another over a length, a way, a delay across them)
// --track@length:Length,0.05,4,0.67,0.01
// --track@delay:Delay,0,4,0.33,0.01
// --track@far:Far,0,20,4,0.01

function each(i, n, t, k) { return animate(i, n, t, k, { what: 'y', from: k.far, to: 0, way: 'SpringOut' }); }
