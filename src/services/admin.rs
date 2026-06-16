use crate::api_response::{extract_array, extract_object};
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::services::path_segment_encode;
use crate::types::{StaffUserInfo, TaskType};

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

    pub async fn list_admin_task_type_staff(
        &self,
        task_type_id: &str,
    ) -> Result<Vec<StaffUserInfo>, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .get(&admin_task_type_staff_path(task_type_id))
            .await?;
        extract_array(resp)
    }

    pub async fn assign_admin_task_type_staff(
        &self,
        task_type_id: &str,
        user_id: &str,
    ) -> Result<(), BiolabError> {
        self.http
            .post_empty(
                &admin_task_type_staff_path(task_type_id),
                &staff_assign_body(user_id),
            )
            .await
    }

    pub async fn remove_admin_task_type_staff(
        &self,
        task_type_id: &str,
        user_id: &str,
    ) -> Result<(), BiolabError> {
        self.http
            .delete_empty(&admin_task_type_staff_user_path(task_type_id, user_id))
            .await
    }
}

fn admin_task_types_path() -> &'static str {
    "/task-types"
}

fn admin_task_type_path(task_type_id: &str) -> String {
    format!("/task-types/{}", path_segment_encode(task_type_id))
}

fn admin_task_type_staff_path(task_type_id: &str) -> String {
    format!("{}/staff", admin_task_type_path(task_type_id))
}

fn admin_task_type_staff_user_path(task_type_id: &str, user_id: &str) -> String {
    format!(
        "{}/{}",
        admin_task_type_staff_path(task_type_id),
        path_segment_encode(user_id)
    )
}

fn staff_assign_body(user_id: &str) -> serde_json::Value {
    serde_json::json!({ "user_id": user_id })
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
        assert_eq!(
            admin_task_type_staff_path("type with space"),
            "/task-types/type%20with%20space/staff"
        );
        assert_eq!(
            admin_task_type_staff_user_path("type with space", "user/1"),
            "/task-types/type%20with%20space/staff/user%2F1"
        );
    }

    #[test]
    fn builds_staff_assign_body() {
        assert_eq!(
            staff_assign_body("user-1"),
            serde_json::json!({ "user_id": "user-1" })
        );
    }
}
