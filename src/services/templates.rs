use crate::api_response::{extract_array, extract_object};
use crate::client::{BiolabClient, BiolabError};
use crate::types::Template;

impl BiolabClient {
    pub async fn list_templates(&self) -> Result<Vec<Template>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/order-info-templates/").await?;
        extract_array(resp)
    }

    pub async fn get_template(&self, id: &str) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .get(&format!("/order-info-templates/{id}"))
            .await?;
        extract_object(resp)
    }

    pub async fn get_default_template(
        &self,
        order_type: Option<&str>,
    ) -> Result<Template, BiolabError> {
        let qs = if let Some(ot) = order_type {
            format!("/order-info-templates/default?order_type={ot}")
        } else {
            "/order-info-templates/default".to_string()
        };
        let resp: serde_json::Value = self.http.get(&qs).await?;
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
        let resp: serde_json::Value = self
            .http
            .put(&format!("/order-info-templates/{id}"), data)
            .await?;
        extract_object(resp)
    }

    pub async fn delete_template(&self, id: &str) -> Result<serde_json::Value, BiolabError> {
        self.http
            .delete(&format!("/order-info-templates/{id}"))
            .await
    }

    pub async fn set_default_template(&self, id: &str) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(
                &format!("/order-info-templates/{id}/set-default"),
                &serde_json::json!({}),
            )
            .await
    }
}
