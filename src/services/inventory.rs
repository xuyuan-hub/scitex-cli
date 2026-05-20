use crate::api_response::{extract_array, extract_object};
use crate::client::{BiolabClient, BiolabError};
use crate::types::{Location, Stock, StockStats};

impl BiolabClient {
    pub async fn list_stocks(
        &self,
        primer_name: Option<&str>,
        location_id: Option<&str>,
        low_stock: bool,
    ) -> Result<Vec<Stock>, BiolabError> {
        let mut params = vec![];
        if let Some(name) = primer_name {
            params.push(format!("primer_name={}", url_encode(name)));
        }
        if let Some(loc) = location_id {
            params.push(format!("location_id={loc}"));
        }
        if low_stock {
            params.push("low_stock=true".to_string());
        }
        let qs = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        let resp: serde_json::Value = self.http.get(&format!("/inventory/stocks{qs}")).await?;
        extract_array(resp)
    }

    pub async fn get_stock(&self, stock_id: &str) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .get(&format!("/inventory/stocks/{stock_id}"))
            .await?;
        extract_object(resp)
    }

    pub async fn get_stock_stats(&self) -> Result<StockStats, BiolabError> {
        let resp: serde_json::Value = self.http.get("/inventory/stats").await?;
        extract_object(resp)
    }

    pub async fn checkin(
        &self,
        stock_id: &str,
        quantity: f64,
        purpose: &str,
    ) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &format!("/inventory/stocks/{stock_id}/checkin"),
                &serde_json::json!({
                    "quantity": quantity,
                    "purpose": purpose,
                }),
            )
            .await?;
        extract_object(resp)
    }

    pub async fn checkout(
        &self,
        stock_id: &str,
        quantity: f64,
        purpose: &str,
        experiment_ref: &str,
    ) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &format!("/inventory/stocks/{stock_id}/checkout"),
                &serde_json::json!({
                    "quantity": quantity,
                    "purpose": purpose,
                    "experiment_ref": experiment_ref,
                }),
            )
            .await?;
        extract_object(resp)
    }

    pub async fn list_locations(&self) -> Result<Vec<Location>, BiolabError> {
        let resp: serde_json::Value = self.http.get("/inventory/locations").await?;
        extract_array(resp)
    }

    pub async fn create_location(
        &self,
        name: &str,
        parent_id: Option<&str>,
    ) -> Result<Location, BiolabError> {
        let mut data = serde_json::json!({ "name": name });
        if let Some(pid) = parent_id {
            data["parent_id"] = serde_json::Value::String(pid.to_string());
        }
        let resp: serde_json::Value = self.http.post("/inventory/locations", &data).await?;
        extract_object(resp)
    }
}

fn url_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
