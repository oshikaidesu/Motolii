# Explorer fixtures: 3D models
Source: Poly Haven (https://polyhaven.com), glTF 1k (.gltf + .bin + textures, as delivered), downloaded 2026-09-29 via
api.polyhaven.com/files/<id>. Licence: CC0 1.0 (public domain). Models: Camera_01, food_apple_01, ArmChair_01, tea_set_01.
Used only by the component explorer (motolii/ui/explorer) to try a 3D face; not a production asset.

glb/: import copies. Motolii's importer takes .glb, .obj and .stl (not multi-file .gltf), so these four were converted with
Blender 4.4.3 (import glTF, export GLB) to put a model in the Media library. The raw .gltf folders above stay as delivered
for the flutter_scene spike; the same CC0 licence applies.
