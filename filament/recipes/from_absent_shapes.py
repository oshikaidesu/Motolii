#!/usr/bin/env python3
# AbsentMotion's shapes (app/src/main/assets/shapes.xml: Alight Motion's shape catalog, each a path, or a kind its own
# shapes.cpp draws from the properties given) as recipes, each a path shape: the paths as they are, the rest the path
# AbsentMotion's ShapePath makes of them (recipes/absent_shapes, run by the tool given); its numbers in Motolii's
# units (AbsentMotion's 200 points across a shape of size 100, Motolii's 1 across a thing of size 1), a contour it
# takes out as a hole (even-odd), a shape drawn by its edge alone by its edge (its colour; its width as wide at 1080 high).
# usage: from_absent_shapes.py shapes.xml tool out_dir
import re, subprocess, sys, xml.etree.ElementTree as ET

src, tool, out = sys.argv[1:4]
shapes = list(ET.parse(src).getroot())
asked = [s for s in shapes if s.get('s')]
said = lambda v: {'true': '1', 'false': '0'}.get(v, v)
made = subprocess.run([tool], input=''.join(f"{s.get('s')} " + ' '.join(f"{p.get('name')}={said(p.get('value'))}" for p in s.findall('property')) + '\n' for s in asked), capture_output=True, text=True, check=True).stdout.split('\n')
drawn = {s.get('name'): made[i] for i, s in enumerate(asked)}
small = lambda d: re.sub(r'-?\d+(?:\.\d+)?(?:e-?\d+)?', lambda m: f'{float(m.group()) / 200:.5g}', d)
for s in shapes:
    name = s.get('name')
    path = s.find('path')
    contours = [path] if path is not None and path.get('d') else (path.findall('contour') if path is not None else [])
    d = drawn.get(name) or ' '.join(c.get('d') for c in contours)
    lines = [f"// {name}: AbsentMotion's {name} shape (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), made by filament/absent_shapes.sh",
             '// make shape path', '// here', f'// say path {small(d)}']
    if any(c.get('exclude') == 'true' for c in contours):
        lines.append('// say fill_rule evenodd')
    if s.get('fillType') == 'none' and (edge := s.find('path-stroke')) is not None:
        colour = edge.find('color').get('value')
        lines += ['// say fill none', f'// say stroke {colour[-6:]}', f"// set stroke_width {float(edge.find('size').get('value')) / 200 * 0.5 * 1080 * 0.6229:.4g}"]
    slug = re.sub(r'[^a-z0-9]+', '_', name.lower()).strip('_')
    open(f'{out}/am_{slug}.js', 'w').write('\n'.join(lines) + '\n')
print(f'{len(shapes)} made')
