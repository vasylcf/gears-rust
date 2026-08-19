-- The cross-tenant trap fixture.
--
-- Our tenant owns 1 -> 2 -> 3. A foreign tenant owns 1 -> 3, with the *same*
-- surrogate ids, because ids are allocated per tenant. A traversal that reads
-- edges without a tenant predicate follows the foreign shortcut and reports
-- node 3 as one hop from node 1.
--
-- This is the fixture `backend_parity::every_backend_holds_the_cross_tenant_trap`
-- springs. It lives in a file because it went missing from the stand once and
-- the test kept passing for days with nothing on the other side of the trap --
-- which is why that test now asserts the fixture exists before trusting a pass.
--
--   docker exec -i gs-stand-db psql -U graph -d graph < gears/graph-storage/dev/seed-trap.sql

\set OURS '00000000-df51-5b42-9538-d2b56b7ee953'
\set FOREIGN_TENANT '00000000-0000-0000-0000-00000000dead'

-- Types first: nodes and edges carry a composite foreign key to graph_type.
INSERT INTO graph_type (tenant_id, id, type_uuid, type_id, kind, json_schema)
SELECT :'FOREIGN_TENANT', id, gen_random_uuid(), type_id, kind, json_schema
FROM graph_type WHERE tenant_id = :'OURS'
ON CONFLICT DO NOTHING;

-- Our tenant's chain.
INSERT INTO graph_node (tenant_id, id, node_key, type_id, name)
SELECT :'OURS', v.id, 'n:' || v.id,
       (SELECT min(id) FROM graph_type WHERE tenant_id = :'OURS' AND kind = 'node'),
       'Node ' || v.id
FROM (VALUES (1),(2),(3)) v(id)
ON CONFLICT DO NOTHING;

INSERT INTO graph_edge (tenant_id, id, edge_key, type_id, src_node_id, dst_node_id)
SELECT :'OURS', v.id, 'e:' || v.id,
       (SELECT min(id) FROM graph_type WHERE tenant_id = :'OURS' AND kind = 'edge'),
       v.src, v.dst
FROM (VALUES (1, 1, 2), (2, 2, 3)) v(id, src, dst)
ON CONFLICT DO NOTHING;

-- The foreign tenant's shortcut, reusing ids 1 and 3.
INSERT INTO graph_node (tenant_id, id, node_key, type_id, name)
SELECT :'FOREIGN_TENANT', v.id, 'f:' || v.id,
       (SELECT min(id) FROM graph_type WHERE tenant_id = :'FOREIGN_TENANT' AND kind = 'node'),
       'Foreign ' || v.id
FROM (VALUES (1),(2),(3)) v(id)
ON CONFLICT DO NOTHING;

INSERT INTO graph_edge (tenant_id, id, edge_key, type_id, src_node_id, dst_node_id)
SELECT :'FOREIGN_TENANT', 900001, 'f:e1',
       (SELECT min(id) FROM graph_type WHERE tenant_id = :'FOREIGN_TENANT' AND kind = 'edge'),
       1, 3
ON CONFLICT DO NOTHING;

SELECT tenant_id, string_agg(src_node_id || ' -> ' || dst_node_id, ', ' ORDER BY src_node_id) AS edges
FROM graph_edge WHERE src_node_id <= 3 AND dst_node_id <= 3
GROUP BY tenant_id;
