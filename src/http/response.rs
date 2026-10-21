use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[allow(dead_code)]
impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn ok(data: T) -> (StatusCode, Json<Self>) {
        (StatusCode::OK, Json(Self { data }))
    }

    pub fn created(data: T) -> (StatusCode, Json<Self>) {
        (StatusCode::CREATED, Json(Self { data }))
    }

    pub fn json(data: T) -> Json<Self> {
        Json(Self { data })
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct PageMeta {
    pub total: i64,
}

#[allow(dead_code)]
impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64) -> Self {
        Self {
            data,
            meta: PageMeta { total },
        }
    }

    pub fn ok(data: Vec<T>, total: i64) -> (StatusCode, Json<Self>) {
        (StatusCode::OK, Json(Self::new(data, total)))
    }

    pub fn json(data: Vec<T>, total: i64) -> Json<Self> {
        Json(Self::new(data, total))
    }
}
