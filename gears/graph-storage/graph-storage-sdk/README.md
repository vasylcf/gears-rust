# weftgraph-sdk

> **`weftgraph` is a code name, not a product name.** This crate is a preview
> release of a graph-storage gear built for CF/Gears and consumed by
> Constructor Studio, published from the fork
> [vasylcf/gears-rust](https://github.com/vasylcf/gears-rust) so that `cargo
> add` is all a consumer needs. It is deliberately *not* published under the
> `cf-gears-*` namespace: the name a first-party graph-storage gear eventually
> takes is Constructor Fabric's to choose, and this crate must not stand in
> its way. In-tree the crate is `cf-gears-graph-storage-sdk`, and the library
> it builds keeps its `graph_storage_sdk` name, so moving to an official crate
> later is a one-line change in a consumer's manifest.

The public contract of the graph-storage gear, for consumers and plugin authors:

- `GraphStorageClientV1` — the in-process client trait a consuming gear resolves
  from the `ClientHub`: type registration, ingest, node reads, projection,
  search, traversal, neighbourhood, revision.
- `models` — transport-agnostic request and response shapes (`NodeSpec`,
  `EdgeSpec`, `IngestRequest`, `SearchRequest`, the element envelope).
- `plugin_api` — the plugin contracts the gear is assembled from:
  `GraphStoreV1`, `GraphEngineV1` and `EmbeddingProviderV1`, plus the
  `EmbeddingSpaceId` identity every provider must derive the same way.
- `gts` — the GTS identifiers of the base ontology and the gear's resource
  types.
- `contract` (feature `test-support`) — the executable form of the plugin
  contracts; every provider and store implementation proves itself against the
  same assertions.

```rust,ignore
let graph = ctx.client_hub().get::<dyn graph_storage_sdk::GraphStorageClientV1>()?;
```

See the gear's [DESIGN](../docs/DESIGN.md) for the API semantics.
