# CAPABILITY: material = MaterialX + OpenPBR. Asks MaterialX's own MSL generator for the blue-glass surface shader.
# run with Blender's bundled python (MaterialX 1.39.2). Writes glass.vert.metal / glass.frag.metal next to this file.
import os, sys
import MaterialX as mx
import MaterialX.PyMaterialXGenShader as gs
import MaterialX.PyMaterialXGenGlsl as _glsl  # load order matters for the bundled dylibs
import MaterialX.PyMaterialXGenMsl as msl

here = os.path.dirname(os.path.abspath(__file__))
doc = mx.createDocument()
stdlib = mx.createDocument()
mx.loadLibraries(mx.getDefaultDataLibraryFolders(), mx.getDefaultDataSearchPath(), stdlib)
doc.importLibrary(stdlib)
s = doc.addNode("open_pbr_surface", "glass", "surfaceshader")
for name, typ, val in [("base_weight", "float", 0.0), ("transmission_weight", "float", 1.0),
                       ("transmission_color", "color3", mx.Color3(0.06, 0.30, 1.0)), ("transmission_depth", "float", 0.9),
                       ("specular_ior", "float", 1.5), ("specular_roughness", "float", 0.02)]:
    s.setInputValue(name, val, typ)
m = doc.addMaterialNode("glass_mat", s)
gen = msl.MslShaderGenerator.create()
ctx = gs.GenContext(gen)
ctx.registerSourceCodeSearchPath(mx.getDefaultDataSearchPath())
shader = gen.generate("glass", m, ctx)
for stage, ext in [(gs.VERTEX_STAGE, "vert"), (gs.PIXEL_STAGE, "frag")]:
    open(os.path.join(here, f"glass.{ext}.metal"), "w").write(shader.getSourceCode(stage))
print("ok", [len(shader.getSourceCode(x)) for x in (gs.VERTEX_STAGE, gs.PIXEL_STAGE)])
