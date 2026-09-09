# Visibility-query readiness evidence

This is a geometry-query experiment, not a finished image renderer.
The three case files came from the prior headless reflection diagnosis.
The Python study treats mesh/plane intersections as surface events, without evaluating final material color or transmission chains.
The primary workload samples the three meshes; it does not reproduce the editor's full layer compositing or MSAA.

```sh
python3 -m venv /tmp/query-lab
/tmp/query-lab/bin/pip install -r requirements.txt
/tmp/query-lab/bin/python coverage_lab.py /tmp/query-data cases/full-02.json cases/full-03.json cases/full-32.json
/tmp/query-lab/bin/python make_gpu_packets.py /tmp/query-data
/tmp/query-lab/bin/python full_resolution_packet.py /tmp/query-data/dense.json
/tmp/query-lab/bin/python rebase_packet.py /tmp/query-data/dense.json /tmp/query-data/rebased.json
```

The GPU executable is in the rerun fork's `research/visibility-query` directory.
Its README documents the runs and the intentionally failing native candidate-filter mode.
`classify_boundaries.py` consumes the original dense packet and GPU mismatch details.
`rebase_packet.py` accepts `group` or `rebase` as its optional third argument for ablation; the default applies both.
Surface grouping in this script is specific to the two known coplanar 2D layers in this fixture.
It is not a general group detector or a completed material-compositing implementation.

`ideal_depth_layer_coverage` measures whether the known reference hit is directly present among the first N intersections from either probe.
It is not the accuracy of every possible reconstruction algorithm.
`hit_coverage` for the finite shell prototype requires the same surface ID and distance error below 0.5 scene units.
`false_hit_fraction` uses all query rays as its denominator.
An absent proxy candidate is not proof of an empty scene.
