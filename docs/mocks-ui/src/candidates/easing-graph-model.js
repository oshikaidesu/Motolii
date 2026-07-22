// AM実機スクリーンショット34枚(IMG_4933-4966)からの幾何抽出に基づく
// 補間曲線モデル。toolkit非依存の純関数のみで、React描画とnode検証、
// 将来のegui custom paintが同じ写像を共有する。
// 出典: docs/reviews/2026-07-19-am-keyframe-graph-observation.md

export const PLOT = { width: 473, height: 499, boxLeft: 30, boxWidth: 413 };

export function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function roundTo(value, step) {
  const digits = step >= 1 ? 0 : 2;
  return Number((Math.round(value / step) * step).toFixed(digits));
}

export function snap(value, min, max, step) {
  return roundTo(clamp(value, min, max), step);
}

function fract(value) {
  return value - Math.floor(value);
}

function smoothstep(edge0, edge1, value) {
  const t = clamp((value - edge0) / (edge1 - edge0 || 1e-6), 0, 1);
  return t * t * (3 - 2 * t);
}

function mix(a, b, t) {
  return a + (b - a) * t;
}

const TAU = Math.PI * 2;

// curveやhandleの変更で座標写像を変えない。Overshoot ON時は最初から
// manual handleとElastic limitの全可動域を収める固定viewへ切り替える。
export const STANDARD_VIEW = Object.freeze({ top: 1.35, bottom: -0.35 });
export const OVERSHOOT_VIEW = Object.freeze({ top: 2.2, bottom: -0.5 });

export function viewForOvershoot(enabled) {
  return enabled ? OVERSHOOT_VIEW : STANDARD_VIEW;
}

export function xOf(u) {
  return PLOT.boxLeft + u * PLOT.boxWidth;
}

export function yOf(v, view) {
  return ((view.top - v) / (view.top - view.bottom)) * PLOT.height;
}

export function pointFrom(fractionX, fractionY, view) {
  return {
    u: (fractionX * PLOT.width - PLOT.boxLeft) / PLOT.boxWidth,
    v: view.top - fractionY * (view.top - view.bottom),
  };
}

// ---- Bounce ----------------------------------------------------------------
// 実機: 自己相似バウンド。振幅と持続の両方が1バウンドごとにd倍
// (弾道則sqrt(d)ではない)。立ち上がりは(u/T)^2、頂点はv=1に接するcusp、
// 谷は滑らかな放物線。弧が入り切らなくなったらv=1で平坦保持。
// handle=(a,h)は最初の谷の頂点そのもの。d=1-h、T=a/(1+d)。
export function bounceValue(parameters, u) {
  if (u >= 1) return 1;
  const d = clamp(1 - parameters.dip, 0.02, 1);
  const T = Math.max(parameters.firstDip / (1 + d), 0.02);
  if (u <= T) return (u / T) ** 2;
  let cusp = T;
  let scale = d;
  for (let k = 0; k < 24; k += 1) {
    const width = 2 * T * scale;
    if (cusp + width > 1) return 1;
    if (u <= cusp + width) {
      const local = (u - cusp - T * scale) / (T * scale);
      return 1 - scale * (1 - local * local);
    }
    cusp += width;
    scale *= d;
  }
  return 1;
}

// ---- Elastic ---------------------------------------------------------------
// 実機: v = 1 - (A-1)(1-u)^n cos(2πu/p)。クリップではなく振幅スケーリング。
// A=limit(点線の天井、handleはその右端)、p=波handleのx(最初の谷がそこに来る)、
// n=波handleの高さ(v_h=0→n=1、上げるほど早く減衰、曲線上=平坦)。
export function elasticValue(parameters, u) {
  if (u >= 1) return 1;
  const { limit, period, damp } = parameters;
  const n = damp >= 0.97 ? 400 : 1 / (1 - damp) ** 2;
  const closed = (limit - 1) * Math.pow(1 - u, n);
  const envelope = mix(1, closed, smoothstep(0, 0.45 * period, u));
  return 1 - envelope * Math.cos((TAU * u) / period);
}

// ---- Cyclic ----------------------------------------------------------------
// 実機: v = f + (1-f)·W(frac(u/T))、f=E·u(envelope線)。
// W: 頂点位相s(top guide上のhandle x = s·T)、平滑度c(白handle x = c·T、
// 0=cosine 1=linear)。谷はenvelope線に、頂はv=1に接する。
function cyclicWave(parameters, phi) {
  const s = clamp(parameters.peak, 0.02, 0.98);
  const tri = phi < s ? phi / s : (1 - phi) / (1 - s);
  const smooth =
    phi < s
      ? (1 - Math.cos((Math.PI * phi) / s)) / 2
      : (1 + Math.cos((Math.PI * (phi - s)) / (1 - s))) / 2;
  return mix(smooth, tri, parameters.linear);
}

export function cyclicValue(parameters, u) {
  if (u >= 1) return parameters.envelopeEnd + (1 - parameters.envelopeEnd) * cyclicWave(parameters, fract(1 / parameters.period));
  const floor = parameters.envelopeEnd * u;
  return floor + (1 - floor) * cyclicWave(parameters, fract(u / parameters.period));
}

// ---- Random ----------------------------------------------------------------
// 実機: v = u + amp·env·noise + bias·Ψ。noiseは滑らかなvalue noiseで
// 0..1ボックスへはクランプされない。seedは左端の縦scrub(再抽選のみ)、
// 粒度は上辺(左=最細)、自由handleはv=null線0.47からの距離が振幅・xが
// エネルギー中心、下辺handleは帯全体を上下へ押すbias。
function latticeNoise(seed, index) {
  const value = Math.sin((seed + index * 97.13) * 12.9898) * 43758.5453;
  return (value - Math.floor(value)) * 2 - 1;
}

function valueNoise(x, seed) {
  const index = Math.floor(x);
  const local = x - index;
  const a = latticeNoise(seed, index);
  const b = latticeNoise(seed, index + 1);
  return mix(a, b, (1 - Math.cos(Math.PI * local)) / 2);
}

export const RANDOM_NULL_LEVEL = 0.47;

export function randomValue(parameters, u) {
  if (u <= 0) return 0;
  if (u >= 1) return 1;
  const seed = Math.round(parameters.seed);
  const frequency = mix(40, 8, clamp(parameters.grain / 0.5, 0, 1));
  const noise =
    0.8 * valueNoise(u * frequency, seed) +
    0.2 * valueNoise(u * frequency * 2.7, seed + 37);
  const amplitude = clamp(
    1.7 * Math.abs(parameters.centerV - RANDOM_NULL_LEVEL),
    0,
    0.9,
  );
  const envelope = smoothstep(
    0,
    1,
    clamp(1 - Math.abs(u - parameters.centerU) / 0.55, 0, 1),
  );
  const fade = smoothstep(0, 0.05, u) * smoothstep(1, 0.95, u);
  const push = (0.5 - parameters.bias) *
    (1 - Math.exp(-u / 0.15)) *
    (1 - Math.exp(-(1 - u) / 0.15));
  return u + amplitude * envelope * fade * noise + push;
}

// ---- Steps -----------------------------------------------------------------
// 実機: 対角線のsample-and-hold量子化。白anchor(基線上)のx=段幅w、
// 黄satellite(上辺guide上)のanchorからの水平offset=平滑幅s。
// 平滑rampは段時刻t_kに「到着」する(easeが段に先行する)。
export function stepsValue(parameters, u) {
  if (u >= 1) return 1;
  const { width, smooth } = parameters;
  const index = Math.floor(u / width);
  const local = u - index * width;
  const rampStart = width - smooth;
  const progress =
    smooth <= 0.0001 ? 0 : smoothstep(0, 1, clamp((local - rampStart) / smooth, 0, 1));
  return clamp(width * (index + progress), 0, 1);
}

// ---- Elastic Steps ---------------------------------------------------------
// 実機: v = P·(k + S_E(τ))。黄handle(基線上)のx=P=段幅かつ段高。
// 白handle(左端)の高さE=弾性: E=0で滑らかなS、E→1で瞬間ジャンプ+
// 減衰リング(初回+36%、周期0.112τ、半周期ごと×0.72)。終端は(1,1)へのジャンプ。
export function elasticStepsValue(parameters, u) {
  if (u >= 1) return 1;
  const { width, elasticity } = parameters;
  // Stepsと異なり、遷移は段時刻kPで「開始」する(実機IMG_4965/4966)。
  if (u < width) return 0;
  const index = Math.floor(u / width) - 1;
  const tau = (u - (index + 1) * width) / width;
  const riseWidth = mix(0.45, 0.03, elasticity);
  let response = smoothstep(0, riseWidth, tau);
  if (tau > riseWidth && elasticity > 0.01) {
    const ringT = tau - riseWidth;
    const ringAmp = 0.36 * Math.pow(elasticity, 3.2);
    response += ringAmp * Math.exp(-5.9 * ringT) * Math.sin((TAU * ringT) / 0.112);
  }
  const predip = 0.045 * Math.sin(Math.PI * clamp(elasticity, 0, 1));
  response -= predip * smoothstep(0.78, 0.99, tau);
  return clamp(width * (index + response), -0.2, 1.35);
}

// ---- Handle仕様 ------------------------------------------------------------
// anchor(parameters)→{u,v}: 実機と同じ載り場所(曲線/基線/guide/limit線)。
// apply(parameters, point)→parameters: pointer点からの逆写像。
// kind: param=実機の黄handle、anchor=白handle。
export const ADVANCED_SPECS = {
  Bounce: {
    confirmed: true,
    evaluate: bounceValue,
    defaults: { firstDip: 0.27, dip: 0.2 },
    handles: [
      {
        id: 'first-dip',
        label: 'FIRST DIP',
        role: 'button',
        kind: 'param',
        params: [
          { key: 'firstDip', label: 'TIMING', axis: 'x', min: 0.06, max: 0.7, step: 0.01 },
          { key: 'dip', label: 'DEPTH', axis: 'y', min: 0, max: 0.9, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.firstDip, v: p.dip }),
        apply: (p, point) => ({
          ...p,
          firstDip: snap(point.u, 0.06, 0.7, 0.01),
          dip: snap(point.v, 0, 0.9, 0.01),
        }),
      },
    ],
    decorations: () => [],
  },
  Elastic: {
    confirmed: true,
    evaluate: elasticValue,
    // overshootが型の意味に含まれる型だけが0..1の外へ描ける。
    overshoots: true,
    defaults: { limit: 1.5, period: 0.3, damp: 0.35 },
    handles: [
      {
        id: 'overshoot-limit',
        label: 'OVERSHOOT LIMIT',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'limit', label: 'LIMIT', axis: 'y', min: 1.02, max: 2, step: 0.01 },
        ],
        anchor: (p) => ({ u: 0.97, v: p.limit }),
        apply: (p, point) => ({ ...p, limit: snap(point.v, 1.02, 2, 0.01) }),
      },
      {
        id: 'wave-size',
        label: 'WAVE SIZE',
        role: 'button',
        kind: 'param',
        params: [
          { key: 'period', label: 'PERIOD', axis: 'x', min: 0.12, max: 0.7, step: 0.01 },
          { key: 'damp', label: 'DAMPING', axis: 'y', min: 0, max: 0.97, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.period, v: p.damp }),
        apply: (p, point) => ({
          ...p,
          period: snap(point.u, 0.12, 0.7, 0.01),
          damp: snap(point.v, 0, 0.97, 0.01),
        }),
      },
    ],
    decorations: (p) => [
      { key: 'limit-line', className: 'advanced-limit', from: { u: 0, v: p.limit }, to: { u: 0.97, v: p.limit } },
      { key: 'wave-stem', className: 'advanced-stem', from: { u: p.period, v: p.damp }, to: { u: p.period, v: elasticValue(p, p.period) } },
    ],
  },
  Cyclic: {
    confirmed: true,
    evaluate: cyclicValue,
    defaults: { period: 2 / 7, peak: 0.5, linear: 0, envelopeEnd: 0 },
    handles: [
      {
        id: 'period',
        label: 'PERIOD',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'period', label: 'PERIOD', axis: 'x', min: 0.08, max: 0.95, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.period, v: 0 }),
        apply: (p, point) => ({ ...p, period: snap(point.u, 0.08, 0.95, 0.01) }),
      },
      {
        id: 'peak-position',
        label: 'PEAK POSITION',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'peak', label: 'PEAK', axis: 'x', min: 0.02, max: 0.98, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.peak * p.period, v: 1 }),
        apply: (p, point) => ({
          ...p,
          peak: snap(point.u / p.period, 0.02, 0.98, 0.01),
        }),
      },
      {
        id: 'smoothness',
        label: 'SMOOTHNESS',
        role: 'slider',
        kind: 'anchor',
        params: [
          { key: 'linear', label: 'COS → LINEAR', axis: 'x', min: 0, max: 1, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.linear * p.period, v: 1.5 }),
        apply: (p, point) => ({
          ...p,
          linear: snap(point.u / p.period, 0, 1, 0.01),
        }),
      },
      {
        id: 'lower-limit',
        label: 'LOWER LIMIT',
        role: 'slider',
        kind: 'anchor',
        params: [
          { key: 'envelopeEnd', label: 'LOWER LIMIT', axis: 'y', min: 0, max: 0.95, step: 0.01 },
        ],
        anchor: (p) => ({ u: 1, v: p.envelopeEnd }),
        apply: (p, point) => ({ ...p, envelopeEnd: snap(point.v, 0, 0.95, 0.01) }),
      },
    ],
    decorations: (p) => [
      { key: 'shape-stem', className: 'advanced-stem anchor-stem', from: { u: p.linear * p.period, v: 1.5 }, to: { u: p.peak * p.period, v: 1 } },
      { key: 'envelope', className: 'advanced-envelope', from: { u: 0, v: 0 }, to: { u: 1, v: p.envelopeEnd } },
    ],
  },
  Random: {
    confirmed: true,
    evaluate: randomValue,
    defaults: { seed: 500, grain: 0.15, centerU: 0.5, centerV: 0.75, bias: 0.5 },
    handles: [
      {
        id: 'seed',
        label: 'SEED',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'seed', label: 'SEED', axis: 'y', min: 1, max: 999, step: 1 },
        ],
        // 左端の縦scrub。曲線始点と混ざらないよう中段の帯(v 0.15..0.85)を使う。
        anchor: (p) => ({ u: 0.005, v: 0.15 + (p.seed / 999) * 0.7 }),
        apply: (p, point) => ({
          ...p,
          seed: snap(((point.v - 0.15) / 0.7) * 998 + 1, 1, 999, 1),
        }),
      },
      {
        id: 'size',
        label: 'SIZE',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'grain', label: 'GRAIN', axis: 'x', min: 0, max: 0.5, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.grain, v: 1 }),
        apply: (p, point) => ({ ...p, grain: snap(point.u, 0, 0.5, 0.01) }),
      },
      {
        id: 'height',
        label: 'HEIGHT',
        role: 'button',
        kind: 'param',
        params: [
          { key: 'centerU', label: 'CENTER', axis: 'x', min: 0.05, max: 0.95, step: 0.01 },
          { key: 'centerV', label: 'HEIGHT', axis: 'y', min: -0.2, max: 1.2, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.centerU, v: p.centerV }),
        apply: (p, point) => ({
          ...p,
          centerU: snap(point.u, 0.05, 0.95, 0.01),
          centerV: snap(point.v, -0.2, 1.2, 0.01),
        }),
      },
      {
        id: 'bias',
        label: 'CURVE BIAS',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'bias', label: 'BIAS', axis: 'x', min: 0, max: 1, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.bias, v: 0 }),
        apply: (p, point) => ({ ...p, bias: snap(point.u, 0, 1, 0.01) }),
      },
    ],
    decorations: (p) => [
      { key: 'seed-trace', className: 'advanced-limit', from: { u: 0.005, v: 1 }, to: { u: 0.005, v: 0.15 + (p.seed / 999) * 0.7 } },
    ],
    // 実機では(1,1)に白い終端keyアンカーが立つ。end値の変更はkey値不変の
    // 不変条件に反するため、Motoliiでは非操作の目印としてだけ描く。
    staticMarkers: [{ key: 'end-key-anchor', u: 1, v: 1 }],
  },
  Steps: {
    confirmed: true,
    evaluate: stepsValue,
    // 実機は終端の端数ジャンプを結線せず、(1,1)のendpoint dotだけで示す。
    openEnd: true,
    defaults: { width: 0.178, smooth: 0 },
    handles: [
      {
        id: 'step-width',
        label: 'STEP WIDTH',
        role: 'slider',
        kind: 'anchor',
        params: [
          { key: 'width', label: 'WIDTH', axis: 'x', min: 0.06, max: 0.5, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.width, v: 0 }),
        apply: (p, point) => {
          const width = snap(point.u, 0.06, 0.5, 0.01);
          return { ...p, width, smooth: Math.min(p.smooth, width * 0.95) };
        },
      },
      {
        id: 'smoothness',
        label: 'SMOOTHNESS',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'smooth', label: 'SMOOTH', axis: 'x', min: 0, max: 0.45, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.width + p.smooth, v: 1 }),
        apply: (p, point) => ({
          ...p,
          smooth: snap(point.u - p.width, 0, p.width * 0.95, 0.01),
        }),
      },
    ],
    decorations: (p) => [
      { key: 'lever', className: 'advanced-stem anchor-stem', from: { u: p.width, v: 0 }, to: { u: p.width + p.smooth, v: 1 } },
    ],
  },
  'Elastic Steps': {
    confirmed: true,
    evaluate: elasticStepsValue,
    overshoots: true,
    openEnd: true,
    defaults: { width: 0.2, elasticity: 0.5 },
    handles: [
      {
        id: 'step-width',
        label: 'STEP WIDTH',
        role: 'slider',
        kind: 'param',
        params: [
          { key: 'width', label: 'WIDTH', axis: 'x', min: 0.09, max: 0.5, step: 0.01 },
        ],
        anchor: (p) => ({ u: p.width, v: 0 }),
        apply: (p, point) => ({ ...p, width: snap(point.u, 0.09, 0.5, 0.01) }),
      },
      {
        id: 'elasticity',
        label: 'ELASTICITY',
        role: 'slider',
        kind: 'anchor',
        params: [
          { key: 'elasticity', label: 'ELASTICITY', axis: 'y', min: 0, max: 1, step: 0.01 },
        ],
        anchor: (p) => ({ u: 0.02, v: p.elasticity }),
        apply: (p, point) => ({ ...p, elasticity: snap(point.v, 0, 1, 0.01) }),
      },
    ],
    decorations: (p) => {
      const cornerU = Math.min(2 * p.width, 0.95);
      return [
        { key: 'lever', className: 'advanced-stem anchor-stem', from: { u: 0.02, v: p.elasticity }, to: { u: cornerU, v: elasticStepsValue(p, cornerU + 0.001) } },
      ];
    },
  },
};

export function makeInitialAdvancedParameters() {
  return Object.fromEntries(
    Object.entries(ADVANCED_SPECS).map(([name, spec]) => [
      name,
      { ...spec.defaults },
    ]),
  );
}

export function advancedPathPoints(name, parameters, samples = 240) {
  const spec = ADVANCED_SPECS[name];
  const points = [];
  for (let index = 0; index <= samples; index += 1) {
    // openEnd型は終端ジャンプを結線しない(端数はendpoint dotが示す)。
    const u =
      spec.openEnd && index === samples ? 0.9995 : index / samples;
    points.push({ u, v: spec.evaluate(parameters, u) });
  }
  return points;
}
