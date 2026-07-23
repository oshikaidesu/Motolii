import Konva from "konva";
import { Application, Particle, ParticleContainer, Rectangle, Texture } from "pixi.js";

const WIDTH = 1200;
const HEIGHT = 512;
const VISIBLE_KEYS = 20_000;
const DRAWS_PER_SAMPLE = 5;

function percentile(samples, fraction) {
  const sorted = [...samples].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length * fraction)];
}

function keyPosition(index) {
  return {
    x: (index * 37) % WIDTH,
    y: 18 + (index % 30) * 16,
  };
}

function objectPosition(index) {
  return {
    x: (index * 71) % WIDTH,
    y: 24 + (index % 16) * 29,
  };
}

function particleAt(position, kind, tint) {
  return new Particle({
    texture: Texture.WHITE,
    x: position.x,
    y: position.y,
    scaleX: kind === "key" ? 3 : 22,
    scaleY: kind === "key" ? 3 : 7,
    anchorX: 0.5,
    anchorY: 0.5,
    tint,
  });
}

function pixiParticles(count, kind, tint) {
  const positionFor = kind === "key" ? keyPosition : objectPosition;
  return Array.from({ length: count }, (_, index) => particleAt(positionFor(index), kind, tint));
}

function konvaNodes(count, kind) {
  const positionFor = kind === "key" ? keyPosition : objectPosition;
  return Array.from({ length: count }, (_, index) => {
    const position = positionFor(index);
    if (kind === "key") {
      return new Konva.Circle({ ...position, radius: 1.5, fill: "#59c7ff", listening: false });
    }
    return new Konva.Rect({
      ...position,
      width: 22,
      height: 7,
      offsetX: 11,
      offsetY: 3.5,
      fill: "#ffca63",
      listening: false,
    });
  });
}

export class DynamicSceneBenchmark {
  async initialize(pixiHost, konvaHost) {
    this.pixiApp = new Application();
    await this.pixiApp.init({
      width: WIDTH,
      height: HEIGHT,
      preference: "webgl",
      antialias: false,
      autoStart: false,
      background: "#111822",
      resolution: 1,
    });
    this.pixiApp.canvas.dataset.testid = "pixi-dynamic-surface";
    pixiHost.replaceChildren(this.pixiApp.canvas);

    this.pixiBackground = new ParticleContainer({
      particles: pixiParticles(VISIBLE_KEYS, "key", 0x38536b),
      dynamicProperties: {
        position: true,
        vertex: false,
        rotation: false,
        uvs: false,
        color: false,
      },
    });
    this.pixiBackground.update();
    this.pixiApp.stage.addChild(this.pixiBackground);
    this.renderPixi();

    this.konvaStage = new Konva.Stage({ container: konvaHost, width: WIDTH, height: HEIGHT });
    this.konvaStage.container().dataset.testid = "konva-dynamic-surface";
    this.konvaBackground = new Konva.Layer({ listening: false });
    this.konvaBackground.add(...konvaNodes(VISIBLE_KEYS, "key"));
    this.konvaStage.add(this.konvaBackground);
    this.konvaBackground.draw();

    const extracted = this.pixiApp.renderer.extract.pixels({
      target: this.pixiApp.stage,
      frame: new Rectangle(0, 0, WIDTH, HEIGHT),
    });
    let nonBackgroundPixels = 0;
    for (let offset = 0; offset < extracted.pixels.length; offset += 4) {
      if (extracted.pixels[offset] > 30 || extracted.pixels[offset + 1] > 40) {
        nonBackgroundPixels += 1;
      }
    }

    return {
      pixiRenderer: this.pixiApp.renderer.constructor.name,
      pixiCompletionSync: this.pixiApp.renderer.gl ? "gl.finish" : "render return",
      pixiNonBackgroundPixels: nonBackgroundPixels,
      konvaRenderer: "Canvas2D scene canvas + hit canvas per Layer",
      visibleKeys: VISIBLE_KEYS,
    };
  }

  renderPixi() {
    this.pixiApp.renderer.render({ container: this.pixiApp.stage });
    this.pixiApp.renderer.gl?.finish();
  }

  async measurePixi(kind, selected, frames) {
    const startedSetup = performance.now();
    const overlay = new ParticleContainer({
      particles: pixiParticles(selected, kind, 0xffca63),
      dynamicProperties: {
        position: true,
        vertex: false,
        rotation: false,
        uvs: false,
        color: false,
      },
    });
    overlay.update();
    this.pixiApp.stage.addChild(overlay);
    this.renderPixi();
    const setupMs = performance.now() - startedSetup;

    let semanticCommits = 0;
    const samples = [];
    for (let sample = 0; sample < frames; sample += 1) {
      const started = performance.now();
      for (let draw = 0; draw < DRAWS_PER_SAMPLE; draw += 1) {
        const frame = sample * DRAWS_PER_SAMPLE + draw;
        overlay.x = ((frame + 1) / (frames * DRAWS_PER_SAMPLE)) * 96;
        overlay.y = Math.sin(frame / 9) * 5;
        this.renderPixi();
      }
      samples.push((performance.now() - started) / DRAWS_PER_SAMPLE);
    }
    const semanticWritesDuringMove = semanticCommits;

    overlay.position.set(0, 0);
    this.renderPixi();
    const cancelRestored = overlay.x === 0 && overlay.y === 0;

    overlay.position.set(96, 0);
    this.renderPixi();
    semanticCommits += 1;

    this.pixiApp.stage.removeChild(overlay);
    overlay.destroy();
    this.renderPixi();

    return {
      library: "pixi.js",
      renderer: this.pixiApp.renderer.constructor.name,
      kind,
      selected,
      frames: frames * DRAWS_PER_SAMPLE,
      samples: frames,
      setupMs,
      medianMs: percentile(samples, 0.5),
      p95Ms: percentile(samples, 0.95),
      semanticWritesDuringMove,
      cancelRestored,
      commitsOnRelease: semanticCommits,
      updateStrategy: "translate one ParticleContainer overlay; React state unchanged per frame",
    };
  }

  async measureKonva(kind, selected, frames) {
    const startedSetup = performance.now();
    const dragLayer = new Konva.Layer({ listening: false });
    const overlay = new Konva.Group({ listening: false });
    overlay.add(...konvaNodes(selected, kind));
    dragLayer.add(overlay);
    this.konvaStage.add(dragLayer);
    dragLayer.draw();
    const setupMs = performance.now() - startedSetup;

    let semanticCommits = 0;
    const samples = [];
    for (let sample = 0; sample < frames; sample += 1) {
      const started = performance.now();
      for (let draw = 0; draw < DRAWS_PER_SAMPLE; draw += 1) {
        const frame = sample * DRAWS_PER_SAMPLE + draw;
        overlay.position({
          x: ((frame + 1) / (frames * DRAWS_PER_SAMPLE)) * 96,
          y: Math.sin(frame / 9) * 5,
        });
        dragLayer.draw();
      }
      samples.push((performance.now() - started) / DRAWS_PER_SAMPLE);
    }
    const semanticWritesDuringMove = semanticCommits;

    overlay.position({ x: 0, y: 0 });
    dragLayer.draw();
    const cancelRestored = overlay.x() === 0 && overlay.y() === 0;

    overlay.position({ x: 96, y: 0 });
    dragLayer.draw();
    semanticCommits += 1;

    dragLayer.destroy();

    return {
      library: "konva",
      renderer: "Canvas2D drag Layer",
      kind,
      selected,
      frames: frames * DRAWS_PER_SAMPLE,
      samples: frames,
      setupMs,
      medianMs: percentile(samples, 0.5),
      p95Ms: percentile(samples, 0.95),
      semanticWritesDuringMove,
      cancelRestored,
      commitsOnRelease: semanticCommits,
      updateStrategy: "translate one Group on a dedicated drag Layer; React state unchanged per frame",
    };
  }

  async measureAll(frames = 90) {
    const cases = [
      ["key", 1],
      ["key", 1_000],
      ["key", 10_000],
      ["object", 1],
      ["object", 100],
      ["object", 1_000],
    ];
    const pixi = [];
    const konva = [];
    for (const [kind, selected] of cases) {
      pixi.push(await this.measurePixi(kind, selected, frames));
      konva.push(await this.measureKonva(kind, selected, frames));
    }
    return { samples: frames, drawsPerSample: DRAWS_PER_SAMPLE, pixi, konva };
  }
}
