use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::Bucket;

#[derive(Debug, Deserialize, Validate)]
pub struct ListBucketsRequest {
    #[validate(range(min = 900, max = 43200))]
    pub duration_seconds: Option<u32>,
}

#[derive(Serialize)]
pub struct BucketResponse {
    pub name: String,
}

impl From<Bucket> for BucketResponse {
    fn from(bucket: Bucket) -> Self {
        Self { name: bucket.name }
    }
}

#[derive(Serialize)]
pub struct ListBucketsResponse {
    pub buckets: Vec<BucketResponse>,
}

impl ListBucketsResponse {
    pub fn from_buckets(buckets: Vec<Bucket>) -> Self {
        Self {
            buckets: buckets.into_iter().map(Into::into).collect(),
        }
    }
}
