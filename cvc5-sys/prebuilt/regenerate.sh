#!/usr/bin/env bash
# Regenerate the checked-in bindings that docs.rs consumes.
#
#   cvc5-sys/prebuilt/regenerate.sh           update prebuilt/*.rs in place
#   cvc5-sys/prebuilt/regenerate.sh --check   verify prebuilt/*.rs is current
#
# Run from the repository root, with cvc5 already built (the CVC5_LIB_DIR
# environment variable is honoured, as by cvc5-sys/build.rs).
#
# Output is normalized by normalize.py, so it does not matter whether this runs
# on Linux or macOS.
set -euo pipefail

check_only=false
[[ "${1:-}" == "--check" ]] && check_only=true

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
cd "$root"

# Ask cargo where the build script wrote its output rather than guessing, so a
# stale target/ directory cannot be picked up.
out_dir="$(cargo build -p cvc5-sys --features parser --message-format=json 2>/dev/null \
  | python3 -c '
import json, sys
out = None
for line in sys.stdin:
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "build-script-executed" and "cvc5-sys" in m.get("package_id", ""):
        out = m.get("out_dir")
if not out:
    sys.exit("could not determine cvc5-sys OUT_DIR from cargo output")
print(out)
')"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
for f in bindings.rs parser_bindings.rs; do
  [[ -f "$out_dir/$f" ]] || { echo "missing $out_dir/$f" >&2; exit 1; }
  cp "$out_dir/$f" "$tmp/$f"
done
python3 "$here/normalize.py" "$tmp/bindings.rs" "$tmp/parser_bindings.rs"

status=0
for f in bindings.rs parser_bindings.rs; do
  if $check_only; then
    if ! diff -u "$here/$f" "$tmp/$f"; then
      echo "prebuilt/$f is out of date" >&2
      status=1
    fi
  else
    if cmp -s "$here/$f" "$tmp/$f"; then
      echo "prebuilt/$f unchanged"
    else
      cp "$tmp/$f" "$here/$f"
      echo "prebuilt/$f updated"
    fi
  fi
done

if $check_only && [[ $status -ne 0 ]]; then
  cat >&2 <<'MSG'

The checked-in bindings no longer match the cvc5 submodule. Regenerate them:

    cvc5-sys/prebuilt/regenerate.sh

and commit the result. They are checked in because docs.rs builds without cvc5
available and reads them instead of running bindgen.
MSG
fi
exit $status
