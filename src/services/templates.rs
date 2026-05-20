use crate::api_response::{envelope_data, extract_array, extract_object};
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::services::url_encode;
use crate::types::Template;

impl BiolabClient {
    pub async fn list_templates(&self) -> Result<Vec<Template>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/order-info-templates/").await?;
        extract_array(resp)
    }

    pub async fn get_template(&self, id: &str) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.http.get(&template_path(id)).await?;
        extract_object(resp)
    }

    pub async fn get_default_template(
        &self,
        order_type: Option<&str>,
    ) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.http.get(&default_template_path(order_type)).await?;
        extract_object(resp)
    }

    pub async fn create_template(&self, data: &serde_json::Value) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.http.post("/order-info-templates/", data).await?;
        extract_object(resp)
    }

    pub async fn update_template(
        &self,
        id: &str,
        data: &serde_json::Value,
    ) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.http.put(&template_path(id), data).await?;
        extract_object(resp)
    }

    pub async fn delete_template(&self, id: &str) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self.http.delete(&template_path(id)).await?;
        Ok(envelope_data(resp))
    }

    pub async fn set_default_template(&self, id: &str) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(&set_default_template_path(id), &serde_json::json!({}))
            .await?;
        Ok(envelope_data(resp))
    }
}

fn template_path(id: &str) -> String {
    format!("/order-info-templates/{id}")
}

fn default_template_path(order_type: Option<&str>) -> String {
    if let Some(order_type) = order_type {
        format!(
            "/order-info-templates/default?order_type={}",
            url_encode(order_type)
        )
    } else {
        "/order-info-templates/default".to_string()
    }
}

fn set_default_template_path(id: &str) -> String {
    format!("/order-info-templates/{id}/set-default")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_template_paths() {
        assert_eq!(template_path("tpl-1"), "/order-info-templates/tpl-1");
        assert_eq!(
            set_default_template_path("tpl-1"),
            "/order-info-templates/tpl-1/set-default"
        );
    }

    #[test]
    fn builds_default_template_path() {
        assert_eq!(default_template_path(None), "/order-info-templates/default");
        assert_eq!(
            default_template_path(Some("primer synthesis")),
            "/order-info-templates/default?order_type=primer+synthesis"
        );
    }
}
