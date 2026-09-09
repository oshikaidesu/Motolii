"""Summarize paired samples; GPU timestamps remain diagnostics, not the adoption oracle."""
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
        assert data['pixel_equality']
        records.extend(dict(r, run=run) for r in data['records'])
    for scene in ['static', 'moving']:
        groups = [[r for r in records if r['scenario'] == scene and r['cache'] == mode] for mode in [False, True]]
        fields = ['total_us', 'prepare_us', 'submit_us', 'wait_us', 'readback_us', 'key_us', 'captures', 'hits', 'retained_input_texture_bytes']
        modes = [{k: statistics.median(r[k] for r in group) for k in fields} for group in groups]
        pairs = collections.defaultdict(dict)
        for r in records:
            if r['scenario'] == scene:
                pairs[(r['run'], r['frame'])][r['cache']] = r['total_us']
        deltas = [p[True] - p[False] for p in pairs.values()]
        rng = random.Random(20260909)
        boot = sorted(statistics.median(rng.choices(deltas, k=len(deltas))) for _ in range(4000))
        rows.append(dict(copies=count, scenario=scene, off=modes[0], on=modes[1],
            reduction_percent=100 * (1 - modes[1]['total_us'] / modes[0]['total_us']),
            paired_delta_median_us=statistics.median(deltas),
            paired_delta_bootstrap_95_us=[boot[100], boot[3899]],
            gpu_status=dict(collections.Counter(r['gpu_status'] for g in groups for r in g))))
summary = dict(gpu='Apple M4 (8 cores)', api='Metal / wgpu 29', resolution=[1280, 720],
    build='dev, unoptimized', samples_per_mode=40, independent_runs=2, warmup_per_run=3,
    pixel_equal_pairs_including_warmup=276,
    gpu_timing_reliable=False,
    gpu_note='Non-monotonic encoder timestamps observed; raw GPU values excluded from adoption decision.',
    cache_entries_limit=1, input_texture_limit_bytes=128*1024*1024,
    metadata_estimate_limit_bytes=8*1024*1024,
    capture_face_limit=512, rows=rows)
(root / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
for r in rows:
    print(r['copies'], r['scenario'], round(r['off']['total_us']/1000,3), '->',
        round(r['on']['total_us']/1000,3), 'ms;',
        'paired 95% interval', r['paired_delta_bootstrap_95_us'])
