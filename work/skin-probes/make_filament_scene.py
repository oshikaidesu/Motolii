# Blender owns curve -> tube (bevel). We only place control points and export glTF.
import bpy, math
bpy.ops.wm.read_factory_settings(use_empty=True)

curve = bpy.data.curves.new("tube_curve", "CURVE")
curve.dimensions = "3D"
curve.resolution_u = 120
curve.bevel_mode = "ROUND"
curve.bevel_depth = 0.85
curve.bevel_resolution = 36
curve.use_fill_caps = True
sp = curve.splines.new("BEZIER")
# Blender is Z-up: (x, -depth, height). Intended (x, height, depth): (-6.5,-0.4,2.0) (-2.4,2.0,0.6) (2.2,-0.7,-0.8) (6.5,1.4,1.2)
pts = [(-6.5, -2.0, -0.4), (-2.4, -0.6, 2.0), (2.2, 0.8, -0.7), (6.5, -1.2, 1.4)]
sp.bezier_points.add(len(pts) - 1)
for p, c in zip(sp.bezier_points, pts):
    p.co = c
    p.handle_left_type = p.handle_right_type = "AUTO"
obj = bpy.data.objects.new("tube", curve)
bpy.context.collection.objects.link(obj)
bpy.context.view_layer.objects.active = obj
obj.select_set(True)
bpy.ops.object.convert(target="MESH")
bpy.ops.object.shade_smooth()
glass = bpy.data.materials.new("glass")
obj.data.materials.append(glass)

bpy.ops.mesh.primitive_plane_add(size=16, location=(0, 0, -1.6))
floor = bpy.context.active_object
floor.name = "floor"
floor.data.materials.append(bpy.data.materials.new("floor"))
# UVs for the floor (check pattern later)
bpy.ops.mesh.primitive_plane_add(size=1, location=(0, -5.0, 1.5))
board = bpy.context.active_object
board.name = "video"
board.scale = (16, 9, 1)
board.rotation_euler = (math.radians(-90), 0, 0)
board.data.materials.append(bpy.data.materials.new("video"))
# The SVG sheet: a plane between the video board and the tube, so the glass bends both.
bpy.ops.mesh.primitive_plane_add(size=1, location=(1.0, -3.0, 1.4))
sheet = bpy.context.active_object
sheet.name = "svg"
sheet.scale = (3.0, 3.0, 1)
sheet.rotation_euler = (math.radians(-90), 0, 0)
sheet.data.materials.append(bpy.data.materials.new("svg"))

# Planes are single-sided; give the floor and the video board thickness so both faces exist (modifier applied at export).
for name, thickness in (("floor", 0.2), ("video", 0.1), ("svg", 0.05)):
    o = bpy.data.objects[name]
    m = o.modifiers.new("SOLIDIFY", "SOLIDIFY")
    m.thickness = thickness
    m.offset = -1
# The Forge's converter reads mesh data only, not glTF node transforms: bake them into the vertices.
for o in bpy.data.objects:
    o.select_set(True)
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
bpy.ops.object.select_all(action="DESELECT")
bpy.ops.export_scene.gltf(filepath="/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/probe/fil/scene/scene.gltf",
                          export_format="GLTF_SEPARATE", export_yup=True, export_apply=True)
print("EXPORTED", len(obj.data.vertices), "tube verts")
