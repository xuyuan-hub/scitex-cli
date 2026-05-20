use crate::api_response::{extract_array, extract_object};
use crate::client::{BiolabClient, BiolabError};
use crate::types::{Application, ApprovalRule, Invitation, Lab, LabMember};

impl BiolabClient {
    pub async fn get_lab(&self) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab").await?;
        extract_object(resp)
    }

    pub async fn create_lab(&self, name: &str) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post("/lab/create", &serde_json::json!({ "name": name }))
            .await?;
        extract_object(resp)
    }

    pub async fn update_lab(&self, data: &serde_json::Value) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.http.patch("/lab", data).await?;
        extract_object(resp)
    }

    pub async fn list_lab_members(&self) -> Result<Vec<LabMember>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab/members").await?;
        extract_array(resp)
    }

    pub async fn update_member_role(
        &self,
        user_id: &str,
        role: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .patch(
                &format!("/lab/members/{user_id}"),
                &serde_json::json!({ "role": role }),
            )
            .await
    }

    pub async fn remove_member(&self, user_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.http.delete(&format!("/lab/members/{user_id}")).await
    }

    pub async fn invite_member(
        &self,
        email: &str,
        role: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                "/lab/invite",
                &serde_json::json!({ "email": email, "role": role }),
            )
            .await
    }

    pub async fn list_invitations(&self) -> Result<Vec<Invitation>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab/invitations").await?;
        extract_array(resp)
    }

    pub async fn accept_invitation(
        &self,
        invitation_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                &format!("/lab/invitations/{invitation_id}/accept"),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn decline_invitation(
        &self,
        invitation_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                &format!("/lab/invitations/{invitation_id}/decline"),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn apply_to_join_lab(
        &self,
        lab_id: &str,
        role: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                &format!("/lab/join/{lab_id}"),
                &serde_json::json!({ "role": role }),
            )
            .await
    }

    pub async fn list_applications(&self) -> Result<Vec<Application>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab/applications").await?;
        extract_array(resp)
    }

    pub async fn approve_application(
        &self,
        app_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                &format!("/lab/applications/{app_id}/approve"),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn reject_application(&self, app_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                &format!("/lab/applications/{app_id}/reject"),
                &serde_json::json!({}),
            )
            .await
    }

    pub async fn list_approval_rules(&self) -> Result<Vec<ApprovalRule>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab/approval-rules").await?;
        extract_array(resp)
    }

    pub async fn add_approval_rule(
        &self,
        data: &serde_json::Value,
    ) -> Result<ApprovalRule, BiolabError> {
        let resp: serde_json::Value = self.http.post("/lab/approval-rules", data).await?;
        extract_object(resp)
    }

    pub async fn remove_approval_rule(
        &self,
        rule_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .delete(&format!("/lab/approval-rules/{rule_id}"))
            .await
    }
}
