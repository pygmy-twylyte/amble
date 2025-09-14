#!/usr/bin/env bash
set -euo pipefail

# Validate the Tree-sitter grammar and queries for Amble DSL.
#
# Usage:
#   bash scripts/ts_validate.sh [sample.amble]
#
# If no sample file is provided, tries test/sample.amble in the grammar repo,
# then falls back to the first .amble file found in this workspace.

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
GRAMMAR_DIR="$ROOT_DIR/tree-sitter-amble-script"

if [[ ! -d "$GRAMMAR_DIR" ]]; then
  echo "ERROR: Expected grammar at $GRAMMAR_DIR" >&2
  exit 1
fi

# Prefer local CLI (no network). Fallback to npx if missing.
TS_CLI="$GRAMMAR_DIR/node_modules/.bin/tree-sitter"
if [[ ! -x "$TS_CLI" ]]; then
  TS_CLI="npx tree-sitter"
  echo "Note: local tree-sitter CLI not found; using 'npx tree-sitter'." >&2
  echo "If this fails, run 'npm ci' inside $GRAMMAR_DIR first." >&2
fi

# Choose sample file
SAMPLE=${1:-}
if [[ -z "${SAMPLE:-}" ]]; then
  if [[ -f "$GRAMMAR_DIR/test/sample.amble" ]]; then
    SAMPLE="$GRAMMAR_DIR/test/sample.amble"
  else
    # Find first .amble in repo
    SAMPLE=$(rg -l --glob "**/*.amble" "$ROOT_DIR" | head -n1 || true)
  fi
fi

if [[ -z "${SAMPLE:-}" ]]; then
  echo "ERROR: No sample .amble file found. Provide a path as an argument." >&2
  exit 1
fi

echo "Grammar dir : $GRAMMAR_DIR"
echo "Sample file : $SAMPLE"

pushd "$GRAMMAR_DIR" >/dev/null

echo "[1/3] Generating parser (tree-sitter generate)"
$TS_CLI generate

echo "[2/3] Parsing sample"
$TS_CLI parse "$SAMPLE"

echo "[3/3] Verifying highlight queries"
# Grammar-bundled queries
if [[ -f "queries/highlights.scm" ]]; then
  $TS_CLI query "queries/highlights.scm" "$SAMPLE" >/dev/null
  echo "  ✓ queries/highlights.scm ok"
fi

# Zed-installed grammar queries
if [[ -f "$ROOT_DIR/zed-amble-script/grammars/amble_dsl/queries/highlights.scm" ]]; then
  $TS_CLI query "$ROOT_DIR/zed-amble-script/grammars/amble_dsl/queries/highlights.scm" "$SAMPLE" >/dev/null
  echo "  ✓ zed-amble-script/grammars/amble_dsl/queries/highlights.scm ok"
fi

# Language-level override (if present)
if [[ -f "$ROOT_DIR/zed-amble-script/languages/amble_dsl/highlights.scm" ]]; then
  $TS_CLI query "$ROOT_DIR/zed-amble-script/languages/amble_dsl/highlights.scm" "$SAMPLE" >/dev/null || true
  echo "  ✓ languages/amble_dsl/highlights.scm (compiled)"
fi

popd >/dev/null

echo "All checks passed. You can now Reload/Install the dev extension in Zed."

