#!/usr/bin/env bash
#
# source-dir-scope.sh — Calculate how much of the project's source files
# fall within the specre source_dirs configuration.
#
# Usage: .claude/scripts/source-dir-scope.sh [project-root]
#
# Output (stdout, one value per line):
#   ratio=<float>              e.g. ratio=0.92
#   source_dirs_files=<int>    files within source_dirs
#   total_files=<int>          total project files (filtered)
#   uncovered=<dir>:<count>    one line per uncovered directory (0 or more)
#
# Exit codes:
#   0  success
#   1  specre.toml not found or parse error

set -euo pipefail

PROJECT_ROOT="${1:-$(pwd)}"
CONFIG="${PROJECT_ROOT}/specre.toml"

if [[ ! -f "$CONFIG" ]]; then
    echo "Error: specre.toml not found at ${CONFIG}" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Parse specre.toml
# ---------------------------------------------------------------------------

# Extract source_dirs array: source_dirs = ["src", "tests"]
# → "src" "tests"
raw_source_dirs=$(grep '^source_dirs' "$CONFIG" | sed 's/^source_dirs[[:space:]]*=[[:space:]]*//')
# Strip brackets and quotes, split by comma
IFS=',' read -ra source_dir_tokens <<< "$(echo "$raw_source_dirs" | tr -d '[]"' | tr -s ' ')"
SOURCE_DIRS=()
for token in "${source_dir_tokens[@]}"; do
    trimmed=$(echo "$token" | xargs)  # trim whitespace
    [[ -n "$trimmed" ]] && SOURCE_DIRS+=("$trimmed")
done

# Extract specre_dir
SPECRE_DIR=$(grep '^specre_dir' "$CONFIG" | sed 's/^specre_dir[[:space:]]*=[[:space:]]*//' | tr -d '"' | xargs)

# Extract target_extensions (optional): target_extensions = ["rs", "ts"]
EXTENSIONS=()
if grep -q '^target_extensions' "$CONFIG" 2>/dev/null; then
    raw_ext=$(grep '^target_extensions' "$CONFIG" | sed 's/^target_extensions[[:space:]]*=[[:space:]]*//')
    IFS=',' read -ra ext_tokens <<< "$(echo "$raw_ext" | tr -d '[]"' | tr -s ' ')"
    for token in "${ext_tokens[@]}"; do
        trimmed=$(echo "$token" | xargs)
        [[ -n "$trimmed" ]] && EXTENSIONS+=("$trimmed")
    done
fi

# If no target_extensions, infer from project type
if [[ ${#EXTENSIONS[@]} -eq 0 ]]; then
    if [[ -f "${PROJECT_ROOT}/Cargo.toml" ]]; then
        EXTENSIONS=("rs")
    elif [[ -f "${PROJECT_ROOT}/package.json" ]]; then
        EXTENSIONS=("ts" "tsx" "js" "jsx")
    elif [[ -f "${PROJECT_ROOT}/pyproject.toml" ]] || [[ -f "${PROJECT_ROOT}/setup.py" ]]; then
        EXTENSIONS=("py")
    elif [[ -f "${PROJECT_ROOT}/go.mod" ]]; then
        EXTENSIONS=("go")
    else
        echo "Error: Cannot infer target extensions. Set target_extensions in specre.toml." >&2
        exit 1
    fi
fi

# ---------------------------------------------------------------------------
# Directories to always exclude from counting
# ---------------------------------------------------------------------------
EXCLUDE_DIRS=(
    ".git"
    ".claude"
    "target"
    "node_modules"
    "dist"
    "build"
    "vendor"
    "__pycache__"
    ".mypy_cache"
    ".pytest_cache"
)

# Also exclude specre_dir
EXCLUDE_DIRS+=("$SPECRE_DIR")

# ---------------------------------------------------------------------------
# Build find arguments for extensions
# ---------------------------------------------------------------------------
build_ext_args() {
    local first=true
    echo -n "\\( "
    for ext in "${EXTENSIONS[@]}"; do
        if $first; then
            first=false
        else
            echo -n " -o "
        fi
        echo -n "-name '*.${ext}'"
    done
    echo -n " \\)"
}

# ---------------------------------------------------------------------------
# Build find exclude arguments
# ---------------------------------------------------------------------------
build_exclude_args() {
    for dir in "${EXCLUDE_DIRS[@]}"; do
        echo -n "-not -path '${PROJECT_ROOT}/${dir}/*' "
    done
}

# ---------------------------------------------------------------------------
# Count files using find (via eval to handle dynamic args)
# ---------------------------------------------------------------------------

# Total project files: all matching files in subdirectories (not root), minus exclusions
FIND_CMD="find '${PROJECT_ROOT}' -mindepth 2 -type f $(build_ext_args) $(build_exclude_args)"
TOTAL_FILES=$(eval "$FIND_CMD" | wc -l)

# Source dirs files: matching files within each source_dir
SOURCE_FILES=0
for sd in "${SOURCE_DIRS[@]}"; do
    dir_path="${PROJECT_ROOT}/${sd}"
    if [[ -d "$dir_path" ]]; then
        count=$(eval "find '${dir_path}' -type f $(build_ext_args)" | wc -l)
        SOURCE_FILES=$((SOURCE_FILES + count))
    fi
done

# ---------------------------------------------------------------------------
# Calculate ratio
# ---------------------------------------------------------------------------
if [[ "$TOTAL_FILES" -eq 0 ]]; then
    RATIO="1.00"
else
    RATIO=$(awk "BEGIN { printf \"%.2f\", ${SOURCE_FILES} / ${TOTAL_FILES} }")
fi

# ---------------------------------------------------------------------------
# Find uncovered directories
# ---------------------------------------------------------------------------
# List all top-level subdirectories that contain matching files but are not
# within source_dirs and not in the exclusion list.

ALL_SUBDIRS=$(find "${PROJECT_ROOT}" -mindepth 1 -maxdepth 1 -type d | sort)
declare -A UNCOVERED_MAP

for subdir in $ALL_SUBDIRS; do
    dirname=$(basename "$subdir")

    # Skip excluded directories
    skip=false
    for excl in "${EXCLUDE_DIRS[@]}"; do
        # Handle multi-level specre_dir like "docs/specres" — extract top-level
        excl_top=$(echo "$excl" | cut -d'/' -f1)
        if [[ "$dirname" == "$excl_top" ]]; then
            # Only skip if the entire dir is excluded (not just a subpath)
            # For specre_dir like "docs/specres", "docs" itself is not excluded
            if [[ "$excl" == "$dirname" ]] || [[ "$excl" == "$dirname/"* && ! "$excl" == *"/"*"/"* ]]; then
                skip=true
                break
            fi
        fi
    done

    # Skip if it's a simple exclusion (exact match)
    for excl in "${EXCLUDE_DIRS[@]}"; do
        [[ "$dirname" == "$excl" ]] && { skip=true; break; }
    done
    $skip && continue

    # Skip if within source_dirs
    in_source=false
    for sd in "${SOURCE_DIRS[@]}"; do
        sd_top=$(echo "$sd" | cut -d'/' -f1)
        [[ "$dirname" == "$sd_top" ]] && { in_source=true; break; }
    done
    $in_source && continue

    # Count matching files in this directory
    count=$(eval "find '${subdir}' -type f $(build_ext_args)" 2>/dev/null | wc -l)
    if [[ "$count" -gt 0 ]]; then
        UNCOVERED_MAP["$dirname"]=$count
    fi
done

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------
echo "ratio=${RATIO}"
echo "source_dirs_files=${SOURCE_FILES}"
echo "total_files=${TOTAL_FILES}"
for dir in $(echo "${!UNCOVERED_MAP[@]}" | tr ' ' '\n' | sort); do
    echo "uncovered=${dir}:${UNCOVERED_MAP[$dir]}"
done
