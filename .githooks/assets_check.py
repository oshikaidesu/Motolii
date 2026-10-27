#!/usr/bin/env python3
# Is ASSETS.md (the ledger of what is at hand) still true? Every file git knows must be covered by a place the ledger
# names (the file itself, its name, a folder above it, or a pattern with *), and every place the ledger names must
# still be there. Run by the pre-commit hook; or by hand:  .githooks/assets_check.py
import fnmatch, glob, os, re, subprocess, sys
root = subprocess.run(['git', 'rev-parse', '--show-toplevel'], capture_output=True, text=True).stdout.strip()
os.chdir(root)
ledger = open('ASSETS.md', encoding='utf-8').read()
# (a place is what stands between ` ` and looks like a path. A bare name after a place on the same line is in that
#  place's folder: `filament/glass_live.h`, `glass_live.mm`)
places = []
for line in ledger.split('\n'):
    folder = ''
    for token in re.findall(r'`([^`\n]+)`', line):
        token = token.strip()
        whole = token.endswith('/')   # (a folder named with its / : all that is in it)
        token = token.rstrip('/')
        if ' ' in token or not (whole or '/' in token or re.search(r'\.[A-Za-z0-9]{1,6}$', token) or os.path.isfile(token)):   # (or a file with no extension, as Brewfile)
            continue
        if '/' not in token and not whole and folder:
            token = folder + '/' + token
        else:
            folder = token if whole or not (re.search(r'\.[A-Za-z0-9*]{1,6}$', os.path.basename(token)) or os.path.isfile(token)) else os.path.dirname(token)
        places.append(token)
files = subprocess.run(['git', 'ls-files', '--cached', '--others', '--exclude-standard'], capture_output=True, text=True).stdout.split('\n')
# (not things one would look for: pictures and films, other people's sources, build output, the reports themselves)
skip = re.compile(r'(^|/)(third_party|target|build|out|\.dart_tool|Pods|ephemeral)/|^reports/|^research_notes/|\.(png|jpe?g|mp4|ico|icns|lock|g\.dart)$|(^|/)\.[^/]+$')
files = [f for f in files if f and not skip.search(f) and os.path.exists(f)]
def covered(f):
    base = os.path.basename(f)
    for p in places:
        if p.startswith('~'):
            continue
        if f == p or base == p or f.startswith(p + '/') or fnmatch.fnmatch(f, p) or fnmatch.fnmatch(base, p):
            return True
    return False
missing = [f for f in files if not covered(f)]
names = {os.path.basename(f) for f in files} | set(files)
ever = None
def tracked_once(p):   # (git has held it at some time: a place only ever on one machine is not looked for in a clone)
    global ever
    if ever is None:
        ever = set(subprocess.run(['git', 'log', '--format=', '--name-only', 'HEAD'], capture_output=True, text=True).stdout.split('\n'))
    q = p.rstrip('/')
    return any(f == q or f.startswith(q + '/') or fnmatch.fnmatch(f, q) for f in ever)
def there(p):
    if glob.glob(os.path.expanduser(p)):
        return True
    if not p.startswith('~') and not tracked_once(p):
        return True   # (never in git: kept on this machine only, as bridge/ or research_notes/)
    if '/out/' in '/' + p + '/':
        return True   # (build output and footage put there by hand: made or placed when asked for)
    if not p.startswith('~') and subprocess.run(['git', 'check-ignore', '-q', p.rstrip('/')], stderr=subprocess.DEVNULL).returncode == 0:
        return True   # (kept out of git on purpose, here only on this machine: pictures and films of the reports, fetched sources)
    return False
gone = sorted({p for p in places if not there(p)})
if missing:
    by = {}
    for f in missing:
        by.setdefault(os.path.dirname(f) or '.', []).append(os.path.basename(f))
    print('ASSETS.md does not say where these are (%d files):' % len(missing))
    for d in sorted(by):
        print('  %s/: %s' % (d, ', '.join(sorted(by[d])[:14]) + (' ... and %d more' % (len(by[d]) - 14) if len(by[d]) > 14 else '')))
if gone:
    print('ASSETS.md names these, and they are not there any more (%d):' % len(gone))
    for p in gone:
        print('  ' + p)
if missing or gone:
    print('Put it right in ASSETS.md (a row for a new thing, or a folder or pattern that covers it; take out what is gone), then commit again.')
    sys.exit(1)
print('ASSETS.md covers all %d files, and all %d places it names are there.' % (len(files), len(places)))
