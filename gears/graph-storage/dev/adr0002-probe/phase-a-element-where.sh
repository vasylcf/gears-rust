#!/usr/bin/env bash
# Phase A of the ADR-0002 probe: does embedding the scope in every element
# pattern cost anything against carrying it in the pattern's top-level WHERE?
#
# ADR-0002 (secure-orm) mandates the first form -- "embed scope into every
# element pattern participating in MATCH, vertices *and* edges alike" -- so if
# it plans or runs worse than the form this gear ships, that is a cost the ADR
# is imposing and should know about.
#
#   ./gears/graph-storage/dev/adr0002-probe/phase-a-element-where.sh
set -euo pipefail

DB=${DB:-gs-stand-db}
RUNS=${RUNS:-9}
TENANT=${TENANT:-00000000-df51-5b42-9538-d2b56b7ee953}

run() {           # run <label> <sql>
  local label=$1 sql=$2 times=()
  for _ in $(seq 1 "$RUNS"); do
    times+=("$(docker exec -i "$DB" psql -U graph -d graph -tAc \
      "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, BUFFERS OFF) $sql" \
      | grep 'Execution Time' | grep -oE '[0-9.]+')")
  done
  printf '  %-28s ' "$label"
  printf '%s\n' "${times[@]}" | sort -n | awk '{a[NR]=$1} END {
    printf "median=%.3fms  min=%.3fms  max=%.3fms\n", a[int((NR+1)/2)], a[1], a[NR] }'
}

rows() {          # rows <sql> -- what the form actually returns, for equality
  docker exec -i "$DB" psql -U graph -d graph -tAc "$sql_prefix $1" 2>&1
}

echo "=== Phase A: scope in every element vs scope in the top-level WHERE ==="
echo "--- one hop, seed 5000"

TOP="SELECT n FROM GRAPH_TABLE(kb_pgq
  MATCH (a IS node)-[e IS edge]->(b IS node)
  WHERE a.tenant_id = '$TENANT' AND b.tenant_id = '$TENANT' AND a.id = 5000
  COLUMNS (b.id AS n)) g"

ELEM="SELECT n FROM GRAPH_TABLE(kb_pgq
  MATCH (a IS node WHERE a.tenant_id = '$TENANT' AND a.id = 5000)
       -[e IS edge WHERE e.tenant_id = '$TENANT']->
        (b IS node WHERE b.tenant_id = '$TENANT')
  COLUMNS (b.id AS n)) g"

# ADR-0002 scopes the edge element too; this gear does not, because composite
# element keys already tie an edge to one tenant. Measured separately so the
# cost of the mandate is attributable.
ELEM_NO_EDGE="SELECT n FROM GRAPH_TABLE(kb_pgq
  MATCH (a IS node WHERE a.tenant_id = '$TENANT' AND a.id = 5000)
       -[e IS edge]->
        (b IS node WHERE b.tenant_id = '$TENANT')
  COLUMNS (b.id AS n)) g"

run "top-level WHERE (ours)" "$TOP"
run "per-element WHERE (ADR-0002)" "$ELEM"
run "per-element, edge unscoped" "$ELEM_NO_EDGE"

echo "--- results identical?"
for label in TOP ELEM ELEM_NO_EDGE; do
  eval "q=\$$label"
  printf '  %-28s %s\n' "$label" \
    "$(docker exec -i "$DB" psql -U graph -d graph -tAc "SELECT string_agg(n::text, ',' ORDER BY n) FROM ($q) t")"
done

echo "--- plans"
for label in TOP ELEM; do
  eval "q=\$$label"
  echo "  [$label]"
  docker exec -i "$DB" psql -U graph -d graph -tAc \
    "EXPLAIN (COSTS OFF) $q" | sed 's/^/    /'
done
