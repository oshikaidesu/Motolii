#!/usr/bin/env python3
"""Read the workspace contract; detect a second core or developer-specific source path."""
import json,sys,re
from pathlib import Path

def read_only_document_dependency(source):
 source=source.split('[dev-dependencies]')[0]
 dependency=re.search(r'^motolii-doc\s*=\s*\{([^}]+)\}',source,re.M)
 fields=dependency.group(1) if dependency else ''
 requested=re.search(r'(?<![-\w])features\s*=\s*\[([^]]*)\]',fields,re.S)
 return bool(re.search(r'\bdefault-features\s*=\s*false\b',fields)) and not (requested and re.search(r'["\x27][^"\x27]+["\x27]',requested.group(1)))

if '--self-test' in sys.argv:
 cases=[
  ('motolii-doc = { path = "doc", default-features = false }',True),
  ('motolii-doc = { path = "doc" }',False),
  ('motolii-doc = { default-features = true }',False),
  ('motolii-doc = { default-features = false, features = ["editing"] }',False),
  ('motolii-doc = { default-features = false, features = ["default"] }',False),
  ('motolii-doc = { default-features = false, features = [\n "editing",\n] }',False),
  ('motolii-doc = { default-features = false, features = [] }',True),
  ('motolii-doc = { default-features = false }\n[dev-dependencies]\nmotolii-doc = { features = ["editing"] }',True),
  ('[dev-dependencies]\nmotolii-doc = { default-features = false }',False),
 ]
 for source,expected in cases:
  if bool(read_only_document_dependency(source)) != expected:
   raise SystemExit(f'Read-only dependency self-test failed: {source}')
 print(f'Stage 5 dependency rules: {len(cases)} passed')
 sys.exit(0)

root=Path(__file__).resolve().parents[1]
contract=json.loads((root/'docs/stage5/workspace.json').read_text())
errors=[]
for key in ['entry','ui','native','document','renderer','history']:
 if not (root/contract[key]).exists():errors.append(f'{key}: missing {contract[key]}')
manifest=(root/contract['native']/'Cargo.toml').read_text()
for name,key in [('motolii-doc','document'),('motolii-render','renderer')]:
 match=re.search(r'^'+re.escape(name)+r'\s*=\s*\{[^\n]*path\s*=\s*"([^"]+)"',manifest,re.M)
 if not match:
  errors.append(f'{name}: missing path dependency');continue
 resolved=(root/contract['native']/match.group(1)).resolve()
 if resolved!=(root/contract[key]).resolve():errors.append(f'{name}: points outside canonical core')
ui=root/contract['ui']
if list(ui.glob('**/crates/motolii-doc')) or list(ui.glob('**/crates/motolii-render')):errors.append('duplicate core beneath UI')
for folder in ['lib','native/src','macos/Runner']:
 for path in (ui/folder).rglob('*'):
  if path.is_file() and path.suffix in {'.dart','.rs','.swift'}:
   if '/Users/' in path.read_text():errors.append(f'developer path in {path.relative_to(root)}')
cargo=(root/contract['cargoWorkspace']).read_text()
build_inputs=[root/'Cargo.toml',root/'Cargo.lock',root/'.cargo/config.toml']
build_inputs += [root/contract[key]/'Cargo.toml' for key in ['native','document','renderer']]
for path in build_inputs:
 source=path.read_text()
 if re.search(r'file://|/Users/|/home/|/opt/homebrew/|/Library/Developer/',source):
  errors.append(f'machine-specific dependency/configuration in {path.relative_to(root)}')
 if path.name=='Cargo.toml' and re.search(r'\bpath\s*=\s*"(?:/|[A-Za-z]:[\\/])',source):
  errors.append(f'absolute dependency path in {path.relative_to(root)}')
match=re.search(r'^default-members\s*=\s*\[([^\]]+)\]',cargo,re.M)
if not match or re.findall(r'"([^"]+)"',match.group(1)) != [contract['cargoDefaultMember']]:
 errors.append('Root Cargo default must be the current Flutter native bridge')
members_match=re.search(r'^members\s*=\s*\[([^\]]+)\]',cargo,re.M)
if members_match and 'motolii' in re.findall(r'"([^"]+)"',members_match.group(1)):
 errors.append('Legacy Dioxus host must not rejoin the active workspace')
if (root/'motolii/Cargo.toml').exists():errors.append('Legacy host manifest must stay in Git history')
workflow=(root/'.github/workflows/ledger-fences.yml').read_text()
if 'app/Cargo.toml' in workflow or 'workspaces: next' in workflow:errors.append('Active CI invokes a historical workspace')
if 'scripts/check-stage5.py' not in workflow:errors.append('Active CI omits Stage 5 entry validation')
document_entries=list(contract.get('documents',{}).items())+[(p,p) for p in contract.get('entryDocuments',[])]
for role,relative in document_entries:
 path=root/relative
 if not path.is_file():
  errors.append(f'document {role}: missing {relative}');continue
 # Historical bodies remain link landing points, not active documentation dependencies.
 current=path.read_text().split('<details>')[0]
 for target in re.findall(r'\]\(([^)]+)\)',current):
  if target.startswith(('http:','https:','mailto:','#')):continue
  local=target.split('#')[0]
  if local and not (path.parent/local).exists():errors.append(f'{relative}: missing link {local}')
if set(contract.get('documents',{})) != {'concept','interaction','migration','recovery','technical'}:
 errors.append('Stage 5 must expose concept, interaction, migration, recovery and technical documents')
modules=json.loads((root/'docs/stage5/modules.json').read_text())
for consumer in modules['readOnlyDocumentConsumers']:
 if not read_only_document_dependency((root/consumer/'Cargo.toml').read_text()):
  errors.append(f'{consumer}: production dependency must not enable document editing')
read_paths=[]
for relative in [modules['readOnlyPlayback'],*modules.get('readOnlyModels',[])]:
 path=root/relative
 read_paths.extend(sorted(path.rglob('*.rs')) if path.is_dir() else [path])
for path in read_paths:
 relative=path.relative_to(root)
 source=re.sub(r'^#\[cfg\(test\)\]\s*mod\s+\w+\s*\{.*?^\}', '', path.read_text(), flags=re.M|re.S)
 imports=' '.join(re.findall(r'\buse\s+([^;]+);',source))
 if re.search(r'\b(?:Document|Intent|EditorRuntime)\b',imports) or re.search(r'\b(?:Document|Intent|EditorRuntime)::|::(?:Document|Intent|EditorRuntime)\b|::document::',source):
  errors.append(f'{relative}: read-side code must not depend on editing authority')
# The core may not keep state that outlives a read. A guard or a cache that hides in the
# process is a responsibility nobody holds: the view that recurses must own what it needs.
# The few process resources that are genuinely one per program are named in the contract.
process_state=modules.get('coreProcessState',{})
if process_state:
 core=root/process_state['root']
 owners={k:set(v) for k,v in process_state['owners'].items()}
 for path in sorted(core.rglob('*.rs')):
  relative=path.relative_to(root)
  name=str(path.relative_to(core))
  source=re.sub(r'#\[cfg\(test\)\][^\n]*\n','',path.read_text())
  for hit in re.findall(r'\bthread_local\s*!|\bstatic\s+mut\b',source):
   errors.append(f'{relative}: hidden process state in the core ({hit.strip()})')
  for held in re.findall(r'\bstatic\s+([A-Z_][A-Z0-9_]*)\s*:\s*[^;=]*\b(?:RefCell|Cell|Mutex|RwLock)\b',source):
   if held not in owners.get(name,set()):
    errors.append(f'{relative}: {held} is process-wide mutable state the contract does not name')
 for name,held in owners.items():
  if not (core/name).is_file():errors.append(f'coreProcessState: missing {name}')
# One solver, one owner. A layer's box, its time and its text must not learn the flow's
# solver by name: the day a second file says `taffy`, the responsibility has two homes.
# The core registers no effects, so a work made anywhere but the one door has none of them
# and fails silently. Only that door may make one.
registration=modules.get('effectRegistration')
if registration:
 house=root/registration['root']
 owner=registration['owner']
 if not (house/owner).is_file():errors.append(f"effectRegistration: missing {owner}")
 for path in sorted(house.rglob('*.rs')):
  name=str(path.relative_to(house))
  if name==owner:continue
  source=re.sub(r'#\[cfg\(test\)\]\s*mod\s+\w+\s*\{.*?^\}', '', path.read_text(), flags=re.M|re.S)
  source='\n'.join(l for l in source.split('\n') if not l.lstrip().startswith('//'))
  for call in registration['calls']:
   if re.search(r'(?<![\w:])'+re.escape(call)+r'\s*\(',source):
    errors.append(f'{path.relative_to(root)}: a work must be made through {owner}, not {call}')
for solver,owner in modules.get('coreSolvers',{}).get('owners',{}).items():
 core=root/modules['coreSolvers']['root']
 if not (core/owner).is_file():errors.append(f'coreSolvers: missing {owner}')
 for path in sorted(core.rglob('*.rs')):
  name=str(path.relative_to(core))
  if name==owner:continue
  source=path.read_text()
  body='\n'.join(l for l in source.split('\n') if not l.lstrip().startswith('//'))
  if re.search(r'\b'+re.escape(solver)+r'\b',body):
   errors.append(f'{path.relative_to(root)}: only {owner} may name the {solver} solver')
for relative,allowed in modules.get('isolatedExtensions',{}).items():
 path=root/relative/'Cargo.toml'
 source=path.read_text()
 dependencies=re.search(r'^\[dependencies\]\s*(.*?)(?=^\[|\Z)',source,re.M|re.S)
 names=set(re.findall(r'^([\w-]+)\s*=',dependencies.group(1),re.M)) if dependencies else set()
 if names!=set(allowed) or re.search(r'\b(?:path|git|package)\s*=',dependencies.group(1) if dependencies else ''):
  errors.append(f'{relative}: extension dependencies must stay isolated: {allowed}')
dart_root=root/modules['dartRoot']
for path in dart_root.rglob('*.dart'):
 rel=path.relative_to(dart_root)
 owner='app' if rel.name=='main.dart' and len(rel.parts)==1 else rel.parts[0]
 if owner not in modules['modules']:
  errors.append(f'Unowned UI source: {rel}');continue
 allowed=set(modules['modules'][owner]['dependsOn'])|{owner}
 source=path.read_text()
 for target in re.findall(r"(?:import|export|part)\s+['\"]([^'\"]+)['\"]",source):
  if target.startswith(('dart:','package:')):continue
  resolved=(path.parent/target).resolve()
  try: target_owner=resolved.relative_to(dart_root.resolve()).parts[0]
  except ValueError:
   errors.append(f'{rel}: dependency outside current UI: {target}');continue
  if target_owner not in allowed:errors.append(f'{rel}: {owner} cannot depend on {target_owner}')
 if owner!='bridge' and re.search(r'\bMethodChannel\s*\(|\.invokeMethod\s*\(',source):errors.append(f'{rel}: native transport must go through bridge')
 if owner!='input' and 'ClampingScrollSimulation(' in source:errors.append(f'{rel}: momentum must go through input')
 if owner!='session' and re.search(r'\bdocument\.value\s*=',source):errors.append(f'{rel}: snapshot writes belong to session')
for path in (root/'motolii/ui/native/src').rglob('*.rs'):
 if 'crate::ui::' in path.read_text():errors.append(f'{path.relative_to(root)}: old UI namespace')
if (root/'motolii/ui/native/src/ui').exists():errors.append('Native editor has an ambiguous UI duplicate')
for error in errors:print(error,file=sys.stderr)
print('Stage 5 workspace: '+('FAIL' if errors else 'PASS'))
sys.exit(bool(errors))
