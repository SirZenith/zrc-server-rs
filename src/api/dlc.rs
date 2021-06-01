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


// POST /purchase/me/pack
pub async fn purcahse_item(
    form: HashMap<String, String>,
    user_id: isize,
    mut conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    let result = if let Ok(pack_id) = get_from_form(&form, "pack_id") {
        conn.purchase_item(user_id, pack_id, ItemType::Pack)
    } else if let Ok(single_id) = get_from_form(&form, "single_id") {
        conn.purchase_item(user_id, single_id, ItemType::Single)
    } else {
        return Err(warp::reject::custom(ZrcSVError::IncompleteForm("pack_id or single_id".to_string())))
    };
    match result {
        Ok(info) => respond_ok(ResponseContainer {
            success: true,
            value: info,
            error_code: 0,
            error_msg: String::new(),
        }),
        Err(e) => Err(warp::reject::custom(ZrcSVError::DBError(e))),
    }
}
