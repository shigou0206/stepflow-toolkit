use async_graphql::{EmptySubscription, Object, Result, Schema};

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn tool(&self, _id: String) -> Result<String, async_graphql::Error> {
        Ok("Tool query not implemented yet".to_string())
    }

    async fn tools(&self) -> Result<Vec<String>, async_graphql::Error> {
        Ok(vec!["Tools query not implemented yet".to_string()])
    }

    async fn execution(&self, _id: String) -> Result<String, async_graphql::Error> {
        Ok("Execution query not implemented yet".to_string())
    }

    async fn executions(&self) -> Result<Vec<String>, async_graphql::Error> {
        Ok(vec!["Executions query not implemented yet".to_string()])
    }
}

#[derive(Default)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn execute_tool(&self, _tool_id: String, _input: String) -> Result<String, async_graphql::Error> {
        Ok("Execute tool mutation not implemented yet".to_string())
    }
}

pub type StepflowSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Create GraphQL schema
pub fn create_schema() -> StepflowSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .finish()
}

/// GraphQL context
pub struct GraphQLContext {
    pub app_state: crate::server::AppState,
} 