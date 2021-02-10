use super::*;

// GET /serve/download/me/song?url&sid
pub async fn get_download_list(
    hostname: String,
    requests: data_access::DLRequest,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let checksums = conn.get_purchase_dl(STATIC_USER_ID, requests, hostname);
    let result = ResponseContainer {
        success: true,
        value: checksums,
        error_code: 0,
    };
    respond(result, warp::http::StatusCode::OK)
}
