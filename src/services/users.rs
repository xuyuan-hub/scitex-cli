use crate::api_response::{envelope_data, extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::types::User;

impl ScientexClient {
    pub async fn list_staff_users(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<User>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&staff_users_path(skip, limit)).await?;
        extract_paginated(resp)
    }

    pub async fn get_me(&self) -> Result<User, ScientexError> {
        let resp: serde_json::Value = self.http.get("/users/me").await?;
        extract_object(resp)
    }

    pub async fn update_me(&self, data: &serde_json::Value) -> Result<User, ScientexError> {
        let resp: serde_json::Value = self.http.patch("/users/me", data).await?;
        extract_object(resp)
    }

    pub async fn change_password(
        &self,
        current: &str,
        new: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .patch("/users/me/password", &password_change_body(current, new))
            .await?;
        Ok(envelope_data(resp))
    }
}

fn staff_users_path(skip: u32, limit: u32) -> String {
    format!("/users/staff?skip={skip}&limit={limit}")
}

fn password_change_body(current: &str, new: &str) -> serde_json::Value {
    serde_json::json!({
        "current_password": current,
        "new_password": new,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_password_change_body() {
        assert_eq!(
            password_change_body("old", "new"),
            serde_json::json!({
                "current_password": "old",
                "new_password": "new",
            })
        );
    }

    #[test]
    fn builds_staff_users_path() {
        assert_eq!(staff_users_path(20, 50), "/users/staff?skip=20&limit=50");
    }
}
