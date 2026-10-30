# Runs inside `blender -b --factory-startup --python server.py -- <port> <tree>`: builds one geometry-node tree,
# then for each 4-byte float it receives, sets the tree's "Strength" input, evaluates, and sends back the mesh
# (vertex count, triangle count, float32 positions, int32 triangle corners).
import bpy, socket, struct, sys, time
import numpy as np

port, tree = int(sys.argv[sys.argv.index('--') + 1]), sys.argv[sys.argv.index('--') + 2]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.mesh.primitive_plane_add()
obj = bpy.context.active_object
ng = bpy.data.node_groups.new('gn', 'GeometryNodeTree')
ng.interface.new_socket('Geometry', in_out='INPUT', socket_type='NodeSocketGeometry')
ng.interface.new_socket('Geometry', in_out='OUTPUT', socket_type='NodeSocketGeometry')
s = ng.interface.new_socket('Strength', in_out='INPUT', socket_type='NodeSocketFloat')
N, L = ng.nodes, ng.links
gi, go = N.new('NodeGroupInput'), N.new('NodeGroupOutput')

if tree.startswith('grid'):   # a grid waved by noise: grid100 = 100x100 vertices
    n = int(tree[4:])
    g = N.new('GeometryNodeMeshGrid'); g.inputs['Vertices X'].default_value = n; g.inputs['Vertices Y'].default_value = n
    noise = N.new('ShaderNodeTexNoise')
    scale = N.new('ShaderNodeVectorMath'); scale.operation = 'SCALE'
    setp = N.new('GeometryNodeSetPosition')
    L.new(noise.outputs['Color'], scale.inputs[0]); L.new(gi.outputs['Strength'], scale.inputs['Scale'])
    L.new(g.outputs['Mesh'], setp.inputs['Geometry']); L.new(scale.outputs['Vector'], setp.inputs['Offset'])
    L.new(setp.outputs['Geometry'], go.inputs['Geometry'])
elif tree.startswith('scatter'):   # cubes scattered on a sphere, realized: scatter1000 = about 1000 cubes
    n = int(tree[7:])
    ico = N.new('GeometryNodeMeshIcoSphere'); ico.inputs['Subdivisions'].default_value = 4
    dist = N.new('GeometryNodeDistributePointsOnFaces'); dist.inputs['Density'].default_value = n / 12.57
    cube = N.new('GeometryNodeMeshCube')
    inst = N.new('GeometryNodeInstanceOnPoints')
    rot = N.new('ShaderNodeCombineXYZ')
    real = N.new('GeometryNodeRealizeInstances')
    size = N.new('ShaderNodeMath'); size.operation = 'MULTIPLY'; size.inputs[1].default_value = 0.05
    L.new(ico.outputs['Mesh'], dist.inputs['Mesh']); L.new(gi.outputs['Strength'], dist.inputs['Seed'])
    L.new(gi.outputs['Strength'], size.inputs[0]); L.new(size.outputs[0], cube.inputs['Size'])
    L.new(dist.outputs['Points'], inst.inputs['Points']); L.new(cube.outputs['Mesh'], inst.inputs['Instance'])
    L.new(dist.outputs['Rotation'], inst.inputs['Rotation'])
    L.new(inst.outputs['Instances'], real.inputs['Geometry']); L.new(real.outputs['Geometry'], go.inputs['Geometry'])
mod = obj.modifiers.new('gn', 'NODES'); mod.node_group = ng
key = s.identifier

srv = socket.socket(); srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1); srv.bind(('127.0.0.1', port)); srv.listen(1)
print('READY', flush=True)
c, _ = srv.accept(); c.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
f = c.makefile('rb')
while True:
    b = f.read(4)
    if len(b) < 4: break
    t0 = time.perf_counter()
    mod[key] = struct.unpack('<f', b)[0]
    obj.update_tag()
    dg = bpy.context.evaluated_depsgraph_get(); dg.update()
    t1 = time.perf_counter()
    ev = obj.evaluated_get(dg); me = ev.to_mesh(); me.calc_loop_triangles()
    v, tr = len(me.vertices), len(me.loop_triangles)
    co = np.empty(v * 3, np.float32); me.vertices.foreach_get('co', co)
    ix = np.empty(tr * 3, np.int32); me.loop_triangles.foreach_get('vertices', ix)
    ev.to_mesh_clear()
    t2 = time.perf_counter()
    c.sendall(struct.pack('<iiff', v, tr, (t1 - t0) * 1e3, (t2 - t1) * 1e3) + co.tobytes() + ix.tobytes())
