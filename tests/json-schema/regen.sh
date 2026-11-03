#!/bin/sh
# Regenerates the *.json references from the *.tsp inputs with the official
# @typespec/json-schema emitter. Needs node; the tests themselves do not.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
npm init -y >/dev/null
npm install --silent @typespec/compiler@1 @typespec/json-schema@1
for tsp in "$here"/*.tsp; do
  name=$(basename "$tsp" .tsp)
  { echo 'import "@typespec/json-schema";'; cat "$tsp"; } > main.tsp
  npx tsp compile main.tsp --emit @typespec/json-schema \
    --option "@typespec/json-schema.emitter-output-dir={cwd}/out" \
    --option "@typespec/json-schema.file-type=json" \
    --option "@typespec/json-schema.emitAllModels=true" \
    --option "@typespec/json-schema.bundleId=$name.json" >/dev/null
  cp "out/$name.json" "$here/$name.json"
  rm -rf out
done
