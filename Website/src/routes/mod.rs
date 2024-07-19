pub mod package;
pub mod middleware;

use crate::templates::GetIndexResponse;

pub async fn get_index() -> GetIndexResponse {
    GetIndexResponse {}
}
