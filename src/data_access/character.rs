use super::*;

const VOICE: [isize; 7] = [0, 1, 2, 3, 100, 1000, 1001];

#[derive(Serialize)]
struct CharacterStats {
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
    character_id: i8,
    #[serde(skip_serializing_if = "is_zero")]
    prog_tempest: f64,
}

#[derive(Serialize)]
pub struct CharacterStatses(Vec<CharacterStats>);

impl CharacterStatses {
    pub fn new(conn: &DBAccessManager, user_id: isize, part_id: Option<isize>) -> Result<CharacterStatses, rusqlite::Error> {
        let cond = match part_id {
            Some(id) => format!("{}{}", sql_stmt::COND_SINGLE_CHAR_STATS, id),
            None => sql_stmt::COND_ALL_CHAR_STATS.to_string(),
        };
        let mut stmt = conn
            .connection
            .prepare(&format!("{}{};", sql_stmt::CHAR_STATS, cond))
            .unwrap();
        let statses = stmt
            .query_map(params![user_id], |row| {
                Ok(CharacterStats {
                    voice: if row.get::<&str, isize>("have_voice").unwrap() >= 0 {
                        VOICE.iter().map(|x| *x).collect()
                    } else {
                        Vec::new()
                    },
                    is_uncapped_override: row.get::<&str, String>("uncapped_override")? == "t",
                    is_uncapped: row.get::<&str, String>("uncapped")? == "t",
                    uncap_cores: Vec::new(),
                    char_type: row.get("char_type")?,
                    skill_id_uncap: row.get("uncap_skill")?,
                    skill_requires_uncap: row.get::<&str, String>("skill_requires_uncap")? == "t",
                    skill_unlock_level: row.get("skill_unlock_level")?,
                    skill_id: row.get("skill_id")?,
                    overdrive: row.get("overdrive")?,
                    prog: row.get("prog")?,
                    frag: row.get("frag")?,
                    level_exp: row.get("level_exp")?,
                    exp: row.get("exp_val")?,
                    level: row.get("lv")?,
                    name: row.get("part_name")?,
                    character_id: row.get("part_id")?,
                    prog_tempest: row.get("prog_tempest")?,
                })
            })
            .unwrap();
        Ok(CharacterStatses(
            statses.into_iter().map(|s| s.unwrap()).collect(),
        ))
    }

    pub fn list_char_ids(&self) -> Vec<i8> {
        self.0.iter().map(|x| x.character_id).collect()
    }
}
