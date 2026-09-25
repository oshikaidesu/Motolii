#!/usr/bin/env python3
"""Motolii owns meaning; its Rerun host owns GPU execution. Two invariants, checked structurally:

  motolii  Production code does not grow GPU execution infrastructure: no raw pipelines, bind
           groups, textures, buffers or samplers, no submission, no poll/map/synchronous readback, no
           renderer of its own, no pool of GPU resources. Encoders are recordings the host submits
           (their command buffers go to `queue_commands` in the same function). Exceptions are the
           responsibility boundaries in motolii/reference/host-ownership.json (EMBEDDER, OFFLINE,
           SEMANTIC_LOWERING), never lines.
  fork     The pinned Rerun fork adds to the host only what the same file classifies as a generic
           primitive family: every public Rust item, WGSL function, global binding and frame
           uniform field it adds over upstream must belong to a family, and no identifier it adds
           may carry Motolii's product vocabulary. A renamed item is a new item: it needs a family.

usage: scripts/check-host-ownership.py [--self-test] [--fork PATH] [--list-fork]
"""
import fnmatch, json, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RULES = json.loads((ROOT / 'motolii/reference/host-ownership.json').read_text())
FIXTURES = ROOT / 'scripts/fixtures/host-ownership'

# ---------------------------------------------------------------- motolii: GPU execution primitives
PRIMITIVES = {
    'raw pipeline': r'\.create_(?:render|compute)_pipeline\s*\(|\.create_shader_module\w*\s*\(|\.create_(?:pipeline|bind_group)_layout\s*\(',
    'raw bind group': r'\.create_bind_group\s*\(',
    'raw allocation': r'\.create_(?:texture|buffer)\w*\s*(?:::<[^>]*>\s*)?\(|\.create_sampler\s*\(|\.create_query_set\s*\(',
    'submission': r'\bqueue\s*\.\s*submit\s*\(|\.submit\s*\(\s*\[|\bbefore_submit\s*\(|\bctx\s*\.\s*begin_frame\s*\(|on_submitted_work_done',
    'gpu wait/map': r'(?:device\w*|device\(\))\s*\.\s*poll\s*\(|\bPollType::|\.map_async\s*\(|\.get_mapped_range\w*\s*\(|map_buffer_on_submit',
    'renderer of its own': r'\bimpl\b[^{;]*\b(?:Renderer|DrawData)\s+for\b',
    'pool of GPU resources': r'^\s*(?:pub(?:\([^)]*\))?\s+)?\w+\s*:\s*[^=;(]*\b(?:Vec|VecDeque|HashMap|BTreeMap|HashSet)<[^;]*\b(?:wgpu::(?:Texture|Buffer|BindGroup|RenderPipeline|ComputePipeline)|GpuTexture|GpuTexture2D|GpuBuffer|GpuBindGroup)\b',
}
ENCODER = r'\.create_command_encoder\s*\('
TEST_FILE = re.compile(r'(^|/)(tests?|[\w]*_tests?|[\w]*_contract|[\w]*_probe)\.rs$|/tests/')
ITEM = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:(?:async|unsafe|const|extern\s+"C")\s+)*(fn|struct|enum|impl|trait|mod)\b\s*(?:<[^>]*>\s*)?([\w:]+)?')


def items_of(lines):
    """For each line, the innermost enclosing named item (fn/struct/impl/...) and its line span."""
    stack, depth, enclosing, pending = [], 0, [], None
    for index, line in enumerate(lines):
        code = re.sub(r'//.*', '', line)
        match = ITEM.match(code)
        if match and match.group(1) != 'mod' or (match and '{' in code):
            name = match.group(2) or ''
            if match.group(1) == 'impl':  # `impl X for Y` / `impl Y`: the type is the item
                target = re.search(r'\bfor\s+([\w:]+)', code) or re.search(r'impl\s*(?:<[^>]*>\s*)?([\w:]+)', code)
                name = target.group(1) if target else name
            pending = (name.split('::')[-1], depth)
        enclosing.append([entry[0] for entry in stack] + ([pending[0]] if pending else []))
        for ch in code:
            if ch == '{':
                depth += 1
                if pending:
                    stack.append((pending[0], pending[1], index))
                    pending = None
            elif ch == '}':
                depth -= 1
                while stack and depth <= stack[-1][1]:
                    stack.pop()
        if pending and code.rstrip().endswith(';'):
            pending = None
    return enclosing


def function_body(lines, index):
    """The text of the function enclosing `lines[index]` (brace-balanced from its `fn`)."""
    start = index
    while start > 0 and not re.match(r'\s*(?:pub(?:\([^)]*\))?\s+)?fn\b', lines[start]):
        start -= 1
    depth, opened, end = 0, False, start
    while end < len(lines):
        code = re.sub(r'//.*', '', lines[end])
        depth += code.count('{') - code.count('}')
        opened = opened or '{' in code
        if opened and depth <= 0:
            break
        end += 1
    return '\n'.join(lines[start:end + 1])


def production_lines(text):
    """(line number, line) outside `#[cfg(test)]` items and comments."""
    pending, depth, opened = False, 0, False
    for number, line in enumerate(text.split('\n'), 1):
        if re.match(r'\s*#\[cfg\(test\)\]', line):
            pending, depth, opened = True, 0, False
            continue
        if pending:
            code = re.sub(r'//.*', '', line)
            depth += code.count('{') - code.count('}')
            opened = opened or '{' in code
            if (opened and depth <= 0) or (not opened and code.rstrip().endswith(';')):
                pending = False
            continue
        if not line.strip().startswith('//'):
            yield number, line


def boundary_for(path, names):
    for boundary in RULES['motolii']['boundaries']:
        if boundary['file'] == path and ('item' not in boundary or boundary['item'] in names):
            return boundary
    return None


def check_motolii_source(path, text):
    """Violations in one production source file (as it would be at `path`)."""
    if TEST_FILE.search(path):
        return []
    lines = text.split('\n')
    enclosing = items_of(lines)
    structs = set(re.findall(r'\bstruct\s+(\w+)', text))
    found = []
    for number, line in production_lines(text):
        kinds = [kind for kind, pattern in PRIMITIVES.items() if re.search(pattern, line)]
        # A collection of GPU resources is a pool only as a field a type keeps, not as an argument.
        if 'pool of GPU resources' in kinds and not (enclosing[number - 1] and enclosing[number - 1][-1] in structs):
            kinds.remove('pool of GPU resources')
        if re.search(ENCODER, line) and 'queue_commands(' not in function_body(lines, number - 1):
            kinds.append('encoder the host does not submit')
        for kind in kinds:
            if boundary_for(path, enclosing[number - 1]) is None:
                found.append(f'{path}:{number}: {kind}: {line.strip()[:110]}')
    return found


def retired_in(where, identifiers, fork):
    """Names of the retired architecture among `identifiers` (Motolii's own meaning may keep the fork-only ones)."""
    retired = RULES['retired']
    names = set(retired['names']) - (set() if fork else set(retired['fork_only']))
    return [f'{where}: `{token}` is retired architecture (motolii/reference/host-ownership.json)' for token in sorted(set(identifiers) & names)]


def check_motolii():
    files = subprocess.run(['git', 'ls-files', *RULES['motolii']['sources']], cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
    found = []
    for path in files:
        text = (ROOT / path).read_text()
        found += check_motolii_source(path, text)
        if not TEST_FILE.search(path):
            found += retired_in(path, re.findall(r'[A-Za-z_]\w*', '\n'.join(line for _, line in production_lines(text))), fork=False)
    return found

# ---------------------------------------------------------------- fork: the host's surface
def crate_path(path):
    """`crates/<group>/<crate>/src/x.rs` -> `<crate>/src/x.rs`."""
    parts = path.split('/')
    return '/'.join(parts[2:]) if parts[0] == 'crates' and len(parts) > 3 else path


def words(identifier):
    return [w.lower() for w in re.findall(r'[A-Z]+(?![a-z])|[A-Z]?[a-z]+|\d+', identifier)]


def diff_additions(diff, base_text, new_text=None):
    """(file, kind, name) the diff adds to the host's surface, and the identifiers on its added lines."""
    added, identifiers, path, in_uniform, shaders = [], [], None, False, {}
    for line in diff.split('\n'):
        if line.startswith('+++ '):
            path = line[6:] if line.startswith('+++ b/') else None
            in_uniform = False
            continue
        if path is None or not line.startswith('+') or line.startswith('+++'):
            if path and path.endswith('.wgsl') and line.startswith(' ') and re.match(r'\s*struct\s+FrameUniformBuffer', line[1:]):
                in_uniform = True
            continue
        code = re.sub(r'//.*', '', line[1:])
        where, base = crate_path(path), base_text(path)
        identifiers += [(where, token) for token in re.findall(r'[A-Za-z_]\w*', code)]
        if path.endswith('.rs'):
            match = re.match(r'\s*pub\s+(?:(?:async|unsafe|const)\s+)*(fn|struct|enum|const|type|mod|trait|static)\s+(\w+)', code)
            if match and not re.search(r'\bpub\s+' + match.group(1) + r'\s+' + match.group(2) + r'\b', base):
                added.append((where, match.group(1), match.group(2)))
                continue
            match = re.match(r'\s*pub\s+(\w+)\s*:', code)
            if match and not re.search(r'\bpub\s+' + match.group(1) + r'\s*:', base):
                added.append((where, 'field', match.group(1)))
            match = re.match(r'\s*impl\b.*\b(Renderer|DrawData)\s+for\s+(\w+)', code)
            if match and not re.search(r'(Renderer|DrawData)\s+for\s+' + match.group(2) + r'\b', base):
                added.append((where, 'impl ' + match.group(1), match.group(2)))
            match = re.match(r'\s*pub\s+use\s+(.*)', code)
            if match:
                for name in re.findall(r'\b([A-Za-z_]\w*)\b', match.group(1).split('::', 1)[-1]):
                    if name not in ('self', 'crate', 'super') and not re.search(r'pub\s+use[^;]*\b' + name + r'\b', base, re.S):
                        added.append((where, 'reexport', name))
        elif path.endswith('.wgsl'):
            shaders.setdefault(path, []).append(line[1:])
    for path, lines in shaders.items():
        new = new_text(path) if new_text else '\n'.join(lines)
        base = wgsl_surface(base_text(path))
        for kind, names in wgsl_surface(new).items():
            added += [(crate_path(path), kind, name) for name in sorted(names - base[kind])]
    return added, identifiers


def wgsl_surface(text):
    """What a shader file offers every other shader: functions, group-0 bindings, frame uniform fields."""
    code = re.sub(r'//.*', '', text)
    uniform = re.search(r'struct\s+FrameUniformBuffer\s*\{(.*?)\}', code, re.S)
    return {
        'wgsl fn': set(re.findall(r'\bfn\s+(\w+)', code)),
        'global binding': set(re.findall(r'@group\(0\)\s*@binding\(\d+\)\s*var(?:<[^>]*>)?\s+(\w+)', code)),
        'frame uniform': set(re.findall(r'(\w+)\s*:', uniform.group(1))) if uniform else set(),
    }


def family_of(where, name):
    for family in RULES['fork']['families']:
        if any(fnmatch.fnmatch(where, pattern) for pattern in family.get('owns', [])):
            return family['name']
        if any(fnmatch.fnmatch(f'{where}:{name}', pattern) for pattern in family.get('items', [])):
            return family['name']
    return None


def check_fork_diff(diff, base_text=lambda path: '', new_text=None):
    added, identifiers = diff_additions(diff, base_text, new_text)
    found = [f'{where}: {kind} `{name}` belongs to no generic primitive family (motolii/reference/host-ownership.json)'
             for where, kind, name in added if family_of(where, name) is None]
    for where in sorted({where for where, _ in identifiers}):
        found += retired_in(where, [token for w, token in identifiers if w == where], fork=True)
    vocabulary = set(RULES['fork']['vocabulary'])
    for where, token in sorted(set(identifiers)):
        hit = vocabulary.intersection(words(token))
        if hit:
            found.append(f'{where}: identifier `{token}` carries Motolii product vocabulary ({", ".join(sorted(hit))})')
    return found, added


def fork_root():
    if '--fork' in sys.argv:
        return Path(sys.argv[sys.argv.index('--fork') + 1])
    metadata = subprocess.run(['cargo', 'metadata', '--format-version', '1', '--locked'], cwd=ROOT, capture_output=True, text=True)
    if metadata.returncode != 0:
        raise SystemExit('host ownership: cargo metadata failed (fetch the pinned fork first): ' + metadata.stderr[-400:])
    for package in json.loads(metadata.stdout)['packages']:
        if package['name'] == 're_renderer':
            path = Path(package['manifest_path']).parent
            while not (path / '.git').exists() and path != path.parent:
                path = path.parent
            return path
    raise SystemExit('host ownership: re_renderer is not a dependency')


def check_fork():
    root, upstream = fork_root(), RULES['fork']['upstream']
    git = lambda *args: subprocess.run(['git', '-C', str(root), *args], capture_output=True, text=True)
    if git('merge-base', '--is-ancestor', upstream, 'HEAD').returncode != 0:
        return [f'fork {root}: the recorded upstream {upstream[:10]} is not an ancestor of the pinned rev: rebase the record'], []
    diff = git('diff', upstream, 'HEAD', '--', 'crates', ':!crates/**/tests/**').stdout
    bases = {}
    def base_text(path):
        if path not in bases:
            bases[path] = git('show', f'{upstream}:{path}').stdout
        return bases[path]
    return check_fork_diff(diff, base_text, lambda path: git('show', f'HEAD:{path}').stdout)

# ---------------------------------------------------------------- self-test: the gate must bite
def self_test():
    failures, cases = [], 0
    for fixture in sorted(FIXTURES.glob('motolii/*.rs')):
        text = fixture.read_text()
        path = re.match(r'// as: (\S+)', text).group(1)
        red = bool(check_motolii_source(path, text) or retired_in(path, re.findall(r'[A-Za-z_]\w*', '\n'.join(l for _, l in production_lines(text))), fork=False))
        cases += 1
        if red != fixture.name.startswith('red_'):
            failures.append(f'{fixture.name}: expected {"red" if fixture.name.startswith("red_") else "green"}, got {"red" if red else "green"}')
    for fixture in sorted(FIXTURES.glob('fork/*.diff')):
        red = bool(check_fork_diff(fixture.read_text())[0])
        cases += 1
        if red != fixture.name.startswith('red_'):
            failures.append(f'{fixture.name}: expected {"red" if fixture.name.startswith("red_") else "green"}, got {"red" if red else "green"}')
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f'host ownership self-test: {cases - len(failures)}/{cases} fixtures as expected')
    return not failures


def main():
    if '--self-test' in sys.argv:
        sys.exit(0 if self_test() else 1)
    motolii = check_motolii()
    fork, added = check_fork()
    if '--list-fork' in sys.argv:
        for where, kind, name in added:
            print(f'{family_of(where, name) or "-":28} {where}: {kind} {name}')
    for line in motolii + fork:
        print(line, file=sys.stderr)
    print(f'host ownership: motolii {len(motolii)} violation(s), fork {len(fork)} violation(s) over {len(added)} additions: ' + ('FAIL' if motolii or fork else 'PASS'))
    sys.exit(1 if motolii or fork else 0)


if __name__ == '__main__':
    main()
