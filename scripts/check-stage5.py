#!/usr/bin/env python3
"""Read the workspace contract; detect a second core or developer-specific source path."""
import json,sys,re
from pathlib import Path
root=Path(__file__).resolve().parents[1]
contract=json.loads((root/'docs/stage5/workspace.json').read_text())
errors=[]
for key in ['entry','ui','native','document','renderer','legacy_ui']:
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
match=re.search(r'^default-members\s*=\s*\[([^\]]+)\]',cargo,re.M)
if not match or re.findall(r'"([^"]+)"',match.group(1)) != [contract['cargoDefaultMember']]:
 errors.append('Root Cargo default must be the current Flutter native bridge')
members_match=re.search(r'^members\s*=\s*\[([^\]]+)\]',cargo,re.M)
if members_match and 'motolii' in re.findall(r'"([^"]+)"',members_match.group(1)):
 errors.append('Legacy Dioxus host must not rejoin the active workspace')
if '[workspace]' in (root/'motolii/Cargo.toml').read_text():errors.append('Duplicate current Cargo workspace')
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
