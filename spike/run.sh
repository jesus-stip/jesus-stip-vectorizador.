#!/usr/bin/env bash
# usage: run.sh <vtracer-binary> <samples/png dir> <out dir>
set -euo pipefail
VT="$1"; IN="$2"; OUT="$3"; mkdir -p "$OUT"
for f in "$IN"/*.png; do
  n=$(basename "$f" .png)
  for p in default detailed; do
    ms=$(node trace-imagetracer.mjs "$f" "$OUT/$n.itj-$p.svg" "$p" | sed 's/.*:\([0-9]*\).*/\1/')
    echo "$n itj-$p $ms"
  done
  declare -A VTP=(
    [poster]="--preset poster"
    [medium]="--colormode color --hierarchical stacked --mode spline -f 4 -p 6 -g 16 -c 60 -l 4 -s 45"
    [high]="--colormode color --hierarchical stacked --mode spline -f 2 -p 7 -g 8 -c 60 -l 3.5 -s 45"
  )
  for p in poster medium high; do
    s=$(date +%s%N); $VT -i "$f" -o "$OUT/$n.vt-$p.svg" ${VTP[$p]} >/dev/null; e=$(date +%s%N)
    echo "$n vt-$p $(( (e-s)/1000000 ))"
  done
done
