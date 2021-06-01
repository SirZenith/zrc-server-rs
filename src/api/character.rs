use super::*;

#[derive(Deserialize)]
pub struct ChangeToCharacter {
    character: isize,
    skill_sealed: bool,
}

// POST /user/me/characters
pub async fn change_character(
    change_to: ChangeToCharacter,
    user_id: isize,
    conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    conn.change_character(user_id, change_to.character, change_to.skill_sealed)
        .map_err(|e| warp::reject::custom(ZrcSVError::DBError(e)))?;
    let mut result = HashMap::new();
    result.insert("user_id".to_string(), user_id);
    result.insert("character".to_string(), change_to.character);
    respond_ok(ResponseContainer {
        success: true,
        value: result,
        error_code: 0,
        error_msg: String::new(),
    })
}

#[derive(Serialize)]
pub struct ToggleResult {
    user_id: isize,
    character: data_access::CharacterStatses,
}

// POST /user/me/characters/<part_id>/toggle_uncap
pub async fn toggle_uncap(
    part_id: isize,
    user_id: isize,
    conn: DBAccessManager,
) -> ZrcSVResult<impl warp::Reply> {
    let stats = conn.toggle_uncap(user_id, part_id).unwrap();
    let json = warp::reply::json(&ResponseContainer {
        success: true,
        value: ToggleResult {
            user_id: user_id,
            character: stats,
        },
        error_code: 0,
        error_msg: String::new(),
    });
    Ok(json)
}
