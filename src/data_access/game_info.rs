use super::*;

#[derive(Serialize)]
struct LevelStep {
    level: isize,
    level_exp: isize,
}

#[derive(Serialize)]
pub struct GameInfo {
    curr_ts: i64,
    max_stamina: i8,
    stamina_recover_tick: isize,
    core_exp: isize,
    level_steps: Vec<LevelStep>,
    world_ranking_enabled: bool,
    is_byd_chapter_unlocked: bool,
}

impl GameInfo {
    pub fn new(conn: &DBAccessManager) -> Result<Self, rusqlite::Error> {
        let mut stmt = conn.connection.prepare(sql_stmt::LEVEL_STEP).unwrap();
        let mut level_steps = Vec::new();
        let steps = stmt
            .query_map(params![], |row| Ok(LevelStep {
                level: row.get(0)?,
                level_exp: row.get(1)?,
            }))
            .unwrap();
        for step in steps {
            let step = step.unwrap();
            level_steps.push(step);
        }

        let (mut curr_ts, mut max_stamina, mut stamina_recover_tick) = (0, 0, 0);
        let (mut core_exp, mut world_ranking_enabled, mut is_byd_chapter_unlocked) =
            (250, false, false);
        let mut stmt = conn.connection.prepare(sql_stmt::GAME_INFO).unwrap();
        stmt.query_row(params![], |row| {
            curr_ts = row.get(0)?;
            max_stamina = row.get(1)?;
            stamina_recover_tick = row.get(2)?;
            core_exp = row.get(3)?;
            world_ranking_enabled = row.get::<usize, String>(4)? == "t";
            is_byd_chapter_unlocked = row.get::<usize, String>(5)? == "t";
            Ok(())
        })
        .unwrap();
        Ok(GameInfo {
            curr_ts,
            max_stamina,
            stamina_recover_tick,
            core_exp,
            level_steps,
            world_ranking_enabled,
            is_byd_chapter_unlocked,
        })
    }
}
