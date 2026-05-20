use crate::api_response::{envelope_data, extract_array, extract_object};
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::types::{CreatePrimerOrder, CreateSequencingOrder, Order};

impl BiolabClient {
    pub async fn list_orders(&self, skip: u32, limit: u32) -> Result<Vec<Order>, BiolabError> {
        let resp: serde_json::Value = self.http.get(&list_orders_path(skip, limit)).await?;
        extract_array(resp)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.http.get(&order_path(order_id)).await?;
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
        let resp: serde_json::Value = self.http.patch(&order_path(order_id), data).await?;
        extract_object(resp)
    }

    pub async fn resend_order(&self, order_id: &str) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(&resend_order_path(order_id), &serde_json::json!({}))
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn download_order(&self, order_id: &str) -> Result<Vec<u8>, BiolabError> {
        self.http
            .download_bytes(&download_order_path(order_id))
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
        let resp: serde_json::Value = self
            .http
            .upload_file("/orders/primer/upload-excel", file_path)
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn upload_sequencing_excel(
        &self,
        file_path: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .upload_file("/orders/sequencing/upload-excel", file_path)
            .await?;
        Ok(envelope_data(resp))
    }
}

fn list_orders_path(skip: u32, limit: u32) -> String {
    format!("/orders/?skip={skip}&limit={limit}")
}

fn order_path(order_id: &str) -> String {
    format!("/orders/{order_id}")
}

fn resend_order_path(order_id: &str) -> String {
    format!("/orders/{order_id}/send")
}

fn download_order_path(order_id: &str) -> String {
    format!("/orders/{order_id}/download")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_order_collection_path_with_pagination() {
        assert_eq!(list_orders_path(20, 50), "/orders/?skip=20&limit=50");
    }

    #[test]
    fn builds_order_detail_and_action_paths() {
        assert_eq!(order_path("ord_123"), "/orders/ord_123");
        assert_eq!(resend_order_path("ord_123"), "/orders/ord_123/send");
        assert_eq!(download_order_path("ord_123"), "/orders/ord_123/download");
    }
}
