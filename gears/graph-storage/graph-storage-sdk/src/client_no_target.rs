//! The client `GraphStorageClientV1::in_tenant` returns from an
//! implementation that does not support a target tenant.
//!
//! It refuses every call. Answering for the caller's own tenant instead would
//! write and read the wrong tenant's rows without a trace, which is the one
//! outcome a target tenant exists to rule out.

use async_trait::async_trait;
use toolkit_canonical_errors::{CanonicalError, resource_error};
use toolkit_security::SecurityContext;

use crate::client::GraphStorageClientV1;
use crate::models::{
    DeleteOutcome, EdgeKey, GraphRevision, GtsTypeId, IngestOutcome, IngestRequest,
    NeighborhoodRequest, NodeCount, NodeKey, NodeRow, NodeView, Page, RegisteredType,
    SearchRequest, SearchResponse, TraversalResponse, TraverseRequest, TypeQuery, TypeRecord,
    TypeRegistration, TypeRegistrationOptions,
};

#[resource_error(gts_id!("cf.core.graph.node.v1~"))]
struct TargetTenantError;

/// A client that refuses every call: the provided `in_tenant` of an
/// implementation that cannot honour a target tenant.
pub struct NoTargetTenant;

/// The `unimplemented` error a provided client method answers with when the
/// implementation does not override it.
pub fn unimplemented(message: &'static str) -> CanonicalError {
    TargetTenantError::unimplemented(message).create()
}

fn refused() -> CanonicalError {
    TargetTenantError::unimplemented(
        "this GraphStorageClientV1 implementation does not support a target tenant",
    )
    .create()
}

#[async_trait]
impl GraphStorageClientV1 for NoTargetTenant {
    async fn register_types_with(
        &self,
        _: &SecurityContext,
        _: Vec<TypeRegistration>,
        _: TypeRegistrationOptions,
    ) -> Result<Vec<RegisteredType>, CanonicalError> {
        Err(refused())
    }

    async fn get_type(
        &self,
        _: &SecurityContext,
        _: &GtsTypeId,
    ) -> Result<TypeRecord, CanonicalError> {
        Err(refused())
    }

    async fn list_types(
        &self,
        _: &SecurityContext,
        _: TypeQuery,
    ) -> Result<Page<TypeRecord>, CanonicalError> {
        Err(refused())
    }

    async fn ingest(
        &self,
        _: &SecurityContext,
        _: IngestRequest,
    ) -> Result<IngestOutcome, CanonicalError> {
        Err(refused())
    }

    async fn delete_node(
        &self,
        _: &SecurityContext,
        _: &NodeKey,
    ) -> Result<DeleteOutcome, CanonicalError> {
        Err(refused())
    }

    async fn delete_edge(
        &self,
        _: &SecurityContext,
        _: &EdgeKey,
    ) -> Result<DeleteOutcome, CanonicalError> {
        Err(refused())
    }

    async fn get_node(
        &self,
        _: &SecurityContext,
        _: &NodeKey,
        _: Option<u32>,
    ) -> Result<NodeView, CanonicalError> {
        Err(refused())
    }

    async fn project_nodes(
        &self,
        _: &SecurityContext,
        _: &[String],
        _: toolkit_odata::ODataQuery,
    ) -> Result<toolkit_odata::Page<NodeRow>, CanonicalError> {
        Err(refused())
    }

    async fn count_nodes(
        &self,
        _: &SecurityContext,
        _: &[String],
        _: toolkit_odata::ODataQuery,
    ) -> Result<NodeCount, CanonicalError> {
        Err(refused())
    }

    async fn search(
        &self,
        _: &SecurityContext,
        _: SearchRequest,
    ) -> Result<SearchResponse, CanonicalError> {
        Err(refused())
    }

    async fn traverse(
        &self,
        _: &SecurityContext,
        _: TraverseRequest,
    ) -> Result<TraversalResponse, CanonicalError> {
        Err(refused())
    }

    async fn neighborhood(
        &self,
        _: &SecurityContext,
        _: NeighborhoodRequest,
    ) -> Result<TraversalResponse, CanonicalError> {
        Err(refused())
    }

    async fn revision(&self, _: &SecurityContext) -> Result<GraphRevision, CanonicalError> {
        Err(refused())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A target the implementation cannot honour is refused, never served
    /// for the caller's own tenant.
    #[tokio::test]
    async fn a_target_tenant_an_implementation_cannot_honour_is_refused() {
        let ctx = SecurityContext::anonymous();
        let scoped = NoTargetTenant.in_tenant(uuid::Uuid::now_v7());
        let error = scoped.revision(&ctx).await.expect_err("refused");
        assert!(
            format!("{error:?}")
                .to_lowercase()
                .contains("unimplemented"),
            "{error:?}"
        );
    }
}
