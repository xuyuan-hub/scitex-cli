use std::sync::Arc;

use reqwest::Client;
use thiserror::Error;

use crate::config::Config;
use crate::types::*;

#[derive(Debug, Error)]
pub enum BiolabError {
    #[error("HTTP {status} {path}: {detail}")]
    HttpError {
        status: u16,
        path: String,
        detail: String,
    },
    #[error("Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Token not found. Run `biolab login` first.")]
    NotAuthenticated,
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct BiolabClient {
    client: Client,
    config: Arc<Config>,
}

impl BiolabClient {
    pub fn new(config: Arc<Config>) -> Result<Self, BiolabError> {
        let token = config.load_token().ok_or(BiolabError::NotAuthenticated)?;
        let client = Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                        .unwrap(),
                );
                h.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json"),
                );
                h
            })
            .build()
            .map_err(BiolabError::RequestError)?;
        Ok(Self { client, config })
    }

    pub fn with_token(config: Arc<Config>, token: &str) -> Result<Self, BiolabError> {
        config.save_token(token).map_err(|e| BiolabError::ParseError(e.to_string()))?;
        Self::new(config)
    }

    // ---- internals ----

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, BiolabError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client.get(&url).send().await.map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BiolabError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    async fn patch<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BiolabError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client
            .patch(&url)
            .json(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    async fn put<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BiolabError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client
            .put(&url)
            .json(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, BiolabError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client.delete(&url).send().await.map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    async fn download_bytes(&self, path: &str) -> Result<Vec<u8>, BiolabError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client.get(&url).send().await.map_err(BiolabError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(BiolabError::HttpError { status, path: path.into(), detail });
        }
        resp.bytes().await.map(|b| b.to_vec()).map_err(BiolabError::RequestError)
    }

    // ---- users ----

    pub async fn get_me(&self) -> Result<User, BiolabError> {
        let resp: serde_json::Value = self.get("/users/me").await?;
        extract_object(resp)
    }

    pub async fn update_me(&self, data: &serde_json::Value) -> Result<User, BiolabError> {
        let resp: serde_json::Value = self.patch("/users/me", data).await?;
        extract_object(resp)
    }

    pub async fn change_password(&self, current: &str, new: &str) -> Result<serde_json::Value, BiolabError> {
        self.patch("/users/me/password", &serde_json::json!({
            "current_password": current,
            "new_password": new,
        })).await
    }

    // ---- orders ----

    pub async fn list_orders(&self, skip: u32, limit: u32) -> Result<Vec<Order>, BiolabError> {
        let resp: serde_json::Value = self.get(&format!("/orders/?skip={skip}&limit={limit}")).await?;
        extract_array(resp)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.get(&format!("/orders/{order_id}")).await?;
        extract_object(resp)
    }

    pub async fn create_primer_order(&self, order: &CreatePrimerOrder) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.post("/orders/primer", order).await?;
        extract_object(resp)
    }

    pub async fn create_sequencing_order(&self, order: &CreateSequencingOrder) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.post("/orders/sequencing", order).await?;
        extract_object(resp)
    }

    pub async fn update_order(&self, order_id: &str, data: &serde_json::Value) -> Result<Order, BiolabError> {
        let resp: serde_json::Value = self.patch(&format!("/orders/{order_id}"), data).await?;
        extract_object(resp)
    }

    pub async fn resend_order(&self, order_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/orders/{order_id}/send"), &serde_json::json!({})).await
    }

    pub async fn download_order(&self, order_id: &str) -> Result<Vec<u8>, BiolabError> {
        self.download_bytes(&format!("/orders/{order_id}/download")).await
    }

    pub async fn download_primer_template(&self) -> Result<Vec<u8>, BiolabError> {
        self.download_bytes("/orders/primer/template").await
    }

    pub async fn download_sequencing_template(&self) -> Result<Vec<u8>, BiolabError> {
        self.download_bytes("/orders/sequencing/template").await
    }

    pub async fn upload_primer_excel(&self, file_path: &str) -> Result<serde_json::Value, BiolabError> {
        self.upload_file("/orders/primer/upload-excel", file_path).await
    }

    pub async fn upload_sequencing_excel(&self, file_path: &str) -> Result<serde_json::Value, BiolabError> {
        self.upload_file("/orders/sequencing/upload-excel", file_path).await
    }

    async fn upload_file(&self, path: &str, file_path: &str) -> Result<serde_json::Value, BiolabError> {
        use std::path::Path as StdPath;
        let fname = StdPath::new(file_path).file_name().unwrap_or_default().to_string_lossy();
        let content = std::fs::read(file_path)
            .map_err(|e| BiolabError::ParseError(format!("Cannot read file {file_path}: {e}")))?;

        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"file\"; filename=\"{fname}\"\r\n").as_bytes(),
        );
        body.extend_from_slice(
            b"Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\r\n\r\n",
        );
        body.extend_from_slice(&content);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let url = format!("{}{}", self.config.base_url, path);
        let resp = self.client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .body(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    // ---- templates ----

    pub async fn list_templates(&self) -> Result<Vec<Template>, BiolabError> {
        let resp: serde_json::Value = self.get("/order-info-templates/").await?;
        extract_array(resp)
    }

    pub async fn get_template(&self, id: &str) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.get(&format!("/order-info-templates/{id}")).await?;
        extract_object(resp)
    }

    pub async fn get_default_template(&self, order_type: Option<&str>) -> Result<Template, BiolabError> {
        let qs = if let Some(ot) = order_type {
            format!("/order-info-templates/default?order_type={ot}")
        } else {
            "/order-info-templates/default".to_string()
        };
        let resp: serde_json::Value = self.get(&qs).await?;
        extract_object(resp)
    }

    pub async fn create_template(&self, data: &serde_json::Value) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.post("/order-info-templates/", data).await?;
        extract_object(resp)
    }

    pub async fn update_template(&self, id: &str, data: &serde_json::Value) -> Result<Template, BiolabError> {
        let resp: serde_json::Value = self.put(&format!("/order-info-templates/{id}"), data).await?;
        extract_object(resp)
    }

    pub async fn delete_template(&self, id: &str) -> Result<serde_json::Value, BiolabError> {
        self.delete(&format!("/order-info-templates/{id}")).await
    }

    pub async fn set_default_template(&self, id: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/order-info-templates/{id}/set-default"), &serde_json::json!({})).await
    }

    // ---- inventory ----

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
        let resp: serde_json::Value = self.get(&format!("/inventory/stocks{qs}")).await?;
        extract_array(resp)
    }

    pub async fn get_stock(&self, stock_id: &str) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self.get(&format!("/inventory/stocks/{stock_id}")).await?;
        extract_object(resp)
    }

    pub async fn get_stock_stats(&self) -> Result<StockStats, BiolabError> {
        let resp: serde_json::Value = self.get("/inventory/stats").await?;
        extract_object(resp)
    }

    pub async fn checkin(
        &self,
        stock_id: &str,
        quantity: f64,
        purpose: &str,
    ) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self.post(&format!("/inventory/stocks/{stock_id}/checkin"), &serde_json::json!({
            "quantity": quantity,
            "purpose": purpose,
        })).await?;
        extract_object(resp)
    }

    pub async fn checkout(
        &self,
        stock_id: &str,
        quantity: f64,
        purpose: &str,
        experiment_ref: &str,
    ) -> Result<Stock, BiolabError> {
        let resp: serde_json::Value = self.post(&format!("/inventory/stocks/{stock_id}/checkout"), &serde_json::json!({
            "quantity": quantity,
            "purpose": purpose,
            "experiment_ref": experiment_ref,
        })).await?;
        extract_object(resp)
    }

    pub async fn list_locations(&self) -> Result<Vec<Location>, BiolabError> {
        let resp: serde_json::Value = self.get("/inventory/locations").await?;
        extract_array(resp)
    }

    pub async fn create_location(&self, name: &str, parent_id: Option<&str>) -> Result<Location, BiolabError> {
        let mut data = serde_json::json!({ "name": name });
        if let Some(pid) = parent_id {
            data["parent_id"] = serde_json::Value::String(pid.to_string());
        }
        let resp: serde_json::Value = self.post("/inventory/locations", &data).await?;
        extract_object(resp)
    }

    // ---- lab ----

    pub async fn get_lab(&self) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.get("/lab").await?;
        extract_object(resp)
    }

    pub async fn create_lab(&self, name: &str) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.post("/lab/create", &serde_json::json!({ "name": name })).await?;
        extract_object(resp)
    }

    pub async fn update_lab(&self, data: &serde_json::Value) -> Result<Lab, BiolabError> {
        let resp: serde_json::Value = self.patch("/lab", data).await?;
        extract_object(resp)
    }

    pub async fn list_lab_members(&self) -> Result<Vec<LabMember>, BiolabError> {
        let resp: serde_json::Value = self.get("/lab/members").await?;
        extract_array(resp)
    }

    pub async fn update_member_role(&self, user_id: &str, role: &str) -> Result<serde_json::Value, BiolabError> {
        self.patch(&format!("/lab/members/{user_id}"), &serde_json::json!({ "role": role })).await
    }

    pub async fn remove_member(&self, user_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.delete(&format!("/lab/members/{user_id}")).await
    }

    pub async fn invite_member(&self, email: &str, role: &str) -> Result<serde_json::Value, BiolabError> {
        self.post("/lab/invite", &serde_json::json!({ "email": email, "role": role })).await
    }

    pub async fn list_invitations(&self) -> Result<Vec<Invitation>, BiolabError> {
        let resp: serde_json::Value = self.get("/lab/invitations").await?;
        extract_array(resp)
    }

    pub async fn accept_invitation(&self, invitation_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/lab/invitations/{invitation_id}/accept"), &serde_json::json!({})).await
    }

    pub async fn decline_invitation(&self, invitation_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/lab/invitations/{invitation_id}/decline"), &serde_json::json!({})).await
    }

    pub async fn apply_to_join_lab(&self, lab_id: &str, role: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/lab/join/{lab_id}"), &serde_json::json!({ "role": role })).await
    }

    pub async fn list_applications(&self) -> Result<Vec<Application>, BiolabError> {
        let resp: serde_json::Value = self.get("/lab/applications").await?;
        extract_array(resp)
    }

    pub async fn approve_application(&self, app_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/lab/applications/{app_id}/approve"), &serde_json::json!({})).await
    }

    pub async fn reject_application(&self, app_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.post(&format!("/lab/applications/{app_id}/reject"), &serde_json::json!({})).await
    }

    pub async fn list_approval_rules(&self) -> Result<Vec<ApprovalRule>, BiolabError> {
        let resp: serde_json::Value = self.get("/lab/approval-rules").await?;
        extract_array(resp)
    }

    pub async fn add_approval_rule(&self, data: &serde_json::Value) -> Result<ApprovalRule, BiolabError> {
        let resp: serde_json::Value = self.post("/lab/approval-rules", data).await?;
        extract_object(resp)
    }

    pub async fn remove_approval_rule(&self, rule_id: &str) -> Result<serde_json::Value, BiolabError> {
        self.delete(&format!("/lab/approval-rules/{rule_id}")).await
    }
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
    path: &str,
) -> Result<T, BiolabError> {
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let detail = resp.text().await.unwrap_or_default();
        return Err(BiolabError::HttpError {
            status,
            path: path.to_string(),
            detail,
        });
    }
    resp.json::<T>().await.map_err(BiolabError::RequestError)
}

fn extract_array<T: serde::de::DeserializeOwned>(resp: serde_json::Value) -> Result<Vec<T>, BiolabError> {
    if let Some(arr) = resp.as_array() {
        Ok(serde_json::from_value(serde_json::Value::Array(arr.clone())).unwrap_or_default())
    } else if let Some(data) = resp.get("data") {
        if let Some(arr) = data.as_array() {
            Ok(serde_json::from_value(serde_json::Value::Array(arr.clone())).unwrap_or_default())
        } else {
            Ok(serde_json::from_value(data.clone()).unwrap_or_default())
        }
    } else {
        Ok(vec![])
    }
}

fn extract_object<T: serde::de::DeserializeOwned>(resp: serde_json::Value) -> Result<T, BiolabError> {
    if let Some(data) = resp.get("data") {
        serde_json::from_value(data.clone()).map_err(|e| BiolabError::ParseError(e.to_string()))
    } else {
        serde_json::from_value(resp).map_err(|e| BiolabError::ParseError(e.to_string()))
    }
}

fn url_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
