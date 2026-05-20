use crate::api_response::{extract_array, extract_object};
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::services::url_encode;
use crate::types::{Location, Stock, StockStats};

impl BiolabClient {
    pub async fn list_stocks(
        &self,
        primer_name: Option<&str>,
        location_id: Option<&str>,
        low_stock: bool,
    ) -> Result<Vec<Stock>, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .get(&list_stocks_path(primer_name, location_id, low_stock))
            .await?;
        extract_array(resp)
    }

    pub async fn get_stock(&self, stock_id: &str) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self.http.get(&stock_path(stock_id)).await?;
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
                &checkin_path(stock_id),
                &stock_change_body(quantity, purpose),
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
                &checkout_path(stock_id),
                &checkout_body(quantity, purpose, experiment_ref),
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
        let data = create_location_body(name, parent_id);
        let resp: serde_json::Value = self.http.post("/inventory/locations", &data).await?;
        extract_object(resp)
    }
}

fn list_stocks_path(
    primer_name: Option<&str>,
    location_id: Option<&str>,
    low_stock: bool,
) -> String {
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
    if params.is_empty() {
        "/inventory/stocks".to_string()
    } else {
        format!("/inventory/stocks?{}", params.join("&"))
    }
}

fn stock_path(stock_id: &str) -> String {
    format!("/inventory/stocks/{stock_id}")
}

fn checkin_path(stock_id: &str) -> String {
    format!("/inventory/stocks/{stock_id}/checkin")
}

fn checkout_path(stock_id: &str) -> String {
    format!("/inventory/stocks/{stock_id}/checkout")
}

fn stock_change_body(quantity: f64, purpose: &str) -> serde_json::Value {
    serde_json::json!({
        "quantity": quantity,
        "purpose": purpose,
    })
}

fn checkout_body(quantity: f64, purpose: &str, experiment_ref: &str) -> serde_json::Value {
    serde_json::json!({
        "quantity": quantity,
        "purpose": purpose,
        "experiment_ref": experiment_ref,
    })
}

fn create_location_body(name: &str, parent_id: Option<&str>) -> serde_json::Value {
    let mut data = serde_json::json!({ "name": name });
    if let Some(pid) = parent_id {
        data["parent_id"] = serde_json::Value::String(pid.to_string());
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_stock_list_path_without_filters() {
        assert_eq!(list_stocks_path(None, None, false), "/inventory/stocks");
    }

    #[test]
    fn builds_stock_list_path_with_encoded_filters() {
        assert_eq!(
            list_stocks_path(Some("primer A+B"), Some("loc-1"), true),
            "/inventory/stocks?primer_name=primer+A%2BB&location_id=loc-1&low_stock=true"
        );
    }

    #[test]
    fn builds_stock_action_paths() {
        assert_eq!(stock_path("stock-1"), "/inventory/stocks/stock-1");
        assert_eq!(checkin_path("stock-1"), "/inventory/stocks/stock-1/checkin");
        assert_eq!(
            checkout_path("stock-1"),
            "/inventory/stocks/stock-1/checkout"
        );
    }

    #[test]
    fn builds_stock_change_bodies() {
        assert_eq!(
            stock_change_body(2.5, "restock"),
            serde_json::json!({ "quantity": 2.5, "purpose": "restock" })
        );
        assert_eq!(
            checkout_body(1.0, "experiment", "exp-7"),
            serde_json::json!({
                "quantity": 1.0,
                "purpose": "experiment",
                "experiment_ref": "exp-7"
            })
        );
    }

    #[test]
    fn builds_location_body_with_optional_parent() {
        assert_eq!(
            create_location_body("Freezer A", None),
            serde_json::json!({ "name": "Freezer A" })
        );
        assert_eq!(
            create_location_body("Shelf 1", Some("freezer-a")),
            serde_json::json!({ "name": "Shelf 1", "parent_id": "freezer-a" })
        );
    }
}
