use crate::api_response::{envelope_data, extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::{path_segment_encode, url_encode};
use crate::types::{
    ManifestImportTask, SeedIntakeBatch, SeedIntakeTask, SeedLot, SeedMovement, SeedPlacement,
    SeedReservation,
};
use std::path::{Path, PathBuf};

impl ScientexClient {
    pub async fn list_seed_object_types(
        &self,
        slug: &str,
    ) -> Result<PaginatedList<serde_json::Value>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_object_types_path(slug)).await?;
        extract_paginated(resp)
    }

    pub async fn create_seed_object_type(
        &self,
        slug: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self.http.post(&seed_object_types_path(slug), data).await?;
        extract_object(resp)
    }

    pub async fn get_seed_object_type(
        &self,
        slug: &str,
        config_id: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&seed_object_type_path(slug, config_id))
            .await?;
        extract_object(resp)
    }

    pub async fn update_seed_object_type(
        &self,
        slug: &str,
        config_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .patch(&seed_object_type_path(slug, config_id), data)
            .await?;
        extract_object(resp)
    }

    pub async fn list_seed_intake_batches(
        &self,
        slug: &str,
    ) -> Result<PaginatedList<SeedIntakeBatch>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_batches_path(slug)).await?;
        extract_paginated(resp)
    }

    pub async fn create_seed_intake_batch(
        &self,
        slug: &str,
        data: &serde_json::Value,
    ) -> Result<SeedIntakeBatch, ScientexError> {
        let resp: serde_json::Value = self.http.post(&seed_batches_path(slug), data).await?;
        extract_object(resp)
    }

    pub async fn get_seed_intake_batch(
        &self,
        slug: &str,
        batch_id: &str,
    ) -> Result<SeedIntakeBatch, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_batch_path(slug, batch_id)).await?;
        extract_object(resp)
    }

    pub async fn download_seed_manifest_template(
        &self,
        slug: &str,
        batch_id: &str,
        output: Option<&Path>,
        force: bool,
    ) -> Result<(PathBuf, String), ScientexError> {
        let downloaded = self
            .http
            .download_to_file(&seed_manifest_template_path(slug, batch_id), output, force)
            .await?;
        Ok((downloaded.path, downloaded.server_filename))
    }

    pub async fn create_seed_manifest_import_task(
        &self,
        slug: &str,
        batch_id: &str,
        file_path: &str,
    ) -> Result<ManifestImportTask, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post_multipart(
                &seed_manifest_import_task_path(slug, batch_id),
                &[],
                &[("file", file_path)],
                &[],
            )
            .await?;
        extract_object(resp)
    }

    pub async fn create_seed_intake_task(
        &self,
        slug: &str,
        batch_id: &str,
        record_ids: &[String],
    ) -> Result<SeedIntakeTask, ScientexError> {
        let body = if record_ids.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({ "record_ids": record_ids })
        };
        let resp: serde_json::Value = self
            .http
            .post(&seed_intake_task_path(slug, batch_id), &body)
            .await?;
        extract_object(resp)
    }

    pub async fn list_seed_intake_records(
        &self,
        slug: &str,
        batch_id: Option<&str>,
        status: Option<&str>,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<serde_json::Value>, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&seed_records_path(slug, batch_id, status, skip, limit))
            .await?;
        extract_paginated(resp)
    }

    pub async fn list_public_seed_intake_records(
        &self,
        slug: &str,
        batch_id: Option<&str>,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<serde_json::Value>, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&seed_public_records_path(slug, batch_id, skip, limit))
            .await?;
        extract_paginated(resp)
    }

    pub async fn get_seed_intake_record(
        &self,
        slug: &str,
        record_id: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_record_path(slug, record_id)).await?;
        extract_object(resp)
    }

    pub async fn update_seed_intake_record(
        &self,
        slug: &str,
        record_id: &str,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .patch(&seed_record_path(slug, record_id), data)
            .await?;
        extract_object(resp)
    }

    pub async fn complete_seed_intake_record(
        &self,
        slug: &str,
        record_id: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .post(
                &seed_record_complete_path(slug, record_id),
                &serde_json::json!({}),
            )
            .await?;
        Ok(envelope_data(resp))
    }

    pub async fn list_seed_stocks(
        &self,
        slug: &str,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<serde_json::Value>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_stocks_path(slug, skip, limit)).await?;
        extract_paginated(resp)
    }

    pub async fn get_seed_stock(
        &self,
        slug: &str,
        stock_id: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_stock_path(slug, stock_id)).await?;
        extract_object(resp)
    }

    pub async fn list_seed_lots(
        &self,
        slug: &str,
        seed_type_code: Option<&str>,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<SeedLot>, ScientexError> {
        let response: serde_json::Value = self
            .http
            .get(&seed_lots_path(slug, seed_type_code, skip, limit))
            .await?;
        extract_paginated(response)
    }

    pub async fn get_seed_lot(&self, slug: &str, lot_id: &str) -> Result<SeedLot, ScientexError> {
        let response: serde_json::Value = self.http.get(&seed_lot_path(slug, lot_id)).await?;
        extract_object(response)
    }

    pub async fn list_seed_lot_movements(
        &self,
        slug: &str,
        lot_id: &str,
    ) -> Result<PaginatedList<SeedMovement>, ScientexError> {
        let response: serde_json::Value = self
            .http
            .get(&seed_lot_movements_path(slug, lot_id))
            .await?;
        extract_paginated(response)
    }

    pub async fn list_seed_lot_reservations(
        &self,
        slug: &str,
        lot_id: &str,
    ) -> Result<PaginatedList<SeedReservation>, ScientexError> {
        let response: serde_json::Value = self
            .http
            .get(&seed_lot_reservations_path(slug, lot_id))
            .await?;
        extract_paginated(response)
    }

    pub async fn reserve_seed_lot(
        &self,
        slug: &str,
        lot_id: &str,
        data: &serde_json::Value,
    ) -> Result<SeedReservation, ScientexError> {
        let response: serde_json::Value = self
            .http
            .post(&seed_lot_reservations_path(slug, lot_id), data)
            .await?;
        extract_object(response)
    }

    pub async fn release_seed_reservation(
        &self,
        slug: &str,
        reservation_id: &str,
    ) -> Result<SeedReservation, ScientexError> {
        let response: serde_json::Value = self
            .http
            .post(
                &seed_reservation_release_path(slug, reservation_id),
                &serde_json::json!({}),
            )
            .await?;
        extract_object(response)
    }

    pub async fn checkout_seed_lot(
        &self,
        slug: &str,
        lot_id: &str,
        data: &serde_json::Value,
    ) -> Result<SeedMovement, ScientexError> {
        let response: serde_json::Value = self
            .http
            .post(&seed_lot_checkout_path(slug, lot_id), data)
            .await?;
        extract_object(response)
    }

    pub async fn transfer_seed_lot(
        &self,
        slug: &str,
        lot_id: &str,
        data: &serde_json::Value,
    ) -> Result<SeedPlacement, ScientexError> {
        let response: serde_json::Value = self
            .http
            .post(&seed_lot_transfer_path(slug, lot_id), data)
            .await?;
        extract_object(response)
    }

    pub async fn adjust_seed_lot(
        &self,
        slug: &str,
        lot_id: &str,
        data: &serde_json::Value,
    ) -> Result<SeedMovement, ScientexError> {
        let response: serde_json::Value = self
            .http
            .post(&seed_lot_adjustments_path(slug, lot_id), data)
            .await?;
        extract_object(response)
    }

    /// Return metadata for all available seed intake record fields.
    ///
    /// The response is a `SeedFieldCatalogResponse` envelope:
    /// `{ "data": [SeedFieldMeta, ...] }`.
    pub async fn get_seed_field_catalog(
        &self,
        slug: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self.http.get(&seed_field_catalog_path(slug)).await?;
        Ok(envelope_data(resp))
    }
}

fn seed_base_path(slug: &str) -> String {
    format!("/project/{}/seed", path_segment_encode(slug))
}

fn seed_object_types_path(slug: &str) -> String {
    format!("{}/object-types", seed_base_path(slug))
}

fn seed_object_type_path(slug: &str, config_id: &str) -> String {
    format!(
        "{}/{}",
        seed_object_types_path(slug),
        path_segment_encode(config_id)
    )
}

fn seed_batches_path(slug: &str) -> String {
    format!("{}/intake-batches", seed_base_path(slug))
}

fn seed_batch_path(slug: &str, batch_id: &str) -> String {
    format!(
        "{}/{}",
        seed_batches_path(slug),
        path_segment_encode(batch_id)
    )
}

fn seed_manifest_import_task_path(slug: &str, batch_id: &str) -> String {
    format!("{}/manifest-import-task", seed_batch_path(slug, batch_id))
}

fn seed_manifest_template_path(slug: &str, batch_id: &str) -> String {
    format!("{}/manifest-template", seed_batch_path(slug, batch_id))
}

fn seed_intake_task_path(slug: &str, batch_id: &str) -> String {
    format!("{}/intake-task", seed_batch_path(slug, batch_id))
}

fn seed_records_path(
    slug: &str,
    batch_id: Option<&str>,
    status: Option<&str>,
    skip: u32,
    limit: u32,
) -> String {
    let mut params = vec![format!("skip={skip}"), format!("limit={limit}")];
    push_query_param(&mut params, "batch_id", batch_id);
    push_query_param(&mut params, "status", status);
    format!(
        "{}/intake-records?{}",
        seed_base_path(slug),
        params.join("&")
    )
}

fn seed_public_records_path(slug: &str, batch_id: Option<&str>, skip: u32, limit: u32) -> String {
    let mut params = vec![format!("skip={skip}"), format!("limit={limit}")];
    push_query_param(&mut params, "batch_id", batch_id);
    format!(
        "{}/intake-records/public?{}",
        seed_base_path(slug),
        params.join("&")
    )
}

fn seed_record_path(slug: &str, record_id: &str) -> String {
    format!(
        "{}/intake-records/{}",
        seed_base_path(slug),
        path_segment_encode(record_id)
    )
}

fn seed_record_complete_path(slug: &str, record_id: &str) -> String {
    format!("{}/complete", seed_record_path(slug, record_id))
}

fn seed_stocks_path(slug: &str, skip: u32, limit: u32) -> String {
    format!("{}/stocks?skip={skip}&limit={limit}", seed_base_path(slug))
}

fn seed_stock_path(slug: &str, stock_id: &str) -> String {
    format!(
        "{}/stocks/{}",
        seed_base_path(slug),
        path_segment_encode(stock_id)
    )
}

fn seed_lots_path(slug: &str, seed_type_code: Option<&str>, skip: u32, limit: u32) -> String {
    let mut params = vec![format!("skip={skip}"), format!("limit={limit}")];
    push_query_param(&mut params, "seed_type_code", seed_type_code);
    format!("{}/lots?{}", seed_base_path(slug), params.join("&"))
}

fn seed_lot_path(slug: &str, lot_id: &str) -> String {
    format!(
        "{}/lots/{}",
        seed_base_path(slug),
        path_segment_encode(lot_id)
    )
}

fn seed_lot_movements_path(slug: &str, lot_id: &str) -> String {
    format!("{}/movements", seed_lot_path(slug, lot_id))
}

fn seed_lot_reservations_path(slug: &str, lot_id: &str) -> String {
    format!("{}/reservations", seed_lot_path(slug, lot_id))
}

fn seed_reservation_release_path(slug: &str, reservation_id: &str) -> String {
    format!(
        "{}/reservations/{}/release",
        seed_base_path(slug),
        path_segment_encode(reservation_id)
    )
}

fn seed_lot_checkout_path(slug: &str, lot_id: &str) -> String {
    format!("{}/checkout", seed_lot_path(slug, lot_id))
}

fn seed_lot_transfer_path(slug: &str, lot_id: &str) -> String {
    format!("{}/transfer", seed_lot_path(slug, lot_id))
}

fn seed_lot_adjustments_path(slug: &str, lot_id: &str) -> String {
    format!("{}/adjustments", seed_lot_path(slug, lot_id))
}

fn seed_field_catalog_path(slug: &str) -> String {
    format!("{}/field-catalog", seed_base_path(slug))
}

fn push_query_param(params: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        params.push(format!("{}={}", key, url_encode(value)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_seed_object_type_paths() {
        assert_eq!(
            seed_object_types_path("ta shan"),
            "/project/ta%20shan/seed/object-types"
        );
        assert_eq!(
            seed_object_type_path("ta shan", "cfg 1/a"),
            "/project/ta%20shan/seed/object-types/cfg%201%2Fa"
        );
    }

    #[test]
    fn builds_seed_batch_paths() {
        assert_eq!(
            seed_batches_path("tashan"),
            "/project/tashan/seed/intake-batches"
        );
        assert_eq!(
            seed_manifest_import_task_path("tashan", "batch 1"),
            "/project/tashan/seed/intake-batches/batch%201/manifest-import-task"
        );
        assert_eq!(
            seed_manifest_template_path("tashan", "batch 1"),
            "/project/tashan/seed/intake-batches/batch%201/manifest-template"
        );
        assert_eq!(
            seed_intake_task_path("tashan", "batch 1"),
            "/project/tashan/seed/intake-batches/batch%201/intake-task"
        );
    }

    #[test]
    fn builds_seed_record_paths_with_queries() {
        assert_eq!(
            seed_records_path(
                "tashan",
                Some("batch 1"),
                Some("waiting physical"),
                10,
                20
            ),
            "/project/tashan/seed/intake-records?skip=10&limit=20&batch_id=batch+1&status=waiting+physical"
        );
        assert_eq!(
            seed_public_records_path("tashan", Some("batch 1"), 0, 5),
            "/project/tashan/seed/intake-records/public?skip=0&limit=5&batch_id=batch+1"
        );
        assert_eq!(
            seed_record_complete_path("tashan", "rec 1/a"),
            "/project/tashan/seed/intake-records/rec%201%2Fa/complete"
        );
    }

    #[test]
    fn builds_seed_stock_paths() {
        assert_eq!(
            seed_stocks_path("tashan", 0, 100),
            "/project/tashan/seed/stocks?skip=0&limit=100"
        );
        assert_eq!(
            seed_stock_path("ta shan", "stock 1/a"),
            "/project/ta%20shan/seed/stocks/stock%201%2Fa"
        );
    }

    #[test]
    fn builds_seed_lot_paths() {
        assert_eq!(
            seed_lots_path("ta shan", Some("GM 1"), 10, 20),
            "/project/ta%20shan/seed/lots?skip=10&limit=20&seed_type_code=GM+1"
        );
        assert_eq!(
            seed_lot_path("tashan", "lot 1/a"),
            "/project/tashan/seed/lots/lot%201%2Fa"
        );
        assert_eq!(
            seed_lot_movements_path("tashan", "lot-1"),
            "/project/tashan/seed/lots/lot-1/movements"
        );
        assert_eq!(
            seed_lot_reservations_path("tashan", "lot-1"),
            "/project/tashan/seed/lots/lot-1/reservations"
        );
        assert_eq!(
            seed_reservation_release_path("tashan", "reservation/1"),
            "/project/tashan/seed/reservations/reservation%2F1/release"
        );
        assert_eq!(
            seed_lot_checkout_path("tashan", "lot-1"),
            "/project/tashan/seed/lots/lot-1/checkout"
        );
        assert_eq!(
            seed_lot_transfer_path("tashan", "lot-1"),
            "/project/tashan/seed/lots/lot-1/transfer"
        );
        assert_eq!(
            seed_lot_adjustments_path("tashan", "lot-1"),
            "/project/tashan/seed/lots/lot-1/adjustments"
        );
    }

    #[test]
    fn builds_seed_field_catalog_path() {
        assert_eq!(
            seed_field_catalog_path("tashan"),
            "/project/tashan/seed/field-catalog"
        );
        assert_eq!(
            seed_field_catalog_path("ta shan"),
            "/project/ta%20shan/seed/field-catalog"
        );
    }
}
