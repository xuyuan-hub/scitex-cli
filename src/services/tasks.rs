use crate::api_response::{
    envelope_data, extract_array, extract_object, extract_paginated, PaginatedList,
};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::{path_segment_encode, url_encode};
use crate::types::{
    StaffAssignmentDetail, StaffAssignmentItem, Task, TaskDocument, TaskResult, TaskSummary,
    TaskType, WorkflowDetail,
};

impl ScientexClient {
    pub async fn search_task_types(
        &self,
        skip: u32,
        limit: u32,
        search: Option<&str>,
        filters: Option<&str>,
    ) -> Result<PaginatedList<TaskType>, ScientexError> {
        let path = task_types_path(skip, limit, search, filters);
        let resp: serde_json::Value = self.http.get(&path).await?;
        extract_paginated(resp)
    }

    pub async fn list_lab_task_types(
        &self,
        lab_id: Option<&str>,
    ) -> Result<PaginatedList<TaskType>, ScientexError> {
        let path = lab_task_types_path();
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_paginated(resp)
    }

    pub async fn list_lab_tasks(
        &self,
        skip: u32,
        limit: u32,
        lab_id: Option<&str>,
    ) -> Result<PaginatedList<TaskSummary>, ScientexError> {
        let path = lab_tasks_path(skip, limit);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_paginated(resp)
    }

    pub async fn get_lab_task(
        &self,
        task_id: &str,
        lab_id: Option<&str>,
    ) -> Result<Task, ScientexError> {
        let path = lab_task_path(task_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_object(resp)
    }

    pub async fn list_lab_task_documents(
        &self,
        task_id: &str,
        lab_id: Option<&str>,
    ) -> Result<PaginatedList<TaskDocument>, ScientexError> {
        let path = lab_task_documents_path(task_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_paginated(resp)
    }

    pub async fn download_lab_task_document(
        &self,
        document_id: &str,
        lab_id: Option<&str>,
    ) -> Result<Vec<u8>, ScientexError> {
        let path = lab_task_document_download_path(document_id);
        if let Some(lab_id) = lab_id {
            self.http
                .download_bytes_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await
        } else {
            self.http.download_bytes(&path).await
        }
    }

    pub async fn download_task_output_file(
        &self,
        download_url: &str,
    ) -> Result<Vec<u8>, ScientexError> {
        self.http.download_absolute_bytes(download_url).await
    }

    pub async fn list_lab_task_results(
        &self,
        task_id: &str,
        lab_id: Option<&str>,
    ) -> Result<PaginatedList<TaskResult>, ScientexError> {
        let path = lab_task_results_path(task_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_paginated(resp)
    }

    pub async fn create_task(&self, data: &serde_json::Value) -> Result<Task, ScientexError> {
        let resp: serde_json::Value = self.http.post(tasks_path(), data).await?;
        extract_object(resp)
    }

    pub async fn create_lab_task(
        &self,
        data: &serde_json::Value,
        lab_id: Option<&str>,
    ) -> Result<Task, ScientexError> {
        let path = lab_tasks_create_path();
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .post_with_headers(path, data, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.post(path, data).await?
        };
        extract_object(resp)
    }

    pub async fn create_lab_task_multipart(
        &self,
        data: &serde_json::Value,
        file_fields: &[(&str, &str)],
        lab_id: Option<&str>,
    ) -> Result<Task, ScientexError> {
        let payload = serde_json::to_string(data)
            .map_err(|e| ScientexError::ParseError(format!("Cannot encode task payload: {e}")))?;
        let fields = [("payload", payload)];
        let headers = lab_id
            .map(|lab_id| vec![("X-Current-Lab", lab_id)])
            .unwrap_or_default();
        let resp: serde_json::Value = self
            .http
            .post_multipart(
                lab_tasks_create_path(),
                &fields,
                file_fields,
                headers.as_slice(),
            )
            .await?;
        extract_object(resp)
    }

    pub async fn update_task(
        &self,
        task_id: &str,
        data: &serde_json::Value,
    ) -> Result<Task, ScientexError> {
        let resp: serde_json::Value = self.http.patch(&task_path(task_id), data).await?;
        extract_object(resp)
    }

    pub async fn get_task_workflow(&self, task_id: &str) -> Result<WorkflowDetail, ScientexError> {
        let resp: serde_json::Value = self.http.get(&task_workflow_path(task_id)).await?;
        extract_object(resp)
    }

    pub async fn get_task_type(&self, task_type_id: &str) -> Result<TaskType, ScientexError> {
        let resp: serde_json::Value = self.http.get(&task_type_path(task_type_id)).await?;
        extract_object(resp)
    }

    pub async fn upload_task_field(
        &self,
        task_id: &str,
        file_path: &str,
        field_key: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let path = task_upload_field_path(task_id);
        self.http
            .upload_multipart(&path, file_path, &[("field_key", field_key)], &[])
            .await
    }

    pub async fn upload_lab_task_field(
        &self,
        task_id: &str,
        file_path: &str,
        field_key: &str,
        lab_id: Option<&str>,
    ) -> Result<serde_json::Value, ScientexError> {
        let path = lab_task_upload_field_path(task_id);
        let headers = lab_id
            .map(|lab_id| vec![("X-Current-Lab", lab_id)])
            .unwrap_or_default();
        self.http
            .upload_multipart(
                &path,
                file_path,
                &[("field_key", field_key)],
                headers.as_slice(),
            )
            .await
    }

    pub async fn list_my_task_assignments(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<StaffAssignmentItem>, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&staff_task_assignments_path(skip, limit))
            .await?;
        extract_paginated(resp)
    }

    pub async fn get_my_task_assignment(
        &self,
        assignment_id: &str,
    ) -> Result<StaffAssignmentDetail, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&staff_task_assignment_path(assignment_id))
            .await?;
        extract_object(resp)
    }

    pub async fn update_my_task_assignment_status(
        &self,
        assignment_id: &str,
        status: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .patch(
                &staff_task_assignment_status_path(assignment_id),
                &serde_json::json!({ "status": status }),
            )
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn submit_my_task_result(
        &self,
        assignment_id: &str,
        data: &serde_json::Value,
    ) -> Result<TaskResult, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(&staff_task_assignment_results_path(assignment_id), data)
            .await?;
        extract_object(resp)
    }

    pub async fn list_my_task_documents(
        &self,
        task_id: &str,
    ) -> Result<PaginatedList<TaskDocument>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&staff_task_documents_path(task_id)).await?;
        extract_paginated(resp)
    }

    pub async fn download_my_task_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<u8>, ScientexError> {
        self.http
            .download_bytes(&staff_task_document_download_path(document_id))
            .await
    }

    // --- Admin task document operations (POST/GET/DELETE /tasks/{id}/documents) ---

    /// POST /tasks/{task_id}/documents — upload a document to a task (admin).
    pub async fn upload_task_document(
        &self,
        task_id: &str,
        file_path: &str,
        document_type: &str,
        visibility: Option<&str>,
        part_id: Option<&str>,
    ) -> Result<TaskDocument, ScientexError> {
        let mut fields: Vec<(&str, &str)> = vec![("document_type", document_type)];
        if let Some(vis) = visibility {
            fields.push(("visibility", vis));
        }
        let path = task_documents_path(task_id);
        let resp: serde_json::Value = self
            .http
            .upload_multipart(&path, file_path, &fields, &[])
            .await?;
        let mut doc: TaskDocument = extract_object(resp)?;
        // If part_id was specified, the upload endpoint may not accept it as a
        // multipart field; fall back to a PATCH after creation.
        if let Some(pid) = part_id {
            if doc.part_id.as_deref() != Some(pid) {
                let patch_body = serde_json::json!({ "part_id": pid });
                let patched: serde_json::Value = self
                    .http
                    .patch(&task_document_path(task_id, &doc.id), &patch_body)
                    .await?;
                doc = extract_object(patched)?;
            }
        }
        Ok(doc)
    }

    /// GET /tasks/{task_id}/documents — list documents for a task (admin).
    pub async fn list_task_documents(
        &self,
        task_id: &str,
    ) -> Result<PaginatedList<TaskDocument>, ScientexError> {
        let path = task_documents_path(task_id);
        let resp: serde_json::Value = self.http.get(&path).await?;
        extract_paginated(resp)
    }

    /// DELETE /tasks/{task_id}/documents/{document_id} — delete a task document (admin).
    pub async fn delete_task_document(
        &self,
        task_id: &str,
        document_id: &str,
    ) -> Result<(), ScientexError> {
        let path = task_document_path(task_id, document_id);
        self.http.delete_empty(&path).await
    }

    /// GET /task-types/{type_id}/feedback — list document feedback for a task type (task_manager).
    pub async fn list_task_type_feedback(
        &self,
        task_type_id: &str,
    ) -> Result<Vec<TaskResult>, ScientexError> {
        let path = task_type_feedback_path(task_type_id);
        let resp: serde_json::Value = self.http.get(&path).await?;
        extract_array(resp)
    }
}

fn tasks_path() -> &'static str {
    "/tasks"
}

fn task_path(task_id: &str) -> String {
    format!("/tasks/{}", path_segment_encode(task_id))
}

fn task_types_path(skip: u32, limit: u32, search: Option<&str>, filters: Option<&str>) -> String {
    let mut path = format!("/task-types?skip={skip}&limit={limit}");
    if let Some(search) = search.filter(|value| !value.is_empty()) {
        path.push_str("&search=");
        path.push_str(&url_encode(search));
    }
    if let Some(filters) = filters.filter(|value| !value.is_empty()) {
        path.push_str("&filters=");
        path.push_str(&url_encode(filters));
    }
    path
}

fn lab_task_types_path() -> String {
    "/lab/tasks/task-types".to_string()
}

fn lab_tasks_path(skip: u32, limit: u32) -> String {
    format!("/lab/tasks?skip={skip}&limit={limit}")
}

fn lab_tasks_create_path() -> &'static str {
    "/lab/tasks"
}

fn lab_task_path(task_id: &str) -> String {
    format!("/lab/tasks/{}", path_segment_encode(task_id))
}

fn lab_task_documents_path(task_id: &str) -> String {
    format!("{}/documents", lab_task_path(task_id))
}

fn lab_task_upload_field_path(task_id: &str) -> String {
    format!("{}/upload-field", lab_task_path(task_id))
}

fn lab_task_document_download_path(document_id: &str) -> String {
    format!(
        "/lab/tasks/documents/{}/download",
        path_segment_encode(document_id)
    )
}

fn lab_task_results_path(task_id: &str) -> String {
    format!("{}/results", lab_task_path(task_id))
}

fn task_workflow_path(task_id: &str) -> String {
    format!("{}/workflow", task_path(task_id))
}

fn task_type_path(task_type_id: &str) -> String {
    format!("/task-types/{}", path_segment_encode(task_type_id))
}

fn staff_task_assignments_path(skip: u32, limit: u32) -> String {
    format!("/staff/tasks/assignments?skip={skip}&limit={limit}")
}

fn staff_task_assignment_path(assignment_id: &str) -> String {
    format!(
        "/staff/tasks/assignments/{}",
        path_segment_encode(assignment_id)
    )
}

fn staff_task_assignment_status_path(assignment_id: &str) -> String {
    format!("{}/status", staff_task_assignment_path(assignment_id))
}

fn staff_task_assignment_results_path(assignment_id: &str) -> String {
    format!("{}/results", staff_task_assignment_path(assignment_id))
}

fn staff_task_documents_path(task_id: &str) -> String {
    format!("/staff/tasks/{}/documents", path_segment_encode(task_id))
}

fn staff_task_document_download_path(document_id: &str) -> String {
    format!(
        "/staff/tasks/documents/{}/download",
        path_segment_encode(document_id)
    )
}

fn task_upload_field_path(task_id: &str) -> String {
    format!("/tasks/{}/upload-field", path_segment_encode(task_id))
}

fn task_documents_path(task_id: &str) -> String {
    format!("{}/documents", task_path(task_id))
}

fn task_document_path(task_id: &str, document_id: &str) -> String {
    format!(
        "{}/{}",
        task_documents_path(task_id),
        path_segment_encode(document_id)
    )
}

fn task_type_feedback_path(task_type_id: &str) -> String {
    format!("{}/feedback", task_type_path(task_type_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_lab_task_paths() {
        assert_eq!(lab_task_types_path(), "/lab/tasks/task-types");
        assert_eq!(lab_tasks_path(10, 25), "/lab/tasks?skip=10&limit=25");
        assert_eq!(lab_tasks_create_path(), "/lab/tasks");
        assert_eq!(lab_task_path("task 1"), "/lab/tasks/task%201");
        assert_eq!(
            lab_task_documents_path("task 1"),
            "/lab/tasks/task%201/documents"
        );
        assert_eq!(
            lab_task_upload_field_path("task 1"),
            "/lab/tasks/task%201/upload-field"
        );
        assert_eq!(
            lab_task_document_download_path("doc 1"),
            "/lab/tasks/documents/doc%201/download"
        );
        assert_eq!(
            lab_task_results_path("task 1"),
            "/lab/tasks/task%201/results"
        );
    }

    #[test]
    fn builds_general_task_paths() {
        assert_eq!(tasks_path(), "/tasks");
        assert_eq!(task_path("task 1"), "/tasks/task%201");
        assert_eq!(task_workflow_path("task 1"), "/tasks/task%201/workflow");
        assert_eq!(task_type_path("type 1"), "/task-types/type%201");
        assert_eq!(
            task_documents_path("task 1"),
            "/tasks/task%201/documents"
        );
        assert_eq!(
            task_document_path("task 1", "doc 1"),
            "/tasks/task%201/documents/doc%201"
        );
        assert_eq!(
            task_type_feedback_path("type 1"),
            "/task-types/type%201/feedback"
        );
        assert_eq!(
            task_types_path(0, 100, None, None),
            "/task-types?skip=0&limit=100"
        );
        assert_eq!(
            task_types_path(0, 100, Some("sample qc"), None),
            "/task-types?skip=0&limit=100&search=sample+qc"
        );
        assert_eq!(
            task_types_path(
                0,
                20,
                Some("ngs"),
                Some(r#"[{"field":"category","operator":"eq","value":"compute"}]"#)
            ),
            "/task-types?skip=0&limit=20&search=ngs&filters=%5B%7B%22field%22%3A%22category%22%2C%22operator%22%3A%22eq%22%2C%22value%22%3A%22compute%22%7D%5D"
        );
    }

    #[test]
    fn builds_staff_task_paths() {
        assert_eq!(
            staff_task_assignments_path(0, 100),
            "/staff/tasks/assignments?skip=0&limit=100"
        );
        assert_eq!(
            staff_task_assignment_path("assignment 1"),
            "/staff/tasks/assignments/assignment%201"
        );
        assert_eq!(
            staff_task_assignment_status_path("assignment 1"),
            "/staff/tasks/assignments/assignment%201/status"
        );
        assert_eq!(
            staff_task_assignment_results_path("assignment 1"),
            "/staff/tasks/assignments/assignment%201/results"
        );
        assert_eq!(
            staff_task_documents_path("task 1"),
            "/staff/tasks/task%201/documents"
        );
        assert_eq!(
            staff_task_document_download_path("doc 1"),
            "/staff/tasks/documents/doc%201/download"
        );
    }
}
