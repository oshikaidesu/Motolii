#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATE="$ROOT_DIR/scripts/validate.sh"

fail() {
  echo "test-validate: $*" >&2
  exit 1
}

assert_status() {
  local expected="$1"
  shift
  set +e
  "$@" >/dev/null 2>&1
  local actual=$?
  set -e
  [[ "$actual" -eq "$expected" ]] ||
    fail "expected status $expected, got $actual: $*"
}

list_output="$("$VALIDATE" --list)"
grep -Fqx "  docs policy tooling rust web-build web-contract web-visual" \
  <<<"$list_output" || fail "lane closed set is missing or reordered"
grep -Fqx "  local: docs rust" \
  <<<"$list_output" || fail "local profile is missing or reordered"

[[ "$("$VALIDATE" --plan local)" == "local: docs rust" ]] ||
  fail "local plan drifted"

assert_status 2 "$VALIDATE"
assert_status 2 "$VALIDATE" unknown
assert_status 2 "$VALIDATE" --list unexpected
assert_status 2 "$VALIDATE" --plan unknown
assert_status 2 "$VALIDATE" policy
assert_status 2 "$VALIDATE" policy ""
assert_status 2 "$VALIDATE" policy --files-from
assert_status 2 "$VALIDATE" policy one two
assert_status 1 "$VALIDATE" policy refs/heads/motolii-validation-missing-base

for lane in docs tooling rust web-build web-contract web-visual; do
  assert_status 2 "$VALIDATE" "$lane" unexpected
done

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/motolii-validate.XXXXXX")"
trap 'rm -rf "$temp_dir"' EXIT
mkdir -p "$temp_dir/bin"
cat >"$temp_dir/bin/npm" <<'EOF'
#!/usr/bin/env bash
exit 23
EOF
chmod +x "$temp_dir/bin/npm"
assert_status 23 env PATH="$temp_dir/bin:$PATH" "$VALIDATE" web-build

mkdir -p "$temp_dir/no-command-bin"
ln -s "$(command -v bash)" "$temp_dir/no-command-bin/bash"
ln -s "$(command -v dirname)" "$temp_dir/no-command-bin/dirname"
assert_status 127 env PATH="$temp_dir/no-command-bin" "$VALIDATE" web-build

echo "test-validate: PASS"
