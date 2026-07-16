use crate::api_response::{envelope_data, extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::{path_segment_encode, url_encode};
use crate::types::Order;

impl ScientexClient {
    pub async fn list_orders(
        &self,
        skip: u32,
        limit: u32,
        order_type: Option<&str>,
        supplier_name: Option<&str>,
        status: Option<&str>,
        price_min: Option<&str>,
        price_max: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<PaginatedList<Order>, ScientexError> {
        let path = list_orders_path(
            skip,
            limit,
            order_type,
            supplier_name,
            status,
            price_min,
            price_max,
            date_from,
            date_to,
        );
        let resp: serde_json::Value = self.http.get(&path).await?;
        extract_paginated(resp)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, ScientexError> {
        let resp: serde_json::Value = self.http.get(&order_path(order_id)).await?;
        extract_object(resp)
    }

    pub async fn get_order_stats(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&order_stats_path(start_date, end_date))
            .await?;
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

fn list_orders_path(
    skip: u32,
    limit: u32,
    order_type: Option<&str>,
    supplier_name: Option<&str>,
    status: Option<&str>,
    price_min: Option<&str>,
    price_max: Option<&str>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> String {
    let mut path = format!("/orders/?skip={skip}&limit={limit}");
    for (name, value) in [
        ("order_type", order_type),
        ("supplier_name", supplier_name),
        ("status", status),
        ("price_min", price_min),
        ("price_max", price_max),
        ("date_from", date_from),
        ("date_to", date_to),
    ] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            path.push('&');
            path.push_str(name);
            path.push('=');
            path.push_str(&url_encode(value));
        }
    }
    path
}

fn order_path(order_id: &str) -> String {
    format!("/orders/{}", path_segment_encode(order_id))
}

fn order_stats_path(start_date: Option<&str>, end_date: Option<&str>) -> String {
    let mut path = "/orders/stats".to_string();
    let mut separator = '?';
    for (name, value) in [("start_date", start_date), ("end_date", end_date)] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            path.push(separator);
            separator = '&';
            path.push_str(name);
            path.push('=');
            path.push_str(&url_encode(value));
        }
    }
    path
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
        assert_eq!(
            list_orders_path(20, 50, None, None, None, None, None, None, None),
            "/orders/?skip=20&limit=50"
        );
    }

    #[test]
    fn builds_order_stats_path_with_dates() {
        assert_eq!(order_stats_path(None, None), "/orders/stats");
        assert_eq!(
            order_stats_path(Some("2026-07-01"), Some("2026-07-31")),
            "/orders/stats?start_date=2026-07-01&end_date=2026-07-31"
        );
    }

    #[test]
    fn builds_order_collection_path_with_filters() {
        assert_eq!(
            list_orders_path(
                0,
                100,
                Some("primer synthesis"),
                Some("Sangon & Co"),
                Some("pending_approval"),
                Some("10.50"),
                Some("200"),
                Some("2026-07-01"),
                Some("2026-07-31"),
            ),
            "/orders/?skip=0&limit=100&order_type=primer+synthesis&supplier_name=Sangon+%26+Co&status=pending_approval&price_min=10.50&price_max=200&date_from=2026-07-01&date_to=2026-07-31"
        );
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
