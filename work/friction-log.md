# 30-minute connection log (2026-10-06)
Purpose: measure friction. Not architecture. Entry: motolii-glue a1 → a1.dart. Bypass: motolii-doc / motolii-edit / motolii-render.
Per connection: minutes / glue lines / upstream forked-patched / Motolii model invented / authoritative copy made / native rebuild.

start 07:15:32
07:36:56 externals start
07:44:39 OIIO configure failed: system OpenJPEG version empty vs CMake 4.1; retry with -DUSE_OPENJPEG=OFF (build flag only, no patch)
08:26:29 externals done (OIIO needed -DUSE_OPENJPEG=OFF). Next: Aurora cmake Xcode
08:28:42 Aurora built OK (libhdAurora.dylib). Next: usdrecord with hdAurora

## Aurora (2026-10-06, M4 / macOS 15.5)
- externals: 2 attempts. OIIO configure failed (system OpenJPEG version empty vs CMake 4.1) -> -DUSE_OPENJPEG=OFF. Build flag only. Low-load (nice 19, -j2) ~50 min.
- Aurora itself built in 1m43s. libhdAurora.dylib registers as renderer "Aurora" in usdrecord.
- Forks required by Aurora's own scripts: USD (adsk-feature-hgiraytracing-metal), MaterialX (autodesk-forks, v1.39.5.*.dev_adsk). USD built with PXR_ENABLE_MATERIALX_SUPPORT=OFF (installExternals.py:835) -> no UsdMtlx -> a .mtlx cannot be referenced as an authoritative material document.
- HdAurora builds a MaterialX doc itself from the Hydra network (UsdPreviewSurface-style); it does not read .mtlx files.
- usdrecord --renderer Aurora: sphere+dome light, 160px, no frame in 60s, GPU 95%. Storm in the same harness: 1.2s. Large scene (960px) ran >5min at GPU 95%.
- aurora:max_samples via RenderSettings prim: no visible effect. FrameRecorder has no setting hook; Python Engine has SetRendererSetting but no AOV readback. A per-frame driver would be C++.
- SIP: nohup strips DYLD_LIBRARY_PATH; run without nohup.
10:30:06 Forge: apple_gpu.data lacks 'Apple M4' -> +1 data row (local clone, not code)

## The Forge 16_Raytracing (2026-10-06, M4)
- clone 14s (637MB). Art.zip 1.16GB (conffx.com, official). Xcode build OK with -Wno-nonportable-include-path (upstream include-case typo under -Werror; flag only).
- Forge GPU table (Common_3/OS/Darwin/apple_gpu.data) had no 'Apple M4' -> "Office preset not supported" exit. +1 data row.
- First build without Art/ left PathStatement.txt as a directory in the .app (build artifact). Removed + rebuilt.
- SanMiguel textures are NOT in Art.zip (only daytime_cube.tex) -> null texture -> segfault at 16_Raytracing.cpp:1252. Sample patched: failed textures fall back to Palette_Sky.tex (orig saved _ext/16_Raytracing.cpp.orig).
- macOS window-restore dialog after crashes blocks the window: launch with -ApplePersistenceIgnoreState YES.
- RESULT: ray-query path loop renders SanMiguel (grey, fallback textures) on Metal. Not our scene.
10:47:21 AssetPipelineCmd bug: output path truncated to input-path length -> wrote Art/Meshe/...; moved file by hand (upstream tool bug, no code change)
11:13:32 Forge: loader assumes uint16 indices when vertexCount<=65535 but --meshlets writes uint32 -> garbled geometry; fix = tessellate past 65535 verts and drop --meshlets (no code change). Also converter ignores glTF node transforms (bake in Blender), flag name --meshlets vs help --meshlet.

## Time-driven render + export (2026-10-06)
- Driver in the sample: frame index in -> camera, ffmpeg decodes that frame into the board texture, accumulate N spp, capture PNG, next. Deterministic (md5 of the same frame identical across runs).
- The Forge captures screenshots only under AUTOMATED_TESTING (in queuePresent); the sample now calls captureScreenshot itself. Output goes next to the .app (Screenshots/), not cwd.
- OIDN (brew bottle 2.5.1, CPU only; the Metal device is not in the bottle): 0.1 s/frame at 960x540 turns 24-32 spp into a clean frame.
- 60 frames, 960x540, 32 spp + OIDN + audio mux: 57 s wall on M4.
