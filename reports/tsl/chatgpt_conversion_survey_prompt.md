Please research broadly with web search (English, 日本語 Zenn/Qiita/note, 中文 知乎/CSDN/掘金/Bilibili, 한국어, GitHub, forums such as discourse.threejs.org, Reddit, Hacker News) and answer in Japanese.

## Our situation
We make a video editor (native C++, macOS/Windows). Rendering: Google Filament on Dawn (WebGPU), ThorVG for 2D, one GPU device, one world. We want "cool visual effects found on the web" usable in a user's video immediately.
Many modern web effects are written in three.js TSL (node materials + multi-pass post-processing, e.g. basementstudio Shader Lab, tsl-textures, three.js examples/jsm/tsl/display).
We accept a slow IMPORT step. At import we want to CONVERT (bake) an effect once into a static package:
- WGSL (or GLSL/SPIR-V) shaders
- binding layout (uniform buffers, textures, samplers) and which uniform slots are time / knobs / input picture / mouse
- the pass graph: render targets, their sizes and formats, the order of passes, ping-pong / previous-frame textures, copies
Then a native engine plays it every frame by only updating uniform values. No JavaScript, no browser, no three.js at playback.
What we already have: three.js's own WGSLNodeBuilder run headless gives WGSL + bindings for single-pass effects; Dawn's capture/replay (src/dawn/replay) records every WebGPU call of a run and replays it byte-identically, but it only replays forward (cannot loop a frame with new uniform values).

## Question
List existing projects, tools, libraries, papers, articles and community threads that do this kind of conversion or any part of it. For example (search beyond these):
1. TSL / three.js NodeMaterial / post-processing → exported static shaders and passes (offline/headless export, "compile TSL to WGSL", NodeToy-style exporters, Babylon.js Node Material / Node Render Graph export, PlayCanvas, pmndrs postprocessing).
2. Web effect / shader formats that describe multi-pass graphs as data and are played by native hosts: ISF (multi-pass, persistent buffers), Shadertoy multipass (Buffer A-D) importers in native tools (TouchDesigner, Unity "ShaderMan", Godot, VVVV, Notch, Synesthesia, KodeLife, Magic), cables.gl export, Hydra, Godot visual shaders.
3. GPU API capture → replay or CODE GENERATION with parameterized values: webgpu_recorder (Brendan Duncan), wgpu's API trace + player, Dawn capture/replay, GFXReconstruct (gfxrecon-convert / to-C++), RenderDoc, Spector.js, WebGPU Inspector. Which of them can turn one captured frame into a loopable, parameterized program?
4. Render-graph / frame-graph serialization formats (JSON frame graphs, Filament's FrameGraph, Bevy render graph, Unity SRP RenderGraph, Babylon Frame Graph) that a native engine could load.
5. Shader translation pieces between WGSL and Filament materials (naga, Tint, SPIRV-Cross, Slang, Filament matc).

For each item give: name, URL, license, input → output (what exactly it converts), whether it handles multi-pass / time uniforms / bindings, maintained or not (last activity if you can see it), and how well it fits our case. Only give URLs you actually found by search; say clearly when unsure. End with: which 3 are closest to our case and why.
