use super::*;

#[derive(Deserialize)]
pub struct ChangeToCharacter {
    character: isize,
    skill_sealed: bool,
}

// POST /user/me/characters
pub async fn change_character(
    change_to: ChangeToCharacter,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    conn.change_character(STATIC_USER_ID, change_to.character, change_to.skill_sealed)
        .unwrap();
    let result = format!(
        r#"{{"success": true,"value": {{"user_id": {}, "character": {}}}}}"#,
        STATIC_USER_ID, change_to.character
    );
    Ok(warp::reply::with_status(result, warp::http::StatusCode::OK))
}

#[derive(Serialize)]
pub struct ToggleResult {
    user_id: isize,
    character: data_access::CharacterStatses,
}

// POST /user/me/characters/<part_id>/toggle_uncap
pub async fn toggle_uncap(
    part_id: isize,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_id = STATIC_USER_ID;
    let stats = conn.toggle_uncap(user_id, part_id).unwrap();
    let json = warp::reply::json(&ResponseContainer {
        success: true,
        value: ToggleResult {
            user_id: user_id,
            character: stats,
        },
        error_code: 0,
    });
    Ok(json)
}
