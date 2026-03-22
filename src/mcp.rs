use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

use criome_cozo::CriomeDb;

pub use samskara_core::mcp::{
    QueryParams, DescribeRelationParams, QueryRulesParams,
};

// ── Samskara-reader specific param types ────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryThoughtsParams {
    /// Filter by kind (user, feedback, project, reference, observation)
    #[serde(default)]
    pub kind: Option<String>,
    /// Filter by scope (repo name or "global")
    #[serde(default)]
    pub scope: Option<String>,
    /// Filter by tag
    #[serde(default)]
    pub tag: Option<String>,
    /// Filter by phase (becoming, manifest, retired). Default: exclude retired.
    #[serde(default)]
    pub phase: Option<String>,
}

// ── Server struct ───────────────────────────────────────────────

#[derive(Clone)]
pub struct SamskaraReader {
    db: Arc<CriomeDb>,
    tool_router: ToolRouter<Self>,
}

impl SamskaraReader {
    pub fn new(db: Arc<CriomeDb>) -> Self {
        Self {
            db,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SamskaraReader {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Samskara Reader — read-only access to samskara's world state. \
                 All queries run in immutable mode; mutations are rejected at the DB level."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tool_router]
impl SamskaraReader {
    #[tool(description = "Execute read-only CozoScript against the world database. Mutations are rejected.")]
    async fn query(&self, Parameters(params): Parameters<QueryParams>) -> String {
        samskara_core::mcp::query_immutable(self.db.clone(), params.script).await
    }

    #[tool(description = "List all stored relations in the database.")]
    async fn list_relations(&self) -> String {
        samskara_core::mcp::list_relations_immutable(self.db.clone()).await
    }

    #[tool(description = "Show the schema (columns and types) of a specific relation.")]
    async fn describe_relation(
        &self,
        Parameters(params): Parameters<DescribeRelationParams>,
    ) -> String {
        samskara_core::mcp::describe_relation_immutable(self.db.clone(), params.name).await
    }

    #[tool(description = "Query thoughts with optional filters. Excludes retired-phase by default.")]
    async fn query_thoughts(
        &self,
        Parameters(params): Parameters<QueryThoughtsParams>,
    ) -> String {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut conditions: Vec<String> = Vec::new();

            if let Some(ref phase) = params.phase {
                conditions.push(format!("phase = \"{}\"", phase.replace('"', "\\\"")));
            } else {
                conditions.push("phase != \"retired\"".to_string());
            }

            if let Some(ref kind) = params.kind {
                conditions.push(format!("kind = \"{}\"", kind.replace('"', "\\\"")));
            }
            if let Some(ref scope) = params.scope {
                conditions.push(format!("scope = \"{}\"", scope.replace('"', "\\\"")));
            }

            let base = if let Some(ref tag) = params.tag {
                format!(
                    "?[id, kind, scope, status, title, body, phase, dignity] := \
                     *thought{{id, kind, scope, status, title, body, phase, dignity}}, \
                     *thought_tag{{thought_id: id, tag: \"{}\"}}, \
                     {}",
                    tag.replace('"', "\\\""),
                    conditions.join(", ")
                )
            } else {
                format!(
                    "?[id, kind, scope, status, title, body, phase, dignity] := \
                     *thought{{id, kind, scope, status, title, body, phase, dignity}}, \
                     {}",
                    conditions.join(", ")
                )
            };

            db.run_script_cozo_immutable(&base)
                .map_err(|e| e.to_string())
        })
        .await;

        match result {
            Ok(Ok(text)) => text,
            Ok(Err(e)) => format!("error: {e}"),
            Err(e) => format!("error: task join failed: {e}"),
        }
    }

    #[tool(description = "Query rules by microtheory, type, or scope. Returns id, compact summary, rationale, microtheory, rule_type, and scope.")]
    async fn query_rules(
        &self,
        Parameters(params): Parameters<QueryRulesParams>,
    ) -> String {
        samskara_core::mcp::query_rules(self.db.clone(), params).await
    }
}
