#!/bin/bash
#
# Simple linting script to check for required files in board directories.
#

set -e

echo "--- Running Board Docs Check ---"
BASE_DIR="$(dirname "$0")/.."
EXIT_CODE=0

for board_dir in "$BASE_DIR"/*/; do
    if [ -f "$board_dir/SKIP_CHECK" ]; then
        echo "Skipping check for $(basename "$board_dir")"
        continue
    fi

    echo "Checking board: $(basename "$board_dir")"

    # List of required files for every board
    required_files=(
        "pinout.md"
        "flash-via-probe.md"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "$board_dir/$file" ]; then
            echo "  [FAIL] Missing required file: $file"
            EXIT_CODE=1
        else
            echo "  [OK] Found: $file"
        fi
    done
done

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "\nBoard doc check failed. Please add the missing files."
    exit 1
else
    echo -e "\nBoard doc check passed."
fi

exit 0
