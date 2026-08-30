#!/bin/bash
# Compile every rrdb translated .v file against the RocqOfRust base library.
#
# Usage:
#   ./rocq_compile_all.sh              # compile all translated .v files
#
# Environment:
#   ROCQ_OF_RUST_DIR  path to the rocq-of-rust repository
#                     (default: ~/programming/rocq-of-rust)
#
# Logs are written to rocq_logs/ (git-ignored). Error output is also
# echoed to stderr so failures are visible in CI.
set -u

export PATH="/opt/homebrew/bin:$PATH"
eval $(opam env --switch=rocq-of-rust)

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
ROCQ_OF_RUST_DIR="${ROCQ_OF_RUST_DIR:-$HOME/programming/rocq-of-rust}"
ROCQ_OF_RUST_LIB="$ROCQ_OF_RUST_DIR/RocqOfRust"
LOG_DIR="$REPO_ROOT/rocq_logs"

ROQC_FLAGS="-R $ROCQ_OF_RUST_LIB RocqOfRust -impredicative-set"

cd "$REPO_ROOT" || exit 2
mkdir -p "$LOG_DIR"

# Collect translated .v files (skip ignored/generated dirs).
V_FILES=""
while IFS= read -r f; do
  V_FILES="$V_FILES$f
"
done < <(find . -name "*.v" -not -path "./target/*" | sort)

if [ -z "$V_FILES" ]; then
  echo "No .v files found. Run 'cargo rocq-of-rust' first." >&2
  exit 1
fi

PASS=0
FAIL=0
FAILED_LIST=""

while IFS= read -r f; do
  [ -z "$f" ] && continue
  logname="${f#./}"
  logname="${logname//\//_}"
  log="$LOG_DIR/${logname}.log"
  if rocq compile $ROQC_FLAGS "$f" > "$log" 2>&1; then
    PASS=$((PASS + 1))
    echo "PASS $f"
  else
    FAIL=$((FAIL + 1))
    FAILED_LIST="$FAILED_LIST $f"
    echo "FAIL $f" >&2
    echo "--- rocq output for $f (also saved to $log) ---" >&2
    cat "$log" >&2
    echo "--- end rocq output for $f ---" >&2
  fi
done <<EOF
$V_FILES
EOF

echo "SUMMARY PASS=$PASS FAIL=$FAIL"

if [ "$FAIL" -gt 0 ]; then
  echo "FAILED FILES:$FAILED_LIST" >&2
  exit 1
fi
