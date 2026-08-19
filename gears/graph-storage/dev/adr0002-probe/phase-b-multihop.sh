#!/usr/bin/env bash
# Phase B of the ADR-0002 probe: what a fixed-depth path pattern costs against
# chaining one-hop patterns.
#
# ADR-0002's PathBuilder reads as a path -- .vertex().edge().to() -- so the
# natural way to ask for a two- or three-hop neighbourhood is one MATCH with
# several elements. The PG19 spike found that shape enumerates *paths* rather
# than reachable nodes. This measures the gap on the gear's own schema, with
# ADR-0002's per-element scoping in both forms so the comparison is only about
# pattern shape.
set -euo pipefail

DB=${DB:-gs-stand-db}
RUNS=${RUNS:-5}
TENANT=${TENANT:-00000000-df51-5b42-9538-d2b56b7ee953}
T="'$TENANT'"

time_it() {       # time_it <label> <sql>
  local label=$1 sql=$2 times=()
  for _ in $(seq 1 "$RUNS"); do
    times+=("$(docker exec -i "$DB" psql -U graph -d graph -tAc \
      "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, BUFFERS OFF) $sql" \
      | grep 'Execution Time' | grep -oE '[0-9.]+')")
  done
  printf '  %-34s ' "$label"
  printf '%s\n' "${times[@]}" | sort -n | awk '{a[NR]=$1} END {
    printf "median=%9.3fms\n", a[int((NR+1)/2)] }'
}

count() {         # count <sql>
  docker exec -i "$DB" psql -U graph -d graph -tAc "$1"
}

# One element, scoped the ADR-0002 way.
v() { echo "($1 IS node WHERE $1.tenant_id = $T)"; }
e() { echo "[$1 IS edge WHERE $1.tenant_id = $T]"; }

for SEED in 1875 5000; do
  echo "=== seed $SEED (out-degree $(count "SELECT count(*) FROM graph_edge WHERE tenant_id=$T AND src_node_id=$SEED")) ==="

  PATH2="SELECT c FROM GRAPH_TABLE(kb_pgq
    MATCH (a IS node WHERE a.tenant_id = $T AND a.id = $SEED)-$(e e1)->$(v b)-$(e e2)->(c IS node WHERE c.tenant_id = $T)
    COLUMNS (c.id AS c)) g"

  PATH3="SELECT d FROM GRAPH_TABLE(kb_pgq
    MATCH (a IS node WHERE a.tenant_id = $T AND a.id = $SEED)-$(e e1)->$(v b)-$(e e2)->$(v c)-$(e e3)->(d IS node WHERE d.tenant_id = $T)
    COLUMNS (d.id AS d)) g"

  # Chained one-hop, deduplicated between hops. The frontier reaches the next
  # pattern through a correlated sibling, since a pattern cannot hold a subquery.
  CHAIN2="WITH h1 AS (
      SELECT DISTINCT n FROM GRAPH_TABLE(kb_pgq
        MATCH (a IS node WHERE a.tenant_id = $T AND a.id = $SEED)-$(e e1)->(b IS node WHERE b.tenant_id = $T)
        COLUMNS (b.id AS n)) g)
    SELECT DISTINCT g2.n FROM h1, GRAPH_TABLE(kb_pgq
        MATCH (a IS node WHERE a.tenant_id = $T AND a.id = h1.n)-$(e e2)->(b IS node WHERE b.tenant_id = $T)
        COLUMNS (b.id AS n)) g2"

  CHAIN3="WITH h1 AS (
      SELECT DISTINCT n FROM GRAPH_TABLE(kb_pgq
        MATCH (a IS node WHERE a.tenant_id = $T AND a.id = $SEED)-$(e e1)->(b IS node WHERE b.tenant_id = $T)
        COLUMNS (b.id AS n)) g),
    h2 AS (
      SELECT DISTINCT g2.n FROM h1, GRAPH_TABLE(kb_pgq
        MATCH (a IS node WHERE a.tenant_id = $T AND a.id = h1.n)-$(e e2)->(b IS node WHERE b.tenant_id = $T)
        COLUMNS (b.id AS n)) g2)
    SELECT DISTINCT g3.n FROM h2, GRAPH_TABLE(kb_pgq
        MATCH (a IS node WHERE a.tenant_id = $T AND a.id = h2.n)-$(e e3)->(b IS node WHERE b.tenant_id = $T)
        COLUMNS (b.id AS n)) g3"

  echo "  rows returned: path2=$(count "SELECT count(*) FROM ($PATH2) t")  chain2=$(count "SELECT count(*) FROM ($CHAIN2) t")"
  echo "                 path3=$(count "SELECT count(*) FROM ($PATH3) t")  chain3=$(count "SELECT count(*) FROM ($CHAIN3) t")"
  time_it "depth 2, one path pattern" "$PATH2"
  time_it "depth 2, chained one-hop" "$CHAIN2"
  time_it "depth 3, one path pattern" "$PATH3"
  time_it "depth 3, chained one-hop" "$CHAIN3"
done
