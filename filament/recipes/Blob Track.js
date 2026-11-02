// Boxes that follow what moves in the film under it, with lines between them and where each is: the look called blob
// tracking, or Tracery. Nothing here knows what a person is: what changed from picture to picture is cut into regions.
// Its BlobTracker below is basementstudio/shader-lab's src/lib/blob-tracking/tracker.ts (7421a25, Apache-2.0, Copyright © 2026
// basement.studio LLC), changed only by taking its types and exports off (Node's stripTypeScriptTypes). Its GPU pass, the
// grid it reads, is rewritten in read() below, and how it is drawn is its effect's outline frame, centre dot, coordinate label and lines.
/**
 * Analysis-grid channel layout, written by the GPU analysis pass:
 * R = instantaneous frame difference, G = luma, B = decayed motion energy
 * (`max(previousEnergy * persistence - floor, motion)`).
 *
 * Detection thresholds the energy channel so a blob holds its lock while its
 * subject pauses; the static-scene heuristic reads the instantaneous channel so
 * a paused subject still lets the luminance fallback take over.
 */
const MOTION_CHANNEL = 0
const LUMA_CHANNEL = 1
const MOTION_ENERGY_CHANNEL = 2

const STATIC_STEPS_BEFORE_FALLBACK = 30
const MOTION_ENERGY_EPSILON = 0.01
/** Fewer moving cells than this for STATIC_STEPS_BEFORE_FALLBACK steps means a still scene. */
const STATIC_ACTIVE_CELLS = 2
/** Cells that were on stay on down to this fraction of the threshold. */
const MOTION_HYSTERESIS = 0.55
const TRACK_GRACE_FRAMES = 10
const HISTORY_LENGTH = 16
const MAX_MATCH_DISTANCE = 0.25
const FADE_IN_FRAMES = 4
/** How fast the per-track velocity estimate follows the observed step. */
const VELOCITY_BLEND = 0.35
/**
 * Readback is asynchronous, so a detection describes where a subject was one or
 * two frames ago. Extrapolating by the estimated velocity cancels that lag.
 */
const VELOCITY_LOOKAHEAD = 1.5
const MAX_EDGE_POINTS = 64

function clamp01(value        )         {
  return Math.min(1, Math.max(0, value))
}

function trackPresence(track       )         {
  if (!track.active) {
    return clamp01(1 - track.missedFrames / TRACK_GRACE_FRAMES)
  }
  return clamp01((track.ageFrames + 1) / FADE_IN_FRAMES)
}

class BlobTracker {
          currentBlobs         = []
          lastSteppedTime                = null
          luminanceFallbackActive = false
          nextId = 1
          staticStepCount = 0
          tracks          = []
          priorState                      = null

          binary = new Uint8Array(0)
          previousBinary = new Uint8Array(0)
          visited = new Int32Array(0)
          stack = new Int32Array(0)
          visitGeneration = 0

  reset()       {
    this.currentBlobs = []
    this.lastSteppedTime = null
    this.luminanceFallbackActive = false
    this.nextId = 1
    this.staticStepCount = 0
    this.tracks = []
    this.priorState = null
    this.visited.fill(0)
    this.previousBinary.fill(0)
    this.visitGeneration = 0
  }

  getBlobs()         {
    return this.currentBlobs
  }

  step(
    grid            ,
    gridWidth        ,
    gridHeight        ,
    time        ,
    config
  )       {
    if (this.lastSteppedTime !== null && time === this.lastSteppedTime) {
      this.restoreState(this.priorState)
    } else {
      this.priorState = this.captureState()
      this.lastSteppedTime = time
    }

    const mode = this.resolveMode(grid, gridWidth, gridHeight, config)
    this.binarize(grid, gridWidth, gridHeight, mode, config)
    const detections = this.detect(gridWidth, gridHeight, config)

    this.currentBlobs = config.persistentTracking
      ? this.trackPersistent(detections, config)
      : this.trackStateless(detections)
  }

          captureState()               {
    return {
      luminanceFallbackActive: this.luminanceFallbackActive,
      nextId: this.nextId,
      staticStepCount: this.staticStepCount,
      tracks: this.tracks.map((track) => ({
        ...track,
        history: track.history.slice(),
      })),
    }
  }

          restoreState(state                     )       {
    if (!state) {
      this.luminanceFallbackActive = false
      this.nextId = 1
      this.staticStepCount = 0
      this.tracks = []
      return
    }
    this.luminanceFallbackActive = state.luminanceFallbackActive
    this.nextId = state.nextId
    this.staticStepCount = state.staticStepCount
    this.tracks = state.tracks.map((track) => ({
      ...track,
      history: track.history.slice(),
    }))
  }

          ensureBuffers(cellCount        )       {
    if (this.binary.length === cellCount) {
      return
    }
    this.binary = new Uint8Array(cellCount)
    this.previousBinary = new Uint8Array(cellCount)
    this.visited = new Int32Array(cellCount)
    this.stack = new Int32Array(cellCount)
    this.visitGeneration = 0
  }

          resolveMode(
    grid            ,
    gridWidth        ,
    gridHeight        ,
    config
  )                         {
    if (config.detectionMode !== "auto") {
      return config.detectionMode
    }

    const cellCount = gridWidth * gridHeight
    const cutoff = config.motionThreshold * 255
    let activeCells = 0
    for (let index = 0; index < cellCount; index += 1) {
      if ((grid[index * 4 + MOTION_CHANNEL] ?? 0) >= cutoff) activeCells += 1
    }

    if (activeCells < STATIC_ACTIVE_CELLS) {
      this.staticStepCount += 1
      if (this.staticStepCount >= STATIC_STEPS_BEFORE_FALLBACK) {
        this.luminanceFallbackActive = true
      }
    } else {
      this.staticStepCount = 0
      this.luminanceFallbackActive = false
    }

    return this.luminanceFallbackActive ? "luminance" : "motion"
  }

          binarize(
    grid            ,
    gridWidth        ,
    gridHeight        ,
    mode                        ,
    config
  )       {
    const cellCount = gridWidth * gridHeight
    this.ensureBuffers(cellCount)

    const channelOffset =
      mode === "motion" ? MOTION_ENERGY_CHANNEL : LUMA_CHANNEL
    // `sensitivity` reads high-is-more-sensitive, so it inverts into a cutoff.
    const threshold =
      (mode === "motion" ? config.motionThreshold : 1 - config.sensitivity) *
      255
    const binary = this.binary
    const previous = this.previousBinary
    const low = mode === "motion" ? threshold * MOTION_HYSTERESIS : threshold

    for (let index = 0; index < cellCount; index += 1) {
      const value = grid[index * 4 + channelOffset] ?? 0
      const on = value >= threshold || (value >= low && previous[index] === 1)
      binary[index] = on ? 1 : 0
    }
    previous.set(binary)
  }

          detect(
    gridWidth        ,
    gridHeight        ,
    config
  )              {
    const binary = this.binary
    const visited = this.visited
    const stack = this.stack
    const detections              = []

    this.visitGeneration += 1
    const generation = this.visitGeneration

    for (let startY = 0; startY < gridHeight; startY += 1) {
      for (let startX = 0; startX < gridWidth; startX += 1) {
        const startIndex = startY * gridWidth + startX
        if (binary[startIndex] !== 1 || visited[startIndex] === generation) {
          continue
        }

        let area = 0
        let sumX = 0
        let sumY = 0
        let minX = startX
        let maxX = startX
        let minY = startY
        let maxY = startY

        visited[startIndex] = generation
        stack[0] = startIndex
        let stackSize = 1
        const boundary           = []

        while (stackSize > 0) {
          stackSize -= 1
          const index = stack[stackSize]
          const x = index % gridWidth
          const y = (index - x) / gridWidth

          area += 1
          sumX += x
          sumY += y
          if (x < minX) minX = x
          if (x > maxX) maxX = x
          if (y < minY) minY = y
          if (y > maxY) maxY = y
          if (
            x === 0 ||
            y === 0 ||
            x === gridWidth - 1 ||
            y === gridHeight - 1 ||
            binary[index - 1] !== 1 ||
            binary[index + 1] !== 1 ||
            binary[index - gridWidth] !== 1 ||
            binary[index + gridWidth] !== 1
          ) {
            boundary.push(index)
          }

          if (x > 0) {
            const neighbor = index - 1
            if (binary[neighbor] === 1 && visited[neighbor] !== generation) {
              visited[neighbor] = generation
              stack[stackSize] = neighbor
              stackSize += 1
            }
          }
          if (x < gridWidth - 1) {
            const neighbor = index + 1
            if (binary[neighbor] === 1 && visited[neighbor] !== generation) {
              visited[neighbor] = generation
              stack[stackSize] = neighbor
              stackSize += 1
            }
          }
          if (y > 0) {
            const neighbor = index - gridWidth
            if (binary[neighbor] === 1 && visited[neighbor] !== generation) {
              visited[neighbor] = generation
              stack[stackSize] = neighbor
              stackSize += 1
            }
          }
          if (y < gridHeight - 1) {
            const neighbor = index + gridWidth
            if (binary[neighbor] === 1 && visited[neighbor] !== generation) {
              visited[neighbor] = generation
              stack[stackSize] = neighbor
              stackSize += 1
            }
          }
        }

        if (area < config.minBlobSize) {
          continue
        }

        const boxWidth = maxX - minX + 1
        const boxHeight = maxY - minY + 1
        const stride = Math.max(1, Math.ceil(boundary.length / MAX_EDGE_POINTS))
        const edge              = []
        for (
          let i = 0;
          i < boundary.length && edge.length < MAX_EDGE_POINTS;
          i += stride
        ) {
          const cell = boundary[i]
          const bx = cell % gridWidth
          const by = (cell - bx) / gridWidth
          edge.push({
            x: clamp01((bx + 0.5) / gridWidth),
            y: clamp01((by + 0.5) / gridHeight),
          })
        }
        detections.push({
          area,
          cx: clamp01((sumX / area + 0.5) / gridWidth),
          cy: clamp01((sumY / area + 0.5) / gridHeight),
          edge,
          halfHeight: boxHeight / 2 / gridHeight,
          halfWidth: boxWidth / 2 / gridWidth,
        })
      }
    }

    detections.sort((left, right) => right.area - left.area)
    const limit = Math.max(0, Math.floor(config.blobAmount))
    return detections.slice(0, limit)
  }

          trackStateless(detections             )         {
    this.tracks = []
    return detections.map((detection, index) => ({
      active: true,
      area: detection.area,
      cx: detection.cx,
      cy: detection.cy,
      edge: detection.edge,
      halfHeight: detection.halfHeight,
      halfWidth: detection.halfWidth,
      history: [{ x: detection.cx, y: detection.cy }],
      id: index,
      presence: 1,
      vx: 0,
      vy: 0,
    }))
  }

          trackPersistent(
    detections             ,
    config
  )         {
    const blend = 1 - clamp01(config.smoothing)
    const available = this.tracks.slice()
    const matchedTrackIds = new Set        ()

    for (const detection of detections) {
      let bestTrack               = null
      let bestDistance = Number.POSITIVE_INFINITY

      for (const track of available) {
        if (matchedTrackIds.has(track.id)) {
          continue
        }
        // Compare against where the track is heading, not where it was: two
        // subjects crossing have near-identical positions but opposite
        // velocities, which is what previously swapped their ids.
        const steps = 1 + track.missedFrames
        const predictedX = track.cx + track.vx * steps
        const predictedY = track.cy + track.vy * steps
        const dx = predictedX - detection.cx
        const dy = predictedY - detection.cy
        const distance = dx * dx + dy * dy
        const reach = MAX_MATCH_DISTANCE * (1 + track.missedFrames)
        if (distance > reach * reach) {
          continue
        }
        if (distance < bestDistance) {
          bestDistance = distance
          bestTrack = track
        }
      }

      if (bestTrack) {
        matchedTrackIds.add(bestTrack.id)
        const previousX = bestTrack.cx
        const previousY = bestTrack.cy
        bestTrack.cx += (detection.cx - bestTrack.cx) * blend
        bestTrack.cy += (detection.cy - bestTrack.cy) * blend
        bestTrack.vx += (bestTrack.cx - previousX - bestTrack.vx) * VELOCITY_BLEND
        bestTrack.vy += (bestTrack.cy - previousY - bestTrack.vy) * VELOCITY_BLEND
        bestTrack.halfWidth +=
          (detection.halfWidth - bestTrack.halfWidth) * blend
        bestTrack.halfHeight +=
          (detection.halfHeight - bestTrack.halfHeight) * blend
        bestTrack.area = detection.area
        bestTrack.edge = detection.edge
        bestTrack.active = true
        bestTrack.missedFrames = 0
        bestTrack.ageFrames += 1
        bestTrack.history.push({ x: bestTrack.cx, y: bestTrack.cy })
        if (bestTrack.history.length > HISTORY_LENGTH) {
          bestTrack.history.shift()
        }
      } else {
        this.tracks.push({
          active: true,
          ageFrames: 0,
          area: detection.area,
          cx: detection.cx,
          cy: detection.cy,
          edge: detection.edge,
          halfHeight: detection.halfHeight,
          halfWidth: detection.halfWidth,
          history: [{ x: detection.cx, y: detection.cy }],
          id: this.nextId,
          missedFrames: 0,
          vx: 0,
          vy: 0,
        })
        matchedTrackIds.add(this.nextId)
        this.nextId += 1
      }
    }

    const survivors          = []
    for (const track of this.tracks) {
      if (matchedTrackIds.has(track.id)) {
        survivors.push(track)
        continue
      }
      track.missedFrames += 1
      track.active = false
      track.vx *= 1 - VELOCITY_BLEND
      track.vy *= 1 - VELOCITY_BLEND
      if (track.missedFrames <= TRACK_GRACE_FRAMES) {
        survivors.push(track)
      }
    }
    this.tracks = survivors

    return this.tracks.map((track) => ({
      active: track.active,
      area: track.area,
      cx: track.cx,
      cy: track.cy,
      edge: track.edge,
      halfHeight: track.halfHeight,
      halfWidth: track.halfWidth,
      history: track.history.slice(),
      id: track.id,
      presence: trackPresence(track),
      vx: track.vx,
      vy: track.vy,
    }))
  }
}

// The film under this sketch, read as Shader Lab's analysis pass reads its picture (src/renderer/blob-tracking-pass.ts:
// a 128 x 72 grid; R the change since the last picture over the cell and its neighbours, G luma, B that change kept and fading).
const W = 128, H = 72, keep = 0.82, floorOf = 0.025;
const config = { blobAmount: 6, detectionMode: 'auto', minBlobSize: 3, motionThreshold: 0.12, persistentTracking: true, sensitivity: 0.8, smoothing: 0.6 };
const tracker = new BlobTracker(), grid = new Uint8Array(W * H * 4);
let film, luma = new Float32Array(W * H), before = new Float32Array(W * H), moved = new Float32Array(W * H), energy = new Float32Array(W * H), seen = false, spots = [];

function setup() {
  film = createVideo('film');
  createCanvas(film.width, film.height);
  film.size(W, H);
  textSize(height / 42);
}

// (a cell that changed less than this reaches no threshold of the tracker's, so it spreads nothing: most cells of most pictures)
const small = 0.05, spread = [[0, 0, 1], [1, 0, 1], [-1, 0, 1], [0, 1, 1], [0, -1, 1], [1, 1, 1], [-1, 1, 1], [1, -1, 1], [-1, -1, 1], [2, 0, 0.85], [-2, 0, 0.85], [0, 2, 0.85], [0, -2, 0.85]];

// No film under it (its tile on the shelf): three spots wander, painted here, for it to find.
function pretend() {
  const t = frameCount / 20, p = new Uint8Array(W * H * 4);
  spots = [0, 1, 2].map(k => [0.5 + 0.34 * Math.sin(t * (1 + k / 3) + 2 * k), 0.5 + 0.3 * Math.cos(t * (1.3 - k / 4) + k)]);
  for (const [u, v] of spots) for (let y = -6; y <= 6; ++y) for (let x = -6; x <= 6; ++x) {
    const c = Math.round(v * H + y), r = Math.round(u * W + x);
    if (x * x + y * y < 30 && c >= 0 && c < H && r >= 0 && r < W) p.fill(255, 4 * (c * W + r), 4 * (c * W + r) + 3);
  }
  return p;
}

function read() {
  film.loadPixels();
  const p = film.source.data ? film.pixels : pretend();
  moved.fill(0);
  for (let y = 0, i = 0; y < H; ++y) for (let x = 0; x < W; ++x, ++i) {
    luma[i] = (0.2126 * p[4 * i] + 0.7152 * p[4 * i + 1] + 0.0722 * p[4 * i + 2]) / 255;
    const d = seen ? Math.abs(luma[i] - before[i]) : 0;
    if (d < small) continue;
    for (const [dx, dy, k] of spread) {
      const u = x + dx, v = y + dy, j = v * W + u;
      if (u >= 0 && v >= 0 && u < W && v < H && moved[j] < d * k) moved[j] = d * k;
    }
  }
  for (let i = 0; i < W * H; ++i) {
    const kept = energy[i] * keep - floorOf;
    energy[i] = kept > moved[i] ? kept : moved[i];
    grid[4 * i] = moved[i] * 255, grid[4 * i + 1] = luma[i] * 255, grid[4 * i + 2] = energy[i] * 255, grid[4 * i + 3] = 255;
  }
  [luma, before] = [before, luma];
  seen = true;
}

function draw() {
  clear();
  if (frameCount % 2) {   // (this runs at 60 a second, most films at 30 or less)
    read();
    tracker.step(grid, W, H, frameCount, config);
  }
  const blobs = tracker.getBlobs().filter(b => b.active || b.presence > 0.05).sort((a, b) => a.id - b.id);
  noFill();
  stroke(255);
  strokeWeight(height / 400);
  for (let i = 1; i < blobs.length; ++i) line(blobs[i - 1].cx * width, blobs[i - 1].cy * height, blobs[i].cx * width, blobs[i].cy * height);
  for (const b of blobs) rect((b.cx - b.halfWidth) * width, (b.cy - b.halfHeight) * height, 2 * b.halfWidth * width, 2 * b.halfHeight * height);
  noStroke();
  fill(255);
  for (const [u, v] of spots) circle(u * width, v * height, height / 5);
  for (const b of blobs) {
    circle(b.cx * width, b.cy * height, height / 120);
    text(`${b.cx.toFixed(3)} ${b.cy.toFixed(3)}`, (b.cx + b.halfWidth) * width + 6, (b.cy - b.halfHeight) * height + height / 42);
  }
}
