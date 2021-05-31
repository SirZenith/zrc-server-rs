use super::*;

// GET /serve/download/me/song?url&sid
pub async fn get_download_list(
    // (hostname, prefix_static_file, songs_direname)
    info_group: (String, String, String),
    requests: data_access::DLRequest,
    user_id: isize,
    conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    let checksums = conn.get_purchase_dl(
        user_id,
        requests,
        &info_group.0,
        &info_group.1,
        &info_group.2,
    ).map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?;
    let result = ResponseContainer {
        success: true,
        value: checksums,
        error_code: 0,
        error_msg: String::new(),
    };
    respond_ok(result)
}
