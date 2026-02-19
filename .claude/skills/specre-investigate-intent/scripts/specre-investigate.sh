#!/usr/bin/env bash
# specre-investigate.sh — CLI-based specre investigation helper for non-MCP environments
#
# Usage:
#   specre-investigate.sh --query "<noun> <verb>"         # free-text search
#   specre-investigate.sh --file <path>                   # extract @specre tags, then trace
#   specre-investigate.sh --ulid <ULID>                   # trace a known ULID directly
#   specre-investigate.sh --file <path> --query "<terms>" # file-first with fallback query
#
# Output: structured text for Claude to interpret (card content, search results, file markers)
#
# Exit codes:
#   0  — found at least one relevant specre card
#   1  — no card found (caller should fall back to grep/glob)
#   2  — specre CLI not found or ecosystem not healthy

set -euo pipefail

# ── Helpers ──────────────────────────────────────────────────────────────────

die() { echo "ERROR: $*" >&2; exit 2; }
log() { echo "==> $*" >&2; }
sep() { printf '%s\n' "---"; }

require_specre() {
    command -v specre >/dev/null 2>&1 || die "'specre' not found in PATH."
}

# ── Argument parsing ─────────────────────────────────────────────────────────

FILE=""
QUERY=""
ULID=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --file)   FILE="${2:?'--file requires a path'}";   shift 2 ;;
        --query)  QUERY="${2:?'--query requires a string'}"; shift 2 ;;
        --ulid)   ULID="${2:?'--ulid requires a ULID'}";   shift 2 ;;
        --)       shift; break ;;
        *)        die "Unknown argument: $1" ;;
    esac
done

[[ -z "$FILE" && -z "$QUERY" && -z "$ULID" ]] && die "Provide at least one of --file, --query, or --ulid"

require_specre

# ── Phase 0: Health check ────────────────────────────────────────────────────

log "Running specre health-check..."
HEALTH_JSON=$(specre health-check --json 2>/dev/null || echo '{"healthy":false}')
HEALTHY=$(printf '%s' "$HEALTH_JSON" | grep -o '"healthy":[^,}]*' | grep -o 'true\|false' || echo "false")

printf 'HEALTH_CHECK: healthy=%s\n' "$HEALTHY"
sep

if [[ "$HEALTHY" != "true" ]]; then
    echo "WARNING: specre ecosystem is not healthy. Results may be incomplete."
    sep
fi

FOUND=0

# ── Phase 1: File-based investigation ────────────────────────────────────────

if [[ -n "$FILE" ]]; then
    if [[ ! -f "$FILE" ]]; then
        echo "WARNING: File not found: $FILE"
        sep
    else
        log "Scanning '$FILE' for @specre markers..."
        MARKERS=$(grep -oP '@specre \K[A-Z0-9]{26}' "$FILE" 2>/dev/null || true)

        if [[ -n "$MARKERS" ]]; then
            printf 'MARKERS_FOUND_IN: %s\n' "$FILE"
            while IFS= read -r ulid; do
                printf 'ULID: %s\n' "$ulid"
                log "Tracing ULID $ulid..."
                specre trace "$ulid" 2>/dev/null || echo "(trace failed for $ulid)"
                sep
                FOUND=1
            done <<< "$MARKERS"
        else
            printf 'NO_MARKERS_IN: %s\n' "$FILE"
            sep

            # Derive search terms from file name if no explicit query
            if [[ -z "$QUERY" ]]; then
                BASENAME=$(basename "$FILE" | sed 's/\.[^.]*$//' | tr '_-' ' ')
                log "No markers found. Deriving search from filename: '$BASENAME'"
                QUERY="$BASENAME"
            fi
        fi
    fi
fi

# ── Phase 2: Direct ULID trace ───────────────────────────────────────────────

if [[ -n "$ULID" ]]; then
    log "Tracing ULID: $ULID"
    printf 'TRACE_ULID: %s\n' "$ULID"
    specre trace "$ULID" 2>/dev/null || echo "(trace failed for $ULID)"
    sep
    FOUND=1
fi

# ── Phase 3: Search mode (up to 3 rounds) ────────────────────────────────────

if [[ -n "$QUERY" && "$FOUND" -eq 0 ]]; then
    log "Starting search for: '$QUERY'"

    # Round 1: AND query (most specific)
    printf 'SEARCH_ROUND=1 QUERY="%s"\n' "$QUERY"
    RESULT=$(specre search "$QUERY" --json 2>/dev/null || echo '{"total":0,"results":[]}')
    TOTAL=$(printf '%s' "$RESULT" | grep -o '"total":[0-9]*' | grep -o '[0-9]*' || echo "0")
    printf 'RESULTS: %s\n' "$TOTAL"
    printf '%s\n' "$RESULT"
    sep

    if [[ "$TOTAL" -gt 0 ]]; then
        FOUND=1
    else
        # Round 2: first keyword only
        FIRST_KEYWORD=$(printf '%s' "$QUERY" | awk '{print $1}')
        printf 'SEARCH_ROUND=2 QUERY="%s"\n' "$FIRST_KEYWORD"
        RESULT=$(specre search "$FIRST_KEYWORD" --json 2>/dev/null || echo '{"total":0,"results":[]}')
        TOTAL=$(printf '%s' "$RESULT" | grep -o '"total":[0-9]*' | grep -o '[0-9]*' || echo "0")
        printf 'RESULTS: %s\n' "$TOTAL"
        printf '%s\n' "$RESULT"
        sep

        if [[ "$TOTAL" -gt 0 ]]; then
            FOUND=1
        else
            # Round 3: OR query (widest net)
            printf 'SEARCH_ROUND=3 QUERY="%s" (--or)\n' "$QUERY"
            RESULT=$(specre search "$QUERY" --or --json 2>/dev/null || echo '{"total":0,"results":[]}')
            TOTAL=$(printf '%s' "$RESULT" | grep -o '"total":[0-9]*' | grep -o '[0-9]*' || echo "0")
            printf 'RESULTS: %s\n' "$TOTAL"
            printf '%s\n' "$RESULT"
            sep

            [[ "$TOTAL" -gt 0 ]] && FOUND=1
        fi
    fi
fi

# ── Exit ─────────────────────────────────────────────────────────────────────

if [[ "$FOUND" -eq 0 ]]; then
    echo "OUTCOME: no_specre_card_found"
    echo "RECOMMENDATION: fall back to grep/glob and read source files directly"
    exit 1
else
    echo "OUTCOME: specre_card_found"
    exit 0
fi
