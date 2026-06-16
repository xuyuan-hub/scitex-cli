use crate::api_response::extract_object;
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::services::path_segment_encode;
use crate::types::TaskType;

impl BiolabClient {
    pub async fn create_admin_task_type(
        &self,
        data: &serde_json::Value,
    ) -> Result<TaskType, BiolabError> {
        let resp: serde_json::Value = self.http.post(admin_task_types_path(), data).await?;
        extract_object(resp)
    }

    pub async fn delete_admin_task_type(&self, task_type_id: &str) -> Result<(), BiolabError> {
        self.http
            .delete_empty(&admin_task_type_path(task_type_id))
            .await
    }
}

fn admin_task_types_path() -> &'static str {
    "/task-types"
}

fn admin_task_type_path(task_type_id: &str) -> String {
    format!("/task-types/{}", path_segment_encode(task_type_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_admin_task_type_paths() {
        assert_eq!(admin_task_types_path(), "/task-types");
        assert_eq!(
            admin_task_type_path("type with space"),
            "/task-types/type%20with%20space"
        );
    }
}
