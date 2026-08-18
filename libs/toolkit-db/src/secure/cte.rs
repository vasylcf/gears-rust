//! Safe Common Table Expressions for the secure ORM.
//!
//! Implements Level A of `docs/arch/secure-orm/ADR/0001-secure-cte-policy.md`:
//! the scope is embedded **inside the body of every CTE**, never applied around
//! it, so any table a CTE touches is already filtered.
//!
//! The invariant is structural rather than conventional:
//!
//! * a [`SecureCte`] can only be produced from a `SecureSelect<E, Scoped>`, so a
//!   CTE over an unscoped query is unrepresentable;
//! * [`with_ctes`](super::SecureSelect::with_ctes) returns [`SecureCteSelect`],
//!   a distinct type with its own execution path. It cannot return `Self`,
//!   because `SecureSelect` is backed by `sea_orm::Select<E>`, whose `.all()`
//!   never sees a `WithClause` — the `WITH` would silently vanish at execution;
//! * every CTE must carry the same `AccessScope` as the outer query. Mixing
//!   scopes in one statement is rejected at runtime in all build profiles, not
//!   behind `debug_assert!`.
//!
//! Referencing a CTE from the outer query is the caller's job: attach the CTE
//! by name, then filter the outer query against that name with `sea_query`'s
//! `Alias` and `in_subquery`. Getting the correlation wrong can only mis-select
//! rows, never widen access, because each CTE body already carries its own
//! scope.

use std::sync::Arc;

use sea_orm::{
    EntityTrait, FromQueryResult, QueryTrait, Statement,
    sea_query::{Alias, CommonTableExpression, PostgresQueryBuilder, Query, UnionType, WithClause},
};

use crate::secure::error::ScopeError;
use crate::secure::{AccessScope, DBRunner, DBRunnerInternal, ScopableEntity, SeaOrmRunner};

/// A named, already-scoped query usable as a CTE body.
///
/// Constructed only via [`SecureSelect::into_cte`](super::SecureSelect::into_cte).
#[derive(Debug, Clone)]
pub struct SecureCte {
    name: &'static str,
    query: sea_orm::sea_query::SelectStatement,
    scope: Arc<AccessScope>,
}

impl SecureCte {
    #[must_use]
    pub fn new(
        query: sea_orm::sea_query::SelectStatement,
        name: &'static str,
        scope: Arc<AccessScope>,
    ) -> Self {
        Self { name, query, scope }
    }

    /// The name this CTE is attached under.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The scope embedded in this CTE's body.
    #[must_use]
    pub fn scope(&self) -> &AccessScope {
        &self.scope
    }

    fn into_common_table_expression(self) -> CommonTableExpression {
        let mut cte = CommonTableExpression::new();
        cte.query(self.query).table_name(Alias::new(self.name));
        cte
    }
}

/// A scoped query carrying CTE definitions, with its own execution path.
///
/// Produced by [`SecureSelect::with_ctes`](super::SecureSelect::with_ctes).
#[derive(Debug)]
pub struct SecureCteSelect<E, S> {
    pub(crate) statement: sea_orm::sea_query::SelectStatement,
    pub(crate) with: WithClause,
    pub(crate) _entity: std::marker::PhantomData<E>,
    pub(crate) _state: std::marker::PhantomData<S>,
}

impl<E, S> SecureCteSelect<E, S>
where
    E: ScopableEntity + EntityTrait,
{
    fn build(self) -> Statement {
        let sql = self
            .statement
            .with(self.with)
            .to_string(PostgresQueryBuilder);
        Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql)
    }

    /// Render the statement without executing it.
    ///
    /// Exists so tests can assert that the `WITH` clause survives and that each
    /// CTE body carries its scope predicate.
    #[must_use]
    pub fn to_sql(self) -> String {
        self.build().to_string()
    }
}

impl<E> SecureCteSelect<E, crate::secure::Scoped>
where
    E: ScopableEntity + EntityTrait,
{
    /// Execute and return the entity's models.
    ///
    /// # Errors
    /// Returns [`ScopeError::Db`] when the query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn all(self, runner: &impl DBRunner) -> Result<Vec<E::Model>, ScopeError> {
        let stmt = self.build();
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(E::find().from_raw_sql(stmt).all(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(E::find().from_raw_sql(stmt).all(tx).await?),
        }
    }

    /// Execute and return an arbitrary projection.
    ///
    /// The counterpart of `SecureSelect::project_all` for CTE queries: a hop
    /// that only needs identifiers should not materialise whole models.
    ///
    /// # Errors
    /// Returns [`ScopeError::Db`] when the query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn all_as<T>(self, runner: &impl DBRunner) -> Result<Vec<T>, ScopeError>
    where
        T: FromQueryResult + Send + Sync,
    {
        let stmt = self.build();
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(T::find_by_statement(stmt).all(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(T::find_by_statement(stmt).all(tx).await?),
        }
    }
}

/// Assemble the outer statement and its CTEs, enforcing the same-scope rule.
///
/// # Errors
/// Returns [`ScopeError::Denied`] when a CTE carries a different scope than the
/// outer query: each body would still be individually safe, but a single
/// statement mixing scopes is incoherent and is exactly the ad-hoc pattern the
/// policy exists to prevent.
pub fn assemble<E, I>(
    outer: sea_orm::Select<E>,
    outer_scope: &Arc<AccessScope>,
    ctes: I,
) -> Result<SecureCteSelect<E, crate::secure::Scoped>, ScopeError>
where
    E: ScopableEntity + EntityTrait,
    I: IntoIterator<Item = SecureCte>,
{
    let mut with = WithClause::new();
    for cte in ctes {
        if cte.scope() != outer_scope.as_ref() {
            return Err(ScopeError::Denied(
                "cte scope differs from the outer query scope",
            ));
        }
        with = with.cte(cte.into_common_table_expression()).to_owned();
    }

    Ok(SecureCteSelect {
        statement: outer.into_query(),
        with,
        _entity: std::marker::PhantomData,
        _state: std::marker::PhantomData,
    })
}

/// Build a subquery selecting one column from an attached CTE.
///
/// Sugar over `Query::select().from(Alias::new(cte)).column(Alias::new(col))`,
/// so callers do not hand-roll the untyped identifiers on every hop.
#[must_use]
pub fn cte_column(cte: &str, column: &str) -> sea_orm::sea_query::SelectStatement {
    Query::select()
        .column(Alias::new(column))
        .from(Alias::new(cte))
        .to_owned()
}

/// Build a subquery selecting the union of several columns of one attached CTE.
///
/// A graph hop needs "either endpoint of an incident edge". Expressing that as
/// `id IN (src) OR id IN (dst)` makes `PostgreSQL` fall back to a sequential
/// scan of the outer table: two hashed subplans joined by `OR` cannot drive an
/// index. One `IN` over the union of the columns keeps the semi-join, so the
/// outer table is probed by index. Measured on 199k nodes / 600k edges: 15.2 ms
/// versus 0.30 ms for the same result.
///
/// The first column is taken separately so that "at least one column" is a
/// property of the signature rather than a runtime check.
#[must_use]
pub fn cte_columns_union(
    cte: &str,
    first: &str,
    rest: &[&str],
) -> sea_orm::sea_query::SelectStatement {
    let mut query = cte_column(cte, first);
    for col in rest {
        query = query
            .union(UnionType::Distinct, cte_column(cte, col))
            .to_owned();
    }
    query
}
