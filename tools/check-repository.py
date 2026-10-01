from pathlib import Path
root=Path(__file__).resolve().parents[1]
required=['app/document/Cargo.toml','app/renderer/Cargo.toml','app/ui/pubspec.yaml','app/ui/native/Cargo.toml','docs/current/README.md','tools/motolii-ui.sh']
missing=[p for p in required if not (root/p).is_file()]
if missing: raise SystemExit('Missing: '+', '.join(missing))
manifest=(root/'Cargo.toml').read_text()
for path in ['app/document','app/renderer','app/ui/native']:
 if path not in manifest: raise SystemExit('Cargo workspace misses '+path)
print('OK: Motolii folders')
