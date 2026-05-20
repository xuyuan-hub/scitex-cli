use crate::api_response::extract_object;
use crate::client::{BiolabClient, BiolabError};
use crate::types::User;

impl BiolabClient {
    pub async fn get_me(&self) -> Result<User, BiolabError> {
        let resp: serde_json::Value = self.http.get("/users/me").await?;
        extract_object(resp)
    }

    pub async fn update_me(&self, data: &serde_json::Value) -> Result<User, BiolabError> {
        let resp: serde_json::Value = self.http.patch("/users/me", data).await?;
        extract_object(resp)
    }

    pub async fn change_password(
        &self,
        current: &str,
        new: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        self.http
            .patch(
                "/users/me/password",
                &serde_json::json!({
                    "current_password": current,
                    "new_password": new,
                }),
            )
            .await
    }
}
