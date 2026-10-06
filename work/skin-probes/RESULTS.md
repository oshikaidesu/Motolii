# Skin probes, 2026-10-06 (30-minute phase). Measured on this Mac (Apple M4), debug/throwaway builds. Nothing here is Motolii code.

Every row is one hop of: video decode -> Filament (glass scene) -> IOSurface -> Flutter Texture. Rows say what ran and what did not.

| hop | owner | handle crossing | measured | state |
|---|---|---|---|---|
| H.264 long-GOP exact seek | AVFoundation | IOSurface CVPixelBuffer | median 24 ms / p90 49 ms; reverse 7-9 fps | FAIL for scrub on the original file |
| ProRes Proxy (intra-frame) exact seek | AVFoundation + VideoToolbox (ffmpeg only transcodes the clip once, offline) | IOSurface CVPixelBuffer | median 1.9 ms; reverse -1x 30 fps, -2x 59.8 fps | CLOSED |
| decode -> Filament external image -> IOSurface RT, 1920x1080, in one process | Filament (setExternalImage) | CVPixelBuffer in, IOSurface MTLTexture out | scrub input->pixels median 3.8 ms / p90 4.2 ms; play 59.8 fps, gap median 16.66 ms; Filament frame 1.8 ms (flushAndWait, pessimistic) | CLOSED (quad only, no glass in this probe) |
| glass + transmission + reflection + bloom | Filament gltfio (KHR_materials_transmission/volume/ior) | - | gltf_viewer Metal HUD: 8.61 ms GPU/frame at 2048x1280, 57 fps (vsync-limited) | CLOSED for the look; video was a static PNG there |
| IOSurface video frame -> Flutter Texture widget | Flutter macOS FlutterTexture | CVPixelBuffer (copyPixelBuffer) | debug build: Flutter 60.1 frames/s, 58-59 textures/s handed to the engine; screenshot tex_run.png | CLOSED |
| Filament output IOSurface -> CVPixelBufferCreateWithIOSurface -> Flutter | CoreVideo | IOSurface | not run together (both ends run separately; header verified) | WIRE, unrun |
| WGSL effect on Filament's IOSurface | wgpu (hal texture_from_raw/device_from_raw/queue_from_raw), naga | MTLTexture | source-verified only | WIRE, unrun. Same device+queue needed for in-order sync; Filament PlatformMetal exposes createDevice/createCommandQueue to subclass |
| SVG animation | ThorVG | - | not run | UNKNOWN (static: sw raster once is fine) |
| audio clock | Tracktion TransportControl | - | API read only | UNKNOWN (build/licence) |

Counts for the runs above: GPU->CPU pixel readbacks 0; our per-frame subprocesses 0; our per-frame file reads 0; bulk CPU<->GPU copies 0 (frames stay IOSurface).
Not delivered: a single window showing video + SVG + glass + WGSL together.
