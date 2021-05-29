use super::*;
use data_access::save::BackupData;

// POST /user/me/save
pub async fn upload_backup_data(
    mut data: BackupData,
    user_id: isize,
    mut conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    // println!("{}", serde_json::to_string(&data).unwrap());
    data.update_score_on_cloud(&mut conn, user_id).unwrap();
    data.insert_game_progress(&conn, user_id).unwrap();
    let mut result = HashMap::new();
    result.insert("user_id", user_id);
    respond_ok(ResponseContainer {
        success: true,
        value: result,
        error_code: 0,
        error_msg: String::new(),
    })
}

// GET /user/me/save
pub async fn download_backup_data(user_id: isize, conn: DBAccessManager) -> ZrcSVResult<impl warp::Reply> {
    let mut data = BackupData::new_with_id(user_id);
    match data.get_game_progress(&conn, user_id) {
        false => Ok(warp::reply::with_status(
            r#"{"success":false,"error_code":402}"#.to_string(),
            warp::http::StatusCode::NOT_FOUND,
        )),
        true => {
            data.get_score_data(&conn, user_id);
            let container = ResponseContainer {
                success: true,
                value: data,
                error_code: 0,
                error_msg: String::new(),
            };
            Ok(warp::reply::with_status(
                serde_json::to_string(&container).unwrap(),
                warp::http::StatusCode::OK,
            ))
        }
    }
}
