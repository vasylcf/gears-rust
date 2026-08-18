#!/usr/bin/env bash
# Compare the three hop backends end to end, over HTTP, on the development
# stand. Every number in dev/FINDINGS.md that mentions p50/p95 at a depth came
# from this script, so re-taking them is one command rather than a reconstruction.
#
#   ./gears/graph-storage/dev/bench-hops.sh
#
# The seed list is derived from a fixed sequence, so the three backends see
# identical work and successive runs are comparable. Results are compared for
# equality as well as timed: a backend that is fast because it answers
# differently is not faster.
set -euo pipefail

CONFIG=${CONFIG:-gears/graph-storage/dev/stand.yaml}
BIN=${BIN:-./target/debug/graph-storage-server}
BASE=${BASE:-http://127.0.0.1:8099/graph-storage/v1}
DEPTHS=${DEPTHS:-"1 2 3"}
SAMPLES=${SAMPLES:-40}
WORK=$(mktemp -d)
# The script rewrites `traversal_hop` in the config as it goes. Put back what
# was there, so a run does not silently leave the stand on whichever backend
# happened to be measured last.
ORIGINAL_HOP=$(grep -oP '(?<=traversal_hop: ).*' "$CONFIG")
cleanup() {
  sed -i "s|      traversal_hop: .*|      traversal_hop: $ORIGINAL_HOP|" "$CONFIG"
  pkill -x graph-storage-s 2>/dev/null || true
  rm -rf "$WORK"
}
trap cleanup EXIT

seq 1 "$SAMPLES" | awk '{srand($1*7919); print 1000 + int(rand()*190000)}' > "$WORK/seeds"

restart_with() {
  sed -i "s|      traversal_hop: .*|      traversal_hop: $1|" "$CONFIG"
  pkill -x graph-storage-s 2>/dev/null || true
  sleep 2
  nohup setsid "$BIN" --config "$CONFIG" > "$WORK/$1.log" 2>&1 < /dev/null &
  for _ in $(seq 1 30); do
    if curl -fsS -o /dev/null "$BASE/stats" 2>/dev/null; then return 0; fi
    sleep 1
  done
  echo "backend $1 did not become ready; see $WORK/$1.log" >&2
  return 1
}

echo "=== hop backends, $SAMPLES seeds, depths: $DEPTHS ==="
for backend in two_query cte pgq; do
  restart_with "$backend"

  # A silent fallback would make the timings describe a different backend.
  if grep -q 'two-query hop' "$WORK/$backend.log"; then
    echo "  WARNING: $backend fell back; timings below are not its own" >&2
  fi

  for depth in $DEPTHS; do
    curl -fsS -o /dev/null "$BASE/neighbours?seeds=5000&depth=$depth"   # warm
    while read -r s; do
      curl -fsS "$BASE/neighbours?seeds=$s&depth=$depth"; echo
    done < "$WORK/seeds" >> "$WORK/results.$backend"

    printf "  %-10s depth=%s  " "$backend" "$depth"
    while read -r s; do
      curl -fsS -o /dev/null -w "%{time_total}\n" "$BASE/neighbours?seeds=$s&depth=$depth"
    done < "$WORK/seeds" | sort -n | awk '{a[NR]=$1} END {
      printf "p50=%.1fms p95=%.1fms p99=%.1fms\n",
        a[int(NR*0.5)]*1000, a[int(NR*0.95)]*1000, a[NR]*1000 }'
  done
done

echo "=== agreement across $((SAMPLES * $(echo "$DEPTHS" | wc -w))) requests ==="
for backend in cte pgq; do
  if diff -q "$WORK/results.two_query" "$WORK/results.$backend" > /dev/null; then
    echo "  $backend == two_query: identical"
  else
    echo "  $backend DIFFERS from two_query" >&2
    diff "$WORK/results.two_query" "$WORK/results.$backend" | head -5 >&2
    exit 1
  fi
done
