use super::*;

// GET /serve/download/me/song?url&sid
pub async fn get_download_list(
    // (hostname, prefix_static_file, songs_direname)
    info_group: (String, String, String),
    requests: data_access::DLRequest,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let checksums = conn.get_purchase_dl(
        STATIC_USER_ID,
        requests,
        &info_group.0,
        &info_group.1,
        &info_group.2,
    );
    let result = ResponseContainer {
        success: true,
        value: checksums,
        error_code: 0,
    };
    respond(result, warp::http::StatusCode::OK)
}
