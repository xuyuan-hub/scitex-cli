use crate::api_response::extract_object;
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::types::{FeishuUserSettingsPublic, FeishuUserSettingsUpdate};

impl ScientexClient {
    /// GET /feishu/settings — read the current user's Feishu identity and docs config.
    pub async fn get_feishu_settings(&self) -> Result<FeishuUserSettingsPublic, ScientexError> {
        let resp: serde_json::Value = self.http.get("/feishu/settings").await?;
        extract_object(resp)
    }

    /// PATCH /feishu/settings — update docs_folder_token.
    pub async fn update_feishu_settings(
        &self,
        update: &FeishuUserSettingsUpdate,
    ) -> Result<FeishuUserSettingsPublic, ScientexError> {
        let resp: serde_json::Value = self.http.patch("/feishu/settings", update).await?;
        extract_object(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn feishu_settings_public_parses_full_response() {
        let value: FeishuUserSettingsPublic = serde_json::from_value(json!({
            "open_id": "ou_abc",
            "name": "张三",
            "email": "zhang@example.com",
            "mobile": "13800000000",
            "avatar_url": "https://example.com/avatar.png",
            "docs_folder_token": "fld_xyz"
        }))
        .expect("should parse full feishu settings");
        assert_eq!(value.open_id.as_deref(), Some("ou_abc"));
        assert_eq!(value.docs_folder_token.as_deref(), Some("fld_xyz"));
    }

    #[test]
    fn feishu_settings_public_handles_null_fields() {
        let value: FeishuUserSettingsPublic = serde_json::from_value(json!({
            "open_id": null,
            "name": null,
            "email": null,
            "mobile": null,
            "avatar_url": null,
            "docs_folder_token": null
        }))
        .expect("should parse null feishu settings");
        assert_eq!(value.open_id, None);
        assert_eq!(value.docs_folder_token, None);
    }

    #[test]
    fn feishu_settings_public_handles_missing_fields() {
        let value: FeishuUserSettingsPublic = serde_json::from_value(json!({}))
            .expect("should parse empty feishu settings");
        assert_eq!(value.open_id, None);
        assert_eq!(value.docs_folder_token, None);
    }

    #[test]
    fn feishu_settings_update_serializes_partial() {
        let update = FeishuUserSettingsUpdate {
            docs_folder_token: Some("fld_new".to_string()),
        };
        let json = serde_json::to_value(&update).expect("should serialize");
        assert_eq!(json["docs_folder_token"], "fld_new");
    }

    #[test]
    fn feishu_settings_update_omits_none() {
        let update = FeishuUserSettingsUpdate {
            docs_folder_token: None,
        };
        let json = serde_json::to_value(&update).expect("should serialize");
        assert!(json.get("docs_folder_token").is_none());
    }
}
