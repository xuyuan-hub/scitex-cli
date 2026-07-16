use crate::api_response::{extract_array, extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::path_segment_encode;
use crate::types::{StaffUserInfo, TaskType, TaskTypeDocument};

impl ScientexClient {
    pub async fn list_admin_task_types(
        &self,
        skip: u32,
        limit: u32,
        search: Option<&str>,
        filters: Option<&str>,
    ) -> Result<PaginatedList<TaskType>, ScientexError> {
        let path = admin_task_types_query_path(skip, limit, search, filters);
        let resp: serde_json::Value = self.http.get(&path).await?;
        extract_paginated(resp)
    }

    pub async fn get_admin_task_type(&self, task_type_id: &str) -> Result<TaskType, ScientexError> {
        let resp: serde_json::Value = self.http.get(&admin_task_type_path(task_type_id)).await?;
        extract_object(resp)
    }

    pub async fn update_admin_task_type(
        &self,
        task_type_id: &str,
        data: &serde_json::Value,
    ) -> Result<TaskType, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .patch(&admin_task_type_path(task_type_id), data)
            .await?;
        extract_object(resp)
    }

    pub async fn create_admin_task_type(
        &self,
        data: &serde_json::Value,
        lab_id: Option<&str>,
    ) -> Result<TaskType, ScientexError> {
        let path = admin_task_types_path();
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .post_with_headers(&path, data, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.post(&path, data).await?
        };
        extract_object(resp)
    }

    pub async fn delete_admin_task_type(
        &self,
        task_type_id: &str,
        lab_id: Option<&str>,
    ) -> Result<(), ScientexError> {
        let path = admin_task_type_path(task_type_id);
        if let Some(lab_id) = lab_id {
            self.http
                .delete_empty_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await
        } else {
            self.http.delete_empty(&path).await
        }
    }

    pub async fn list_admin_task_type_staff(
        &self,
        task_type_id: &str,
        lab_id: Option<&str>,
    ) -> Result<Vec<StaffUserInfo>, ScientexError> {
        let path = admin_task_type_staff_path(task_type_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_array(resp)
    }

    pub async fn assign_admin_task_type_staff(
        &self,
        task_type_id: &str,
        user_id: &str,
        lab_id: Option<&str>,
    ) -> Result<(), ScientexError> {
        let path = admin_task_type_staff_path(task_type_id);
        let body = staff_assign_body(user_id);
        if let Some(lab_id) = lab_id {
            self.http
                .post_empty_with_headers(&path, &body, &[("X-Current-Lab", lab_id)])
                .await
        } else {
            self.http.post_empty(&path, &body).await
        }
    }

    pub async fn remove_admin_task_type_staff(
        &self,
        task_type_id: &str,
        user_id: &str,
        lab_id: Option<&str>,
    ) -> Result<(), ScientexError> {
        let path = admin_task_type_staff_user_path(task_type_id, user_id);
        if let Some(lab_id) = lab_id {
            self.http
                .delete_empty_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await
        } else {
            self.http.delete_empty(&path).await
        }
    }

    pub async fn list_admin_task_type_documents(
        &self,
        task_type_id: &str,
        lab_id: Option<&str>,
    ) -> Result<Vec<TaskTypeDocument>, ScientexError> {
        let path = admin_task_type_documents_path(task_type_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_array(resp)
    }

    pub async fn upload_admin_task_type_document(
        &self,
        task_type_id: &str,
        file_path: &str,
        document_type: &str,
        lab_id: Option<&str>,
    ) -> Result<TaskTypeDocument, ScientexError> {
        let fields = [("document_type", document_type)];
        let headers = lab_id
            .map(|id| vec![("X-Current-Lab", id)])
            .unwrap_or_default();
        let resp: serde_json::Value = self
            .http
            .upload_multipart(
                &admin_task_type_documents_path(task_type_id),
                file_path,
                &fields,
                &headers,
            )
            .await?;
        extract_object(resp)
    }

    pub async fn delete_admin_task_type_document(
        &self,
        task_type_id: &str,
        document_id: &str,
        lab_id: Option<&str>,
    ) -> Result<(), ScientexError> {
        let path = admin_task_type_document_path(task_type_id, document_id);
        if let Some(lab_id) = lab_id {
            self.http
                .delete_empty_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await
        } else {
            self.http.delete_empty(&path).await
        }
    }
}

fn admin_task_types_path() -> &'static str {
    "/task-types"
}

fn admin_task_types_query_path(
    skip: u32,
    limit: u32,
    search: Option<&str>,
    filters: Option<&str>,
) -> String {
    let mut path = format!("{}?skip={skip}&limit={limit}", admin_task_types_path());
    if let Some(search) = search.filter(|value| !value.is_empty()) {
        path.push_str("&search=");
        path.push_str(&crate::services::url_encode(search));
    }
    if let Some(filters) = filters.filter(|value| !value.is_empty()) {
        path.push_str("&filters=");
        path.push_str(&crate::services::url_encode(filters));
    }
    path
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

fn admin_task_type_documents_path(task_type_id: &str) -> String {
    format!("{}/documents", admin_task_type_path(task_type_id))
}

fn admin_task_type_document_path(task_type_id: &str, document_id: &str) -> String {
    format!(
        "{}/{}",
        admin_task_type_documents_path(task_type_id),
        path_segment_encode(document_id)
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
            admin_task_types_query_path(0, 20, Some("sample qc"), None),
            "/task-types?skip=0&limit=20&search=sample+qc"
        );
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
    fn builds_admin_task_type_document_paths() {
        assert_eq!(
            admin_task_type_documents_path("type with space"),
            "/task-types/type%20with%20space/documents"
        );
        assert_eq!(
            admin_task_type_document_path("type with space", "doc/1"),
            "/task-types/type%20with%20space/documents/doc%2F1"
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
