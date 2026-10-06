# Motolii v1 (2026-10-06, 30-minute build)

One process: CORE = Rerun (`core/`, C ABI over the Rerun store), RENDER CORE = The Forge (`app/Motolii.cpp`, an IApp built with The Forge's
own Xcode project copied to `_ext/The-Forge/Examples_3/Unit_Tests/{macOS_Xcode,src}/Motolii`), SKIN = Flutter (`skin/`, embedded in the same process),
CAPABILITIES: ThorVG WebGPU engine (`capabilities/svg_thorvg.c`), AVFoundation video, MaterialX-generated OpenPBR MSL (`capabilities/materialx/`).

Wire: The Forge draws into an IOSurface texture the Flutter Texture shows; ThorVG's and the decoder's Metal textures enter The Forge as native handles;
values (camera, video seconds, SVG scale, playhead) go Skin -> Rerun -> The Forge.

Measured (M4, Flutter debug build): Stage 60.0 fps; playhead message -> Forge frame submitted 13-15 ms; video 27-29 of 30 frames/s by exact seek;
ThorVG draw+sync 0.3-0.9 ms; CPU frame max ~15 ms. Pixel readbacks 0, per-frame subprocess 0, per-frame file reads 0, bulk CPU<->GPU copies 0.

Not done / limits: the glass refracts and reflects the environment texture (the video) only, not scene geometry, and has no thickness absorption
(MaterialX hardware path; nobody in the fixed set owns a glass-aware integrator). Geometry is The Forge's generated cube/spheres, not the fixture mesh.
No audio, no export. Hand drag in the Skin was not verified by the agent. The sample's skybox/planets/debug UI are still in the app.
Build: `build_app.sh` (needs EXTRA_C/EXTRA_LD/EXTRA_RPATH as in the session), run: `run.sh`.
