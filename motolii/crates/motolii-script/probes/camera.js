// Probe A: filming techniques as plain JS over the existing vocabulary. Nothing here is a Host feature.
// A technique is a function of time returning camera rows; techniques stack by composing functions, then one bake writes keys.

const ease = (u) => (u < 0.5 ? 2 * u * u : 1 - 2 * (1 - u) * (1 - u)); // power2.inOut, evaluated in JS

/** Orbit + dolly around the target: yaw sweeps `degrees`, distance goes from -> to. */
const orbit = ({ seconds, degrees = 90, pitch = 12, from = 1, to = 0.7 }) => (t) => {
  const u = ease(Math.min(1, t / seconds));
  return { Orbit: [pitch, degrees * u], Distance: from + (to - from) * u };
};

/** Handheld: seeded wobble added on top of another technique's rows. `rate` is the wobble's own tempo. */
const handheld = (inner, { seed = 1, shake = 1 } = {}) => {
  const draw = random(seed);
  const wobble = () => draw() * 2 - 1;
  const memo = new Map();
  return (t) => {
    const key = Math.round(t * 1000);
    if (!memo.has(key)) memo.set(key, [wobble(), wobble(), wobble(), wobble(), wobble()]);
    const [a, b, c, d, e] = memo.get(key);
    const base = inner(t);
    const [pitch, yaw] = base.Orbit ?? [0, 0];
    return { ...base, Center: [a * 14 * shake, b * 10 * shake], Orbit: [pitch + c * 0.8 * shake, yaw + d * 1.1 * shake], Roll: e * 0.6 * shake };
  };
};

/** One bake: sample the path every 1/rate seconds and key every row it names. */
function bake(cam, path, { seconds, rate = 12 }) {
  for (let step = 0; step / rate <= seconds + 1e-9; step++) {
    const t = step / rate;
    for (const [row, value] of Object.entries(path(t))) cam.key(row, t, value);
  }
  return cam;
}

comp({ seconds: 4, background: "#101018" });
const cam = camera({ name: "Chase cam" });
const hero = rectangle({ name: "Hero" });
cam.set("Target", hero);
bake(cam, handheld(orbit({ seconds: 3, degrees: 120 }), { seed: 7 }), { seconds: 3, rate: 12 });
