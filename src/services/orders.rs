use crate::api_response::{envelope_data, extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::path_segment_encode;
use crate::types::Order;

impl ScientexClient {
    pub async fn list_orders(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<Order>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&list_orders_path(skip, limit)).await?;
        extract_paginated(resp)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, ScientexError> {
        let resp: serde_json::Value = self.http.get(&order_path(order_id)).await?;
        extract_object(resp)
    }

    pub async fn get_order_stats(&self) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self.http.get("/orders/stats").await?;
        Ok(envelope_data(resp))
    }

    pub async fn list_pending_approvals(&self) -> Result<PaginatedList<Order>, ScientexError> {
        let resp: serde_json::Value = self.http.get("/orders/approvals/pending").await?;
        extract_paginated(resp)
    }

    pub async fn create_primer_order(
        &self,
        order: &serde_json::Value,
    ) -> Result<Order, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post("/tasks/submit/primer-synthesis", order)
            .await?;
        extract_object(resp)
    }

    pub async fn create_sequencing_order(
        &self,
        order: &serde_json::Value,
    ) -> Result<Order, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post("/tasks/submit/sanger-sequencing", order)
            .await?;
        extract_object(resp)
    }

    pub async fn update_order(
        &self,
        order_id: &str,
        data: &serde_json::Value,
    ) -> Result<Order, ScientexError> {
        let resp: serde_json::Value = self.http.patch(&order_path(order_id), data).await?;
        extract_object(resp)
    }

    pub async fn resend_order(&self, order_id: &str) -> Result<serde_json::Value, ScientexError> {
        self.send_order(order_id).await
    }

    pub async fn send_order(&self, order_id: &str) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(&send_order_path(order_id), &serde_json::json!({}))
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn approve_order(&self, order_id: &str) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &order_action_path(order_id, "approve"),
                &serde_json::json!({}),
            )
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn reject_order(&self, order_id: &str) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &order_action_path(order_id, "reject"),
                &serde_json::json!({}),
            )
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn download_order(&self, order_id: &str) -> Result<Vec<u8>, ScientexError> {
        self.http
            .download_bytes(&download_order_path(order_id))
            .await
    }

    pub async fn download_primer_template(&self) -> Result<Vec<u8>, ScientexError> {
        self.http.download_bytes("/orders/primer/template").await
    }

    pub async fn download_sequencing_template(&self) -> Result<Vec<u8>, ScientexError> {
        self.http
            .download_bytes("/orders/sequencing/template")
            .await
    }

    pub async fn upload_primer_excel(
        &self,
        file_path: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .upload_file("/orders/primer/upload-excel", file_path)
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn upload_sequencing_excel(
        &self,
        file_path: &str,
    ) -> Result<serde_json::Value, ScientexError> {
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
    format!("/orders/{}", path_segment_encode(order_id))
}

fn send_order_path(order_id: &str) -> String {
    format!("/orders/{}/send", path_segment_encode(order_id))
}

fn order_action_path(order_id: &str, action: &str) -> String {
    format!("/orders/{}/{action}", path_segment_encode(order_id))
}

fn download_order_path(order_id: &str) -> String {
    format!("/orders/{}/download", path_segment_encode(order_id))
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
        assert_eq!(send_order_path("ord_123"), "/orders/ord_123/send");
        assert_eq!(
            order_action_path("ord_123", "approve"),
            "/orders/ord_123/approve"
        );
        assert_eq!(
            order_action_path("ord_123", "reject"),
            "/orders/ord_123/reject"
        );
        assert_eq!(download_order_path("ord_123"), "/orders/ord_123/download");
    }

    #[test]
    fn encodes_order_id_path_segments() {
        assert_eq!(order_path("ord 1/a"), "/orders/ord%201%2Fa");
    }
}
