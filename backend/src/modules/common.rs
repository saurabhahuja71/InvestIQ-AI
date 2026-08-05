use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<()>,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: None,
            error: None,
        }
    }

    pub fn ok_with_meta(data: T, meta: Meta) -> Self {
        Self {
            success: true,
            data,
            meta: Some(meta),
            error: None,
        }
    }
}
