"""Compare shared GPU buffers against per-view rebuilds at identical inputs."""
import collections
import json
from pathlib import Path
import random
import statistics

root = Path(__file__).resolve().parent
rows = []
for count in [10, 100, 1000]:
    records = []
    for run in [1, 2]:
        data = json.loads((root / f'{count}-{run}.json').read_text())
        assert data['pixel_equality'] and data['comparison'] == 'gpu-sharing'
        assert all(r['cache'] for r in data['records'])
        records.extend(dict(r, run=run) for r in data['records'])
    for scene in ['static', 'moving']:
        groups = [[r for r in records if r['scenario'] == scene and r['variant_enabled'] == mode] for mode in [False, True]]
        fields = ['total_us', 'prepare_us', 'resolve_us', 'layer_build_us', 'draw_data_prepare_us', 'submit_us', 'wait_us', 'readback_us', 'mesh_instances_uploaded', 'mesh_instance_upload_bytes', 'captures']
        modes = [{k: statistics.median(r[k] for r in group) for k in fields} for group in groups]
        pairs = collections.defaultdict(dict)
        for r in records:
            if r['scenario'] == scene:
                pairs[(r['run'], r['frame'])][r['variant_enabled']] = r['total_us']
        deltas = [p[True] - p[False] for p in pairs.values()]
        rng = random.Random(20260909)
        boot = sorted(statistics.median(rng.choices(deltas, k=len(deltas))) for _ in range(4000))
        rows.append(dict(copies=count, scenario=scene, per_view=modes[0], shared=modes[1],
            total_reduction_percent=100 * (1 - modes[1]['total_us'] / modes[0]['total_us']),
            paired_delta_median_us=statistics.median(deltas),
            paired_delta_bootstrap_95_us=[boot[100], boot[3899]],
            gpu_status=dict(collections.Counter(r['gpu_status'] for g in groups for r in g))))
summary = dict(gpu='Apple M4 (8 cores)', api='Metal / wgpu 29', resolution=[1280, 720],
    build='dev, unoptimized', samples_per_mode=40, independent_runs=2, warmup_per_run=3,
    pixel_equal_pairs_including_warmup=276, reflection_cache_enabled_in_both_modes=True,
    fork='a97e03916c7bf6f03307b3cf2f9710ecebf2642b', gpu_timing_reliable=False,
    cpu_source_mapping_bytes_per_instance=32, gpu_instance_bytes=156,
    rows=rows)
(root / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
for r in rows:
    print(r['copies'], r['scenario'], round(r['per_view']['total_us']/1000,3), '->',
        round(r['shared']['total_us']/1000,3), 'ms;',
        'paired 95% interval', r['paired_delta_bootstrap_95_us'])
