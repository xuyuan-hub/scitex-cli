use crate::api_response::{
    envelope_data, extract_array, extract_object, extract_paginated, PaginatedList,
};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::{empty_body, path_segment_encode, url_encode};
use crate::types::{
    LabTaskTypeDetail, LabTaskTypeListItem, StaffAssignmentDetail, StaffAssignmentItem, Task,
    TaskDocument, TaskExactRerun, TaskPart, TaskPartDetail, TaskResult, TaskRunArtifactPreview,
    TaskSummary, TaskType, UploadFieldResponse, WorkflowDetail,
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
        skip: u32,
        limit: u32,
        search: Option<&str>,
        category: Option<&str>,
        lab_id: Option<&str>,
    ) -> Result<PaginatedList<LabTaskTypeListItem>, ScientexError> {
        let path = lab_task_types_path(skip, limit, search, category);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_paginated(resp)
    }

    pub async fn get_lab_task_type(
        &self,
        task_type_id: &str,
        lab_id: Option<&str>,
    ) -> Result<LabTaskTypeDetail, ScientexError> {
        let path = lab_task_type_path(task_type_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_object(resp)
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

    pub async fn get_lab_task_part(
        &self,
        task_id: &str,
        part_id: &str,
        lab_id: Option<&str>,
    ) -> Result<TaskPartDetail, ScientexError> {
        let path = lab_task_part_path(task_id, part_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .get_with_headers(&path, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.get(&path).await?
        };
        extract_object(resp)
    }

    pub async fn update_lab_task_part_release_schedule(
        &self,
        task_id: &str,
        part_id: &str,
        data: &serde_json::Value,
        lab_id: Option<&str>,
    ) -> Result<TaskPart, ScientexError> {
        let path = lab_task_part_release_schedule_path(task_id, part_id);
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .patch_with_headers(&path, data, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.patch(&path, data).await?
        };
        extract_object(resp)
    }

    pub async fn rerun_lab_task_part_run(
        &self,
        task_id: &str,
        part_id: &str,
        run_id: &str,
        lab_id: Option<&str>,
    ) -> Result<TaskExactRerun, ScientexError> {
        let path = lab_task_part_run_rerun_path(task_id, part_id, run_id);
        let body = empty_body();
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .post_with_headers(&path, &body, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.post(&path, &body).await?
        };
        extract_object(resp)
    }

    pub async fn get_lab_task_run_artifact_preview(
        &self,
        task_id: &str,
        part_id: &str,
        run_id: &str,
        artifact_index: u64,
        lab_id: Option<&str>,
    ) -> Result<TaskRunArtifactPreview, ScientexError> {
        let path = lab_task_run_artifact_preview_path(task_id, part_id, run_id, artifact_index);
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

    pub async fn confirm_lab_task(
        &self,
        task_id: &str,
        lab_id: Option<&str>,
    ) -> Result<serde_json::Value, ScientexError> {
        let path = lab_task_confirm_path(task_id);
        let body = empty_body();
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .post_with_headers(&path, &body, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.post(&path, &body).await?
        };
        Ok(envelope_data(resp))
    }

    pub async fn reject_lab_task(
        &self,
        task_id: &str,
        reason: Option<&str>,
        lab_id: Option<&str>,
    ) -> Result<serde_json::Value, ScientexError> {
        let path = lab_task_reject_path(task_id);
        let body = match reason {
            Some(r) => serde_json::json!({ "reason": r }),
            None => empty_body(),
        };
        let resp: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .post_with_headers(&path, &body, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.post(&path, &body).await?
        };
        Ok(envelope_data(resp))
    }

    pub async fn create_task(&self, data: &serde_json::Value) -> Result<Task, ScientexError> {
        let resp: serde_json::Value = self.http.post(tasks_path(), data).await?;
        extract_object(resp)
    }

    pub async fn list_tasks(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<TaskSummary>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&tasks_query_path(skip, limit)).await?;
        extract_paginated(resp)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Task, ScientexError> {
        let resp: serde_json::Value = self.http.get(&task_path(task_id)).await?;
        extract_object(resp)
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(&task_cancel_path(task_id), &empty_body())
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn get_task_part(
        &self,
        task_id: &str,
        part_id: &str,
    ) -> Result<crate::types::TaskPart, ScientexError> {
        let resp: serde_json::Value = self.http.get(&task_part_path(task_id, part_id)).await?;
        extract_object(resp)
    }

    pub async fn add_task_part(
        &self,
        task_id: &str,
        name: &str,
        description: Option<&str>,
        sort_order: i64,
    ) -> Result<crate::types::TaskPart, ScientexError> {
        let mut fields = vec![
            ("name", name.to_string()),
            ("sort_order", sort_order.to_string()),
        ];
        if let Some(description) = description {
            fields.push(("description", description.to_string()));
        }
        let resp: serde_json::Value = self
            .http
            .post_form(&task_parts_path(task_id), &fields)
            .await?;
        extract_object(resp)
    }

    pub async fn update_task_part(
        &self,
        task_id: &str,
        part_id: &str,
        data: &serde_json::Value,
    ) -> Result<crate::types::TaskPart, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .patch(&task_part_path(task_id, part_id), data)
            .await?;
        extract_object(resp)
    }

    pub async fn create_task_assignment(
        &self,
        task_id: &str,
        part_id: &str,
        assignee_id: &str,
        role: &str,
    ) -> Result<crate::types::TaskAssignment, ScientexError> {
        let body = serde_json::json!({
            "part_id": part_id,
            "assignee_id": assignee_id,
            "role": role,
        });
        let resp: serde_json::Value = self
            .http
            .post(&task_assignments_path(task_id), &body)
            .await?;
        extract_object(resp)
    }

    pub async fn delete_task_assignment(
        &self,
        task_id: &str,
        assignment_id: &str,
    ) -> Result<(), ScientexError> {
        self.http
            .delete_empty(&task_assignment_path(task_id, assignment_id))
            .await
    }

    pub async fn list_task_results(
        &self,
        task_id: &str,
    ) -> Result<PaginatedList<TaskResult>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&task_results_path(task_id)).await?;
        extract_paginated(resp)
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
        search: Option<&str>,
        exclude_status: Option<&str>,
    ) -> Result<PaginatedList<StaffAssignmentItem>, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&staff_task_assignments_path(
                skip,
                limit,
                search,
                exclude_status,
            ))
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

    pub async fn complete_my_task_assignment(
        &self,
        assignment_id: &str,
        data: &serde_json::Value,
    ) -> Result<TaskResult, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(&staff_task_assignment_complete_path(assignment_id), data)
            .await?;
        extract_object(resp)
    }

    pub async fn upload_my_task_field(
        &self,
        task_id: &str,
        file_path: &str,
        field_key: &str,
        visibility: &str,
    ) -> Result<UploadFieldResponse, ScientexError> {
        let fields = [("field_key", field_key), ("visibility", visibility)];
        let resp: serde_json::Value = self
            .http
            .upload_multipart(
                &staff_task_upload_field_path(task_id),
                file_path,
                &fields,
                &[],
            )
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
        if let Some(part_id) = part_id {
            fields.push(("part_id", part_id));
        }
        let path = task_documents_path(task_id);
        let resp: serde_json::Value = self
            .http
            .upload_multipart(&path, file_path, &fields, &[])
            .await?;
        extract_object(resp)
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

    pub async fn download_task_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<u8>, ScientexError> {
        self.http
            .download_bytes(&task_document_download_path(document_id))
            .await
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

fn tasks_query_path(skip: u32, limit: u32) -> String {
    format!("/tasks?skip={skip}&limit={limit}")
}

fn task_path(task_id: &str) -> String {
    format!("/tasks/{}", path_segment_encode(task_id))
}

fn task_cancel_path(task_id: &str) -> String {
    format!("{}/cancel", task_path(task_id))
}

fn task_parts_path(task_id: &str) -> String {
    format!("{}/parts", task_path(task_id))
}

fn task_part_path(task_id: &str, part_id: &str) -> String {
    format!(
        "{}/{}",
        task_parts_path(task_id),
        path_segment_encode(part_id)
    )
}

fn task_assignments_path(task_id: &str) -> String {
    format!("{}/assignments", task_path(task_id))
}

fn task_assignment_path(task_id: &str, assignment_id: &str) -> String {
    format!(
        "{}/{}",
        task_assignments_path(task_id),
        path_segment_encode(assignment_id)
    )
}

fn task_results_path(task_id: &str) -> String {
    format!("{}/results", task_path(task_id))
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

fn lab_task_types_path(
    skip: u32,
    limit: u32,
    search: Option<&str>,
    category: Option<&str>,
) -> String {
    let mut path = format!("/lab/tasks/task-types?skip={skip}&limit={limit}");
    if let Some(search) = search.filter(|value| !value.is_empty()) {
        path.push_str("&search=");
        path.push_str(&url_encode(search));
    }
    if let Some(category) = category.filter(|value| !value.is_empty()) {
        path.push_str("&category=");
        path.push_str(&url_encode(category));
    }
    path
}

fn lab_task_type_path(task_type_id: &str) -> String {
    format!(
        "/lab/tasks/task-types/{}",
        path_segment_encode(task_type_id)
    )
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

fn lab_task_part_path(task_id: &str, part_id: &str) -> String {
    format!(
        "/lab/tasks/{}/parts/{}",
        path_segment_encode(task_id),
        path_segment_encode(part_id)
    )
}

fn lab_task_part_release_schedule_path(task_id: &str, part_id: &str) -> String {
    format!("{}/release-schedule", lab_task_part_path(task_id, part_id))
}

fn lab_task_part_run_rerun_path(task_id: &str, part_id: &str, run_id: &str) -> String {
    format!(
        "{}/runs/{}/rerun",
        lab_task_part_path(task_id, part_id),
        path_segment_encode(run_id)
    )
}

fn lab_task_run_artifact_preview_path(
    task_id: &str,
    part_id: &str,
    run_id: &str,
    artifact_index: u64,
) -> String {
    format!(
        "{}/runs/{}/artifacts/{artifact_index}/preview",
        lab_task_part_path(task_id, part_id),
        path_segment_encode(run_id)
    )
}

fn lab_task_confirm_path(task_id: &str) -> String {
    format!("{}/confirm", lab_task_path(task_id))
}

fn lab_task_reject_path(task_id: &str) -> String {
    format!("{}/reject", lab_task_path(task_id))
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

fn staff_task_assignments_path(
    skip: u32,
    limit: u32,
    search: Option<&str>,
    exclude_status: Option<&str>,
) -> String {
    let mut path = format!("/staff/tasks/assignments?skip={skip}&limit={limit}");
    if let Some(search) = search.filter(|value| !value.is_empty()) {
        path.push_str("&search=");
        path.push_str(&url_encode(search));
    }
    if let Some(status) = exclude_status.filter(|value| !value.is_empty()) {
        path.push_str("&exclude_status=");
        path.push_str(&url_encode(status));
    }
    path
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

fn staff_task_assignment_complete_path(assignment_id: &str) -> String {
    format!(
        "/staff/tasks/assignments/{}/complete",
        path_segment_encode(assignment_id)
    )
}

fn staff_task_upload_field_path(task_id: &str) -> String {
    format!("/staff/tasks/{}/upload-field", path_segment_encode(task_id))
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

fn task_document_download_path(document_id: &str) -> String {
    format!(
        "/tasks/documents/{}/download",
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
        assert_eq!(
            lab_task_types_path(0, 20, None, None),
            "/lab/tasks/task-types?skip=0&limit=20"
        );
        assert_eq!(
            lab_task_types_path(10, 50, Some("sample qc"), Some("COMPUTE")),
            "/lab/tasks/task-types?skip=10&limit=50&search=sample+qc&category=COMPUTE"
        );
        assert_eq!(
            lab_task_type_path("type /1"),
            "/lab/tasks/task-types/type%20%2F1"
        );
        assert_eq!(lab_tasks_path(10, 25), "/lab/tasks?skip=10&limit=25");
        assert_eq!(lab_tasks_create_path(), "/lab/tasks");
        assert_eq!(lab_task_path("task 1"), "/lab/tasks/task%201");
        assert_eq!(
            lab_task_part_path("task 1", "part/1"),
            "/lab/tasks/task%201/parts/part%2F1"
        );
        assert_eq!(
            lab_task_part_release_schedule_path("task 1", "part/1"),
            "/lab/tasks/task%201/parts/part%2F1/release-schedule"
        );
        assert_eq!(
            lab_task_part_run_rerun_path("task 1", "part/1", "run/1"),
            "/lab/tasks/task%201/parts/part%2F1/runs/run%2F1/rerun"
        );
        assert_eq!(
            lab_task_run_artifact_preview_path("task 1", "part/1", "run/1", 2),
            "/lab/tasks/task%201/parts/part%2F1/runs/run%2F1/artifacts/2/preview"
        );
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
        assert_eq!(
            lab_task_confirm_path("task 1"),
            "/lab/tasks/task%201/confirm"
        );
        assert_eq!(lab_task_reject_path("task 1"), "/lab/tasks/task%201/reject");
    }

    #[test]
    fn builds_general_task_paths() {
        assert_eq!(tasks_path(), "/tasks");
        assert_eq!(tasks_query_path(20, 50), "/tasks?skip=20&limit=50");
        assert_eq!(task_path("task 1"), "/tasks/task%201");
        assert_eq!(task_cancel_path("task 1"), "/tasks/task%201/cancel");
        assert_eq!(
            task_part_path("task 1", "part/1"),
            "/tasks/task%201/parts/part%2F1"
        );
        assert_eq!(
            task_assignment_path("task 1", "assignment/1"),
            "/tasks/task%201/assignments/assignment%2F1"
        );
        assert_eq!(task_results_path("task 1"), "/tasks/task%201/results");
        assert_eq!(task_workflow_path("task 1"), "/tasks/task%201/workflow");
        assert_eq!(task_type_path("type 1"), "/task-types/type%201");
        assert_eq!(task_documents_path("task 1"), "/tasks/task%201/documents");
        assert_eq!(
            task_document_path("task 1", "doc 1"),
            "/tasks/task%201/documents/doc%201"
        );
        assert_eq!(
            task_document_download_path("doc 1"),
            "/tasks/documents/doc%201/download"
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
                Some(r#"[{"field":"category","operator":"eq","value":"COMPUTE"}]"#)
            ),
            "/task-types?skip=0&limit=20&search=ngs&filters=%5B%7B%22field%22%3A%22category%22%2C%22operator%22%3A%22eq%22%2C%22value%22%3A%22COMPUTE%22%7D%5D"
        );
    }

    #[test]
    fn builds_staff_task_paths() {
        assert_eq!(
            staff_task_assignments_path(0, 100, None, None),
            "/staff/tasks/assignments?skip=0&limit=100"
        );
        assert_eq!(
            staff_task_assignments_path(10, 20, Some("sample qc"), Some("COMPLETED")),
            "/staff/tasks/assignments?skip=10&limit=20&search=sample+qc&exclude_status=COMPLETED"
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
            staff_task_assignment_complete_path("assignment 1"),
            "/staff/tasks/assignments/assignment%201/complete"
        );
        assert_eq!(
            staff_task_upload_field_path("task 1"),
            "/staff/tasks/task%201/upload-field"
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
