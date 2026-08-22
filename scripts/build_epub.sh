#!/usr/bin/env bash
# Build book.epub from every markdown file in the repo: root files first,
# then every markdown file in the folders under it.
set -euo pipefail

cd "$(dirname "$0")/.."

OUTPUT="book.epub"
TITLE="pkcore"
AUTHOR="folkengine"

# Folders to skip entirely: build output, dependency, and tool-state dirs.
EXCLUDE_PATTERN='^\./(\.git|\.github|\.claude|\.config|\.idea|\.okf|\.superpowers|target|generated|graphify-out|node_modules)/'

# Root-level markdown files first, alphabetical order.
files=()
while IFS= read -r file; do
  files+=("$file")
done < <(find . -maxdepth 1 -name "*.md" | sort)

# Every other markdown file, alphabetical order, skipping excluded folders.
while IFS= read -r file; do
  files+=("$file")
done < <(find . -mindepth 2 -name "*.md" | grep -Ev "$EXCLUDE_PATTERN" | sort)

if [ "${#files[@]}" -eq 0 ]; then
  echo "No markdown files found." >&2
  exit 1
fi

echo "Building ${OUTPUT} from ${#files[@]} markdown files:"
printf '  %s\n' "${files[@]}"

# Pictures are linked relative to each markdown file's own folder, but
# pandoc looks relative to a single --resource-path. List every folder
# that holds a markdown file so pandoc can find pictures in any of them.
resource_path="."
for file in "${files[@]}"; do
  resource_path="${resource_path}:$(dirname "$file")"
done

pandoc \
  --from=gfm \
  --to=epub3 \
  --resource-path="${resource_path}" \
  --metadata title="${TITLE}" \
  --metadata author="${AUTHOR}" \
  --toc \
  --toc-depth=2 \
  -o "${OUTPUT}" \
  "${files[@]}"

echo "Wrote ${OUTPUT}"
