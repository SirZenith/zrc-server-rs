use super::*;

#[derive(Serialize)]
struct MapReward {
    items: Vec<RewardItem>,
    position: isize,
}

#[derive(Serialize)]
struct RewardItem {
    #[serde(rename = "type", skip_serializing_if = "String::is_empty")]
    item_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(skip_serializing_if = "is_zero")]
    amount: i32,
}

#[derive(Serialize)]
struct MapInfo {
    affinity_multiplier: Vec<f64>,
    available_from: i64,
    available_to: i64,
    beyond_health: isize,
    character_affinity: Vec<i8>,
    chapter: isize,
    coordinate: String,
    curr_capture: isize,
    curr_position: isize,
    custom_bg: String,
    is_beyond: bool,
    is_legacy: bool,
    is_locked: bool,
    is_repeatable: bool,
    map_id: String,
    require_id: String,
    require_type: String,
    require_value: isize,
    stamina_cost: isize,
    step_count: isize,
    rewards: Vec<MapReward>,
}

impl MapInfo {
    fn get_map_affinity(&mut self, conn: &DBAccessManager) -> Result<(), rusqlite::Error> {
        let mut characters = Vec::new();
        let mut multiplier = Vec::new();
        let mut stmt = conn.connection.prepare(sql_stmt::MAP_AFFINITY).unwrap();
        let infoes = stmt
            .query_map(&[&self.map_id], |row| {
                Ok((row.get("part_id")?, row.get("multiplier")?))
            })
            .unwrap();

        for info in infoes {
            let info = info.unwrap();
            characters.push(info.0);
            multiplier.push(info.1);
        }

        self.character_affinity = characters;
        self.affinity_multiplier = multiplier;
        Ok(())
    }

    fn get_rewards(&mut self, conn: &DBAccessManager) -> Result<(), rusqlite::Error> {
        let mut stmt = conn.connection.prepare(sql_stmt::MAP_REWARD).unwrap();
        let rewards = stmt
            .query_map(params![self.map_id], |row| {
                Ok((
                    row.get("position")?,
                    RewardItem {
                        id: row.get("reward_id")?,
                        item_type: row.get("item_type")?,
                        amount: row.get("amount")?,
                    },
                ))
            })
            .unwrap();
        for item in rewards {
            let item = item.unwrap();
            self.rewards.push(MapReward {
                items: vec![item.1],
                position: item.0,
            });
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct MapInfoList {
    user_id: isize,
    current_map: String,
    maps: Vec<MapInfo>,
}

impl MapInfoList {
    pub fn new(conn: &DBAccessManager, user_id: isize) -> Result<Self, rusqlite::Error> {
        let mut info_list = MapInfoList {
            user_id: user_id,
            current_map: String::new(),
            maps: Vec::new(),
        };

        let mut stmt = conn.connection.prepare(sql_stmt::MAP_INFO).unwrap();
        let map_infoes = stmt
            .query_map(&[&user_id], |row| {
                let map_id = row.get("map_id")?;
                Ok(MapInfo {
                    available_from: row.get("available_from")?,
                    available_to: row.get("available_to")?,
                    beyond_health: row.get("beyond_health")?,
                    chapter: row.get("chapter")?,
                    coordinate: row.get("coordinate")?,
                    custom_bg: row.get("custom_bg")?,
                    is_beyond: row.get::<&str, String>("is_beyond")? == "t",
                    is_legacy: row.get::<&str, String>("is_legacy")? == "t",
                    is_repeatable: row.get::<&str, String>("is_repeatable")? == "t",
                    map_id: map_id,
                    require_id: row.get("require_id")?,
                    require_type: row.get("require_type")?,
                    require_value: row.get("require_value")?,
                    stamina_cost: row.get("stamina_cost")?,
                    step_count: row.get("step_count")?,
                    curr_capture: row.get("curr_capture")?,
                    curr_position: row.get("curr_position")?,
                    is_locked: row.get::<&str, String>("is_locked")? == "t",
                    affinity_multiplier: Vec::new(),
                    character_affinity: Vec::new(),
                    rewards: Vec::new(),
                })
            })
            .unwrap();
        for map_info in map_infoes {
            let mut map_info = map_info.unwrap();
            map_info.get_map_affinity(conn)?;
            map_info.get_rewards(conn)?;
            info_list.maps.push(map_info);
        }

        Ok(info_list)
    }
}
