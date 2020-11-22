use super::*;

const VOICE: [isize; 7] = [0, 1, 2, 3, 100, 1000, 1001];

#[derive(Serialize)]
pub struct CharacterStats {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    voice: Vec<isize>,
    is_uncapped_override: bool,
    is_uncapped: bool,
    uncap_cores: Vec<String>,
    char_type: i8,
    skill_id_uncap: Option<String>,
    skill_requires_uncap: bool,
    skill_unlock_level: i8,
    skill_id: Option<String>,
    overdrive: f64,
    prog: f64,
    frag: f64,
    level_exp: isize,
    exp: f64,
    level: i8,
    name: String,
    pub character_id: i8,
    #[serde(skip_serializing_if = "is_zero")]
    prog_tempest: f64,
}

#[derive(Serialize)]
pub struct CharacterStatses(Vec<CharacterStats>);

impl CharacterStatses {
    pub fn new(conn: &rusqlite::Connection, part_id: Option<isize>) -> Self {
        let cond = match part_id {
            Some(id) => format!("{}{}", sql_stmt::COND_SINGLE_CHAR_STATS, id),
            None => sql_stmt::COND_ALL_CHAR_STATS.to_string(),
        };
        let mut stmt = conn
            .prepare(&format!("{}{};", sql_stmt::CHAR_STATS, cond))
            .unwrap();
        let statses = stmt
            .query_map(&[&STATIC_USER_ID], |row| CharacterStats {
                voice: if row.get::<&str, isize>("have_voice") >= 0 {
                    VOICE.iter().map(|x| *x).collect()
                } else {
                    Vec::new()
                },
                is_uncapped_override: row.get::<&str, String>("uncapped_override") == "t",
                is_uncapped: row.get::<&str, String>("uncapped") == "t",
                uncap_cores: Vec::new(),
                char_type: row.get("char_type"),
                skill_id_uncap: row.get("uncap_skill"),
                skill_requires_uncap: row.get::<&str, String>("skill_requires_uncap") == "t",
                skill_unlock_level: row.get("skill_unlock_level"),
                skill_id: row.get("skill_id"),
                overdrive: row.get("overdrive"),
                prog: row.get("prog"),
                frag: row.get("frag"),
                level_exp: row.get("level_exp"),
                exp: row.get("exp_val"),
                level: row.get("lv"),
                name: row.get("part_name"),
                character_id: row.get("part_id"),
                prog_tempest: row.get("prog_tempest"),
            })
            .unwrap();
        CharacterStatses(statses.into_iter().map(|s| s.unwrap()).collect())
    }

    pub fn list_char_ids(&self) -> Vec<i8> {
        self.0.iter().map(|x| x.character_id).collect()
    }
}

#[derive(FromForm)]
pub struct ChangeToCharacter {
    character: isize,
    skill_sealed: bool,
}

#[post("/", data = "<change_to>")]
pub fn change_character(conn: ZrcDB, change_to: Form<ChangeToCharacter>) -> String {
    let conn: &rusqlite::Connection = &*conn;
    conn.execute(
        sql_stmt::CHANGE_CHARACTER,
        &[
            &change_to.character,
            &change_to.skill_sealed,
            &STATIC_USER_ID,
        ],
    )
    .unwrap();
    format!(
        r#"{{"success": true,"value": {{"user_id": {}, "character": {}}}}}"#,
        STATIC_USER_ID, change_to.character
    )
}

#[derive(Serialize)]
pub struct ToggleResult {
    user_id: isize,
    character: CharacterStatses
}

#[post("/<part_id>/toggle_uncap")]
pub fn toggle_uncap(part_id: isize, conn: ZrcDB) -> Json<ResponseContainer<ToggleResult>> {
    let mut stmt = conn.prepare(sql_stmt::TOGGLE_UNCAP).unwrap();
    stmt.execute(&[&STATIC_USER_ID, &part_id]).unwrap();
    let stats = CharacterStatses::new(&*conn, Some(part_id));
    Json(ResponseContainer {
        success: true,
        value: ToggleResult {user_id: STATIC_USER_ID, character: stats},
        error_code: 0,
    })
}
