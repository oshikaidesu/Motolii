const shader = `
struct VertexInput {
  @location(0) position: vec2f,
};

@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4f {
  return vec4f(input.position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
  return vec4f(0.35, 0.78, 1.0, 1.0);
}
`;

function lowerBound(keys, time) {
  let low = 0;
  let high = keys.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (keys[middle].time < time) low = middle + 1;
    else high = middle;
  }
  return low;
}

function visibleRange(keys, start, end) {
  return [lowerBound(keys, start), lowerBound(keys, end)];
}

export class TimelineRenderer {
  constructor(canvas, model) {
    this.canvas = canvas;
    this.model = model;
    this.mode = "canvas2d";
    this.webgpu = { available: false, reason: "not requested" };
    this.context2d = canvas.getContext("2d");
  }

  async initializeWebGpu() {
    if (!navigator.gpu) {
      this.webgpu = { available: false, reason: "navigator.gpu unavailable" };
      return this.webgpu;
    }
    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      this.webgpu = { available: false, reason: "requestAdapter returned null" };
      return this.webgpu;
    }
    const device = await adapter.requestDevice();
    const gpuCanvas = document.createElement("canvas");
    gpuCanvas.width = this.canvas.width;
    gpuCanvas.height = this.canvas.height;
    const context = gpuCanvas.getContext("webgpu");
    const format = navigator.gpu.getPreferredCanvasFormat();
    context.configure({ device, format, alphaMode: "premultiplied" });
    const pipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: {
        module: device.createShaderModule({ code: shader }),
        entryPoint: "vs_main",
        buffers: [{
          arrayStride: 8,
          attributes: [{ shaderLocation: 0, offset: 0, format: "float32x2" }],
        }],
      },
      fragment: {
        module: device.createShaderModule({ code: shader }),
        entryPoint: "fs_main",
        targets: [{ format }],
      },
      primitive: { topology: "point-list" },
    });
    const vertexBuffer = device.createBuffer({
      size: this.model.keyframes.length * 8,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    const info = adapter.info;
    const adapterInfo = info ? {
      architecture: info.architecture,
      description: info.description,
      device: info.device,
      vendor: info.vendor,
    } : {};
    this.webgpu = {
      available: true,
      reason: null,
      adapterInfo,
      gpuCanvas,
      context,
      device,
      pipeline,
      vertexBuffer,
    };
    return { available: true, reason: null, adapterInfo };
  }

  drawCanvas(start, secondsVisible) {
    const context = this.context2d;
    const { width, height } = this.canvas;
    const end = start + secondsVisible;
    context.fillStyle = "#111822";
    context.fillRect(0, 0, width, height);
    context.fillStyle = "#38536b";
    for (const clip of this.model.clips) {
      if (clip.start + clip.duration < start || clip.start > end) continue;
      const x = ((clip.start - start) / secondsVisible) * width;
      const w = Math.max(2, (clip.duration / secondsVisible) * width);
      const y = (clip.track / 32) * height;
      context.fillRect(x, y + 2, w, 10);
    }
    const [from, to] = visibleRange(this.model.keyframes, start, end);
    context.fillStyle = "#59c7ff";
    for (let index = from; index < to; index += 1) {
      const key = this.model.keyframes[index];
      const x = ((key.time - start) / secondsVisible) * width;
      const y = (key.track / 32) * height + 15;
      context.fillRect(x, y, 1.5, 1.5);
    }
    return to - from;
  }

  async drawWebGpu(start, secondsVisible) {
    if (!this.webgpu.available) return 0;
    const end = start + secondsVisible;
    const [from, to] = visibleRange(this.model.keyframes, start, end);
    const positions = new Float32Array((to - from) * 2);
    for (let index = from; index < to; index += 1) {
      const key = this.model.keyframes[index];
      const output = (index - from) * 2;
      positions[output] = ((key.time - start) / secondsVisible) * 2 - 1;
      positions[output + 1] = 1 - (key.track / 31) * 2;
    }
    const { device, context, pipeline, vertexBuffer } = this.webgpu;
    device.queue.writeBuffer(vertexBuffer, 0, positions);
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{
        view: context.getCurrentTexture().createView(),
        clearValue: { r: 0.067, g: 0.094, b: 0.133, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pass.setPipeline(pipeline);
    pass.setVertexBuffer(0, vertexBuffer);
    pass.draw(to - from);
    pass.end();
    device.queue.submit([encoder.finish()]);
    await device.queue.onSubmittedWorkDone();
    return to - from;
  }

  async measure(mode, frames = 120) {
    const samples = [];
    let visibleKeys = 0;
    for (let frame = 0; frame < frames; frame += 1) {
      const start = (frame / frames) * 180;
      const started = performance.now();
      visibleKeys = mode === "webgpu"
        ? await this.drawWebGpu(start, 48)
        : this.drawCanvas(start, 48);
      samples.push(performance.now() - started);
    }
    samples.sort((a, b) => a - b);
    return {
      mode,
      frames,
      visibleKeys,
      medianMs: samples[Math.floor(samples.length * 0.5)],
      p95Ms: samples[Math.floor(samples.length * 0.95)],
    };
  }
}
