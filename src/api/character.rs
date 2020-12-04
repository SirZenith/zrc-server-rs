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
    respond(result, warp::http::StatusCode::OK)
}

#[derive(Serialize)]
pub struct ToggleResult {
    user_id: isize,
    character: data_access::character::CharacterStatses,
}

// POST /user/me/characters/<part_id>/toggle_uncap
pub async fn toggle_uncap(
    part_id: isize,
    conn: DBAccessManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let stats = conn.get_char_statuses(STATIC_USER_ID, Some(part_id)).unwrap();
    let json = warp::reply::json(&ResponseContainer {
        success: true,
        value: ToggleResult {
            user_id: STATIC_USER_ID,
            character: stats,
        },
        error_code: 0,
    });
    Ok(json)
}
