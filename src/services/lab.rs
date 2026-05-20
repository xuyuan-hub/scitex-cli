use crate::api_response::{envelope_data, extract_array, extract_object};
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::services::{empty_body, single_field_body};
use crate::types::{Application, ApprovalRule, Invitation, Lab, LabMember};

impl BiolabClient {
    pub async fn get_lab(&self) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab").await?;
        extract_object(resp)
    }

    pub async fn create_lab(&self, name: &str) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.http.post("/lab/create", &single_field_body("name", name)).await?;
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
        let resp: serde_json::Value = self.http.patch(&member_path(user_id), &single_field_body("role", role)).await?;
        Ok(envelope_data(resp))
    }

    pub async fn remove_member(&self, user_id: &str) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self.http.delete(&member_path(user_id)).await?;
        Ok(envelope_data(resp))
    }

    pub async fn invite_member(
        &self,
        email: &str,
        role: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self.http.post("/lab/invite", &invite_body(email, role)).await?;
        Ok(envelope_data(resp))
    }

    pub async fn list_invitations(&self) -> Result<Vec<Invitation>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab/invitations").await?;
        extract_array(resp)
    }

    pub async fn accept_invitation(
        &self,
        invitation_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &invitation_action_path(invitation_id, "accept"),
                &empty_body(),
            )
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn decline_invitation(
        &self,
        invitation_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &invitation_action_path(invitation_id, "decline"),
                &empty_body(),
            )
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn apply_to_join_lab(
        &self,
        lab_id: &str,
        role: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self.http.post(&join_lab_path(lab_id), &single_field_body("role", role)).await?;
        Ok(envelope_data(resp))
    }

    pub async fn list_applications(&self) -> Result<Vec<Application>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/lab/applications").await?;
        extract_array(resp)
    }

    pub async fn approve_application(
        &self,
        app_id: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(&application_action_path(app_id, "approve"), &empty_body())
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn reject_application(&self, app_id: &str) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(&application_action_path(app_id, "reject"), &empty_body())
            .await?;
        Ok(envelope_data(resp))
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
        let resp: serde_json::Value = self.http.delete(&approval_rule_path(rule_id)).await?;
        Ok(envelope_data(resp))
    }
}

fn invite_body(email: &str, role: &str) -> serde_json::Value {
    serde_json::json!({ "email": email, "role": role })
}

fn member_path(user_id: &str) -> String {
    format!("/lab/members/{user_id}")
}

fn invitation_action_path(invitation_id: &str, action: &str) -> String {
    format!("/lab/invitations/{invitation_id}/{action}")
}

fn join_lab_path(lab_id: &str) -> String {
    format!("/lab/join/{lab_id}")
}

fn application_action_path(app_id: &str, action: &str) -> String {
    format!("/lab/applications/{app_id}/{action}")
}

fn approval_rule_path(rule_id: &str) -> String {
    format!("/lab/approval-rules/{rule_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_lab_bodies() {
        assert_eq!(
            single_field_body("name", "BioLab"),
            serde_json::json!({ "name": "BioLab" })
        );
        assert_eq!(
            single_field_body("role", "admin"),
            serde_json::json!({ "role": "admin" })
        );
        assert_eq!(
            invite_body("pi@example.com", "member"),
            serde_json::json!({ "email": "pi@example.com", "role": "member" })
        );
        assert_eq!(empty_body(), serde_json::json!({}));
    }

    #[test]
    fn builds_lab_member_and_join_paths() {
        assert_eq!(member_path("user-1"), "/lab/members/user-1");
        assert_eq!(join_lab_path("lab-1"), "/lab/join/lab-1");
        assert_eq!(approval_rule_path("rule-1"), "/lab/approval-rules/rule-1");
    }

    #[test]
    fn builds_invitation_and_application_action_paths() {
        assert_eq!(
            invitation_action_path("inv-1", "accept"),
            "/lab/invitations/inv-1/accept"
        );
        assert_eq!(
            invitation_action_path("inv-1", "decline"),
            "/lab/invitations/inv-1/decline"
        );
        assert_eq!(
            application_action_path("app-1", "approve"),
            "/lab/applications/app-1/approve"
        );
        assert_eq!(
            application_action_path("app-1", "reject"),
            "/lab/applications/app-1/reject"
        );
    }
}
