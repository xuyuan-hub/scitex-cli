use crate::api_response::{extract_array, extract_object};
use crate::client::{BiolabClient, BiolabError};
use crate::types::{CreatePrimerOrder, CreateSequencingOrder, Order};

impl BiolabClient {
    pub async fn list_orders(&self, skip: u32, limit: u32) -> Result<Vec<Order>, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .get(&format!("/orders/?skip={skip}&limit={limit}"))
            .await?;
        extract_array(resp)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.http.get(&format!("/orders/{order_id}")).await?;
        extract_object(resp)
    }

    pub async fn create_primer_order(
        &self,
        order: &CreatePrimerOrder,
    ) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.http.post("/orders/primer", order).await?;
        extract_object(resp)
    }

    pub async fn create_sequencing_order(
        &self,
        order: &CreateSequencingOrder,
    ) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.http.post("/orders/sequencing", order).await?;
        extract_object(resp)
    }

    pub async fn update_order(
        &self,
        order_id: &str,
        data: &serde_json::Value,
    ) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .patch(&format!("/orders/{order_id}"), data)
            .await?;
        extract_object(resp)
    }

    pub async fn resend_order(&self, order_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.http
            .post(&format!("/orders/{order_id}/send"), &serde_json::json!({}))
            .await
    }

    pub async fn download_order(&self, order_id: &str) -> Result<Vec<u8>, BiolabError> {
        self.http
            .download_bytes(&format!("/orders/{order_id}/download"))
            .await
    }

    pub async fn download_primer_template(&self) -> Result<Vec<u8>, BiolabError> {
        self.http.download_bytes("/orders/primer/template").await
    }

    pub async fn download_sequencing_template(&self) -> Result<Vec<u8>, BiolabError> {
        self.http
            .download_bytes("/orders/sequencing/template")
            .await
    }

    pub async fn upload_primer_excel(
        &self,
        file_path: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .upload_file("/orders/primer/upload-excel", file_path)
            .await
    }

    pub async fn upload_sequencing_excel(
        &self,
        file_path: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .upload_file("/orders/sequencing/upload-excel", file_path)
            .await
    }
}
