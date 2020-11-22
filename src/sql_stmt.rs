// character
// ============================================================================
pub const TOGGLE_UNCAP: &str = r#"
    update part_stats
    set is_uncapped_override = case
        when is_uncapped_override = 't' then
            'f'
        else
            't'
        end
    where user_id = ?1 and part_id = ?2;
"#;

pub const CHAR_STATS: &str = r#"
    select
        ifnull(v.part_id, -1) as "have_voice",
        ifnull(is_uncapped_override, '') as "uncapped_override",
        ifnull(is_uncapped, '') as "uncapped",
        p.char_type,
        ifnull(p.skill_id_uncap, '') as "uncap_skill",
        ifnull(p.skill_requires_uncap, '') as "skill_requires_uncap",
        p.skill_unlock_level,
        ifnull(p.skill_id, '') skill_id,
        overdrive,
        prog,
        frag,
        s.exp_val,
        l.exp_val as "level_exp",
        s.lv,
        p.part_name,
        p.part_id,
        prog_tempest
    from
        part_stats s, level_exp l, partner p left outer join part_voice v on p.part_id = v.part_id
    where
        s.user_id = ?1
        and s.lv = l.lv
        and
"#;

pub const COND_SINGLE_CHAR_STATS: &str = r#"s.part_id = p.part_id and p.part_id = "#;
pub const COND_ALL_CHAR_STATS: &str = r#"s.part_id = p.part_id"#;

pub const CHANGE_CHARACTER: &str = r#"
    update player set partner = ?1, is_skill_sealed = ?2 where user_id = ?3
"#;

// download
// ============================================================================
pub const QUERY_DL: &str = r#"
    select
		song.song_id,
		song.checksum as "audio_checksum",
		ifnull(song.remote_dl, '') as "song_dl",
		cast(chart_info.difficulty as text) as "difficulty",
		chart_info.checksum as "chart_checksum",
		ifnull(chart_info.remote_dl, '') as "chart_dl"
	from
		{table_name}, song, chart_info
	where
		pur.user_id = ?1
        and song.song_id = chart_info.song_id
        and {query_condition}
        and (song.remote_dl = 't' or chart_info.remote_dl = 't')
        {song_id_condition}
"#;

// info
// ============================================================================

pub const GAME_INFO: &str = r#"
    select
        cast(strftime('%s', 'now') as decimal) as now,
        max_stamina,
        stamina_recover_tick,
        core_exp,
        ifnull(world_ranking_enabled, '') as "World Ranking Enabled",
        ifnull(is_byd_chapter_unlocked, '') as "Byd Chapter Unlocked"
    from
        game_info
"#;

pub const LEVEL_STEP: &str = r#"
    select lv, exp_val from level_exp
"#;

pub const PACK_INFO: &str = r#"
    select
        pack_name, price, orig_price, discount_from, discount_to
    from
        pack
"#;

pub const PACK_ITEM: &str = r#"
    select 
        item_id, item_type, is_available
    from
        pack_item
    where
        pack_name = ?
"#;

pub const USER_INFO: &str = r#"
    select
        user_name,
        user_code,
        ifnull(display_name, '') as "display_name",
        ticket,
        ifnull(partner, 0) as "partner",
        ifnull(is_locked_name_duplicated, '') as "locked",
        ifnull(is_skill_sealed, '') as "skill_sealed",
        ifnull(curr_map, '') as "curr_map",
        prog_boost, stamina,
        next_fragstam_ts,
        max_stamina_ts,
        ifnull(max_stamina_notification, '') as "stamina_notification",
        ifnull(is_hide_rating, '') as "hide_rating", 
        ifnull(favorite_partner, 0) as "fav_partner",
        recent_score_date,
        max_friend,
        rating,
        join_date,
        ifnull(g.is_aprilfools, '') as "is_aprilfools"
    from
        player, game_info as g
    where
        user_id = ?1
"#;

pub const USER_MOST_RECENT_SCORE: &str = r#"
    select
		s.song_id, s.difficulty, s.score,
		s.shiny_pure, s.pure, s.far, s.lost,
		s.health, ifnull(s.modifier, 0) modifier,
        s.played_date,
        s.clear_type, s2.clear_type "best_clear_type"
	from
		score s, best_score b, score s2
	where
		s.user_id = ?1
		and s.played_date = (select max(played_date) from score)
		and s.song_id = s2.song_id
		and b.user_id = ?1
		and b.played_date = s2.played_date
"#;

// score
// ============================================================================
pub const BASE_RATING: &str = r#"
    select rating from chart_info where song_id = ?1 and difficulty = ?2
"#;

pub const INSERT_SCORE: &str = r#"
    insert into score (
        user_id, played_date, song_id, difficulty, score,
        shiny_pure, pure, far, lost, rating,
        health, clear_type
    ) values(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
"#;

pub const QUERY_BEST_SCORE: &str = r#"
    select
        s.score, s.played_date
    from
        best_score b, score s
    where
        b.user_id = ?1
        and b.user_id = s.user_id
        and b.played_date = s.played_date
        and s.song_id = ?2
        and s.difficulty = ?3;
"#;

pub const INSERT_BEST_SCORE: &str = r#"
    insert into best_score(user_id, played_date) values(?1, ?2)
"#;

pub const UPDATE_BEST_SCORE: &str = r#"
    update best_score set played_date = ?1 where played_date = ?2
"#;

pub const QUERY_RECENT_SCORE: &str = r#"
    select
        s.played_date,
        s.rating,
        (s.song_id || s.difficulty) iden, 
        r.is_recent_10
    from
        recent_score r, score s
    where
        r.user_id = ?1
        and r.user_id = s.user_id
        and r.played_date = s.played_date
    order by
        rating desc;
"#;

pub const INSERT_RECENT_SCORE: &str = r#"
    insert into recent_score(user_id, played_date, is_recent_10) values(?1, ?2, ?3)
"#;

pub const REPLACE_RECENT_SCORE: &str = r#"
    update recent_score set played_date = ?1, is_recent_10 = ?2 where user_id = ?3 and played_date = ?4
"#;

pub const COMPUTE_RATING: &str = r#"
    with
        recent as (
            select rating
            from  recent_score r, score s
            where r.user_id = ?1
                and r.is_recent_10 = 't'
                and r.user_id = s.user_id
                and r.played_date = s.played_date
        )
    select
        round( (?2 + r10) / (?3 + r10_count) * 100 )
    from (
        select ifnull(sum(rating), 0) r10, ifnull(count(rating), 0) r10_count from recent
    )
"#;

pub const QUERY_BEST_RATING: &str = r#"
    select rating
    from  best_score b, score s
    where b.user_id = ?1
        and b.user_id = s.user_id
        and b.played_date = s.played_date
    order by rating desc;
"#;

pub const UPDATE_RATING: &str = r#"
    update player set rating = ?1 where user_id = ?2
"#;

pub const MAP_INFO: &str = r#"
    select
		available_from,
		available_to,
		beyond_health,
		chapter,
		coordinate,
		ifnull(custom_bg, '') custom_bg,
		ifnull(is_beyond, '') is_beyond,
		ifnull(is_legacy, '') is_legacy,
		ifnull(is_repeatable, '') is_repeatable,
		world_map.map_id,
		ifnull(require_id, '') require_id,
		ifnull(require_type, '') require_type,
		ifnull(require_value, 1) require_value,
		stamina_cost,
		step_count,
		curr_capture,
		curr_position,
		ifnull(is_locked, '') is_locked
	from
		world_map, player_map_prog
	where
		player_map_prog.map_id = world_map.map_id
		and player_map_prog.user_id = ?
"#;

pub const MAP_AFFINITY: &str = r#"
	select part_id, multiplier from map_affinity where map_id = ?
"#;

pub const MAP_REWARD: &str = r#"
    select
        ifnull(reward_id, "") reward_id,
        item_type,
        ifnull(amount, 0) amount,
        position
	from
		map_reward
	where
		map_id = ?
"#;