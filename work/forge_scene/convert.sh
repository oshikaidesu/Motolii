#!/bin/zsh
# Blender -> glTF -> The Forge .bin (AssetPipelineCmd). The sample reads uint32 indices: the Forge loader assumes them only when the mesh has more than 65535 vertices, so the tube is tessellated past that.
FORGE=/Users/member_ottoto/rust_ae/_ext/The-Forge
HERE=/Users/member_ottoto/rust_ae/Motolii/work/forge_scene
/Applications/Blender.app/Contents/MacOS/Blender -b --python $HERE/make_scene.py 2>&1 | grep -E "EXPORTED|Error|Traceback"
rm -rf $FORGE/Art/Meshe $FORGE/Art/Meshes/tube_scene; mkdir -p $FORGE/Art/Meshes/tube_scene
$FORGE/Common_3/Tools/AssetPipeline/Apple/Bin/Release/AssetPipelineCmd -pgltf --input-file $HERE/scene.gltf --output $FORGE/Art/Meshes/tube_scene --force > /dev/null 2>&1
# upstream tool truncates the output path to the input path length; move the result
[ -f $FORGE/Art/Meshe/tube_scenescene.bin ] && mv $FORGE/Art/Meshe/tube_scenescene.bin $FORGE/Art/Meshes/tube_scene/scene.bin
[ -f $FORGE/Art/Meshe/tube_scene/scene.bin ] && mv $FORGE/Art/Meshe/tube_scene/scene.bin $FORGE/Art/Meshes/tube_scene/scene.bin
rm -rf $FORGE/Art/Meshe
ls -la $FORGE/Art/Meshes/tube_scene
