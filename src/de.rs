use nom::{
    bytes::complete::tag,
    bytes::complete::take,
    combinator::cond,
    multi::count,
    number::complete::{le_f32, le_i32, le_u32, u8},
    IResult,
};

use crate::primitives::{de_string, flag};

const SEPARATOR: &[u8] = &[0xA3, 0x5F, 0x02, 0x00];

#[derive(Debug)]
pub struct DePlayer {
    pub dlc_id: u32,
    pub color_id: i32,
    pub selected_color: u8,
    pub selected_team_id: u8,
    pub resolved_team_id: u8,
    pub dat_crc: [u8; 8],
    pub mp_game_version: u8,
    pub civ_id: u32,
    pub unk1: Option<u32>,
    pub ai_type: String,
    pub ai_civ_name_index: u8,
    pub ai_name: String,
    pub name: String,
    pub player_type: u32,
    pub profile_id: u32,
    pub unk2: u32,
    pub player_number: i32,
    pub hd_rm_elo: Option<u32>,
    pub hd_dm_elo: Option<u32>,
    pub prefer_random: bool,
    pub custom_ai: bool,
    pub handicap: Option<[u8; 8]>,
}

#[derive(Debug)]
pub struct DeHeader {
    pub build: Option<u32>,
    pub timestamp: Option<u32>,
    pub version: f32,
    pub interval_version: u32,
    pub game_options_version: u32,
    pub dlc_count: u32,
    pub dlc_ids: Vec<u32>,
    pub dataset_ref: u32,
    pub difficulty_id: u32,
    pub selected_map_id: u32,
    pub resolved_map_id: u32,
    pub reveal_map: u32,
    pub victory_type_id: u32,
    pub starting_resources_id: u32,
    pub starting_age_id: u32,
    pub ending_age_id: u32,
    pub game_type: u32,
    pub speed: f32,
    pub treaty_length: u32,
    pub population_limit: u32,
    pub num_players: u32,
    pub unused_player_color: u32,
    pub victory_amount: i32,
    pub unk_byte: Option<bool>,
    pub trade_enabled: bool,
    pub team_bonus_disabled: bool,
    pub random_positions: bool,
    pub all_techs: bool,
    pub num_starting_units: u8,
    pub lock_teams: bool,
    pub lock_speed: bool,
    pub multiplayer: bool,
    pub cheats: bool,
    pub record_game: bool,
    pub animals_enabled: bool,
    pub predators_enabled: bool,
    pub turbo_enabled: bool,
    pub shared_exploration: bool,
    pub team_positions: bool,
    pub sub_game_mode: Option<u32>,
    pub battle_royale_time: Option<u32>,
    pub handicap: Option<bool>,
    pub unk: Option<bool>,
    pub players: Vec<DePlayer>,
    pub fog_of_war: u8, // TODO: report aoc-mgz bug
    pub cheat_notifications: bool,
    pub colored_chat: bool,
}

fn mgz_de_player(save_version: f32) -> impl Fn(&[u8]) -> IResult<&[u8], DePlayer> {
    move |input: &[u8]| -> IResult<&[u8], DePlayer> {
        let (input, dlc_id) = le_u32(input)?;
        let (input, color_id) = le_i32(input)?;
        let (input, selected_color) = u8(input)?;
        let (input, selected_team_id) = u8(input)?;
        let (input, resolved_team_id) = u8(input)?;
        let (input, dat_crc) = take(8_u8)(input)?;
        let (input, mp_game_version) = u8(input)?;
        let (input, civ_id) = le_u32(input)?;
        let (input, unk1) = cond(save_version >= 61.5f32, le_u32)(input)?;
        let (input, ai_type) = de_string(input)?;
        let (input, ai_civ_name_index) = u8(input)?;
        let (input, ai_name) = de_string(input)?;
        let (input, name) = de_string(input)?;
        let (input, player_type) = le_u32(input)?; // TODO: map to enum
        let (input, profile_id) = le_u32(input)?;
        let (input, unk2) = le_u32(input)?;
        let (input, player_number) = le_i32(input)?;
        let (input, hd_rm_elo) = cond(save_version < 25.22f32, le_u32)(input)?;
        let (input, hd_dm_elo) = cond(save_version < 25.22f32, le_u32)(input)?;
        let (input, prefer_random) = flag(input)?;
        let (input, custom_ai) = flag(input)?;
        let (input, handicap) = cond(save_version >= 25.06f32, take(8_u8))(input)?;

        let player = DePlayer {
            dlc_id,
            color_id,
            selected_color,
            selected_team_id,
            resolved_team_id,
            dat_crc: <&[u8] as TryInto<[u8; 8]>>::try_into(dat_crc).unwrap(),
            mp_game_version,
            civ_id,
            unk1,
            ai_type: ai_type.to_string(),
            ai_civ_name_index,
            ai_name: ai_name.to_string(),
            name: name.to_string(),
            player_type,
            profile_id,
            unk2,
            player_number,
            hd_rm_elo,
            hd_dm_elo,
            prefer_random,
            custom_ai,
            handicap: handicap.map(|x| x.try_into().unwrap()),
        };

        Ok((input, player))
    }
}

pub fn mgz_de(input: &[u8], save_version: f32) -> IResult<&[u8], DeHeader> {
    let (input, build) = cond(save_version >= 25.22f32, le_u32)(input)?;
    let (input, timestamp) = cond(save_version >= 26.16f32, le_u32)(input)?;
    let (input, version) = le_f32(input)?;
    let (input, interval_version) = le_u32(input)?;
    let (input, game_options_version) = le_u32(input)?;
    let (input, dlc_count) = le_u32(input)?;
    let (input, dlc_ids) = count(le_u32, dlc_count as usize)(input)?;
    let (input, dataset_ref) = le_u32(input)?;
    let (input, difficulty_id) = le_u32(input)?; // Or map size?
    let (input, selected_map_id) = le_u32(input)?;
    let (input, resolved_map_id) = le_u32(input)?;
    let (input, reveal_map) = le_u32(input)?;
    let (input, victory_type_id) = le_u32(input)?; // TODO: Map to enum
    let (input, starting_resources_id) = le_u32(input)?; // TODO: Map to enum
    let (input, starting_age_id) = le_u32(input)?; // TODO: Map to enum
    let (input, ending_age_id) = le_u32(input)?; // TODO: Map to enum
    let (input, game_type) = le_u32(input)?; // TODO: Map to enum
    let (input, _) = tag(SEPARATOR)(input)?;
    let (input, _) = tag(SEPARATOR)(input)?;
    let (input, speed) = le_f32(input)?;
    let (input, treaty_length) = le_u32(input)?;
    let (input, population_limit) = le_u32(input)?;
    let (input, num_players) = le_u32(input)?;
    let (input, unused_player_color) = le_u32(input)?;
    let (input, victory_amount) = le_i32(input)?;
    let (input, unk_byte) = cond(save_version >= 61.5f32, flag)(input)?;
    let (input, _) = tag(SEPARATOR)(input)?;
    let (input, trade_enabled) = flag(input)?;
    let (input, team_bonus_disabled) = flag(input)?;
    let (input, random_positions) = flag(input)?;
    let (input, all_techs) = flag(input)?;
    let (input, num_starting_units) = u8(input)?;
    let (input, lock_teams) = flag(input)?;
    let (input, lock_speed) = flag(input)?;
    let (input, multiplayer) = flag(input)?;
    let (input, cheats) = flag(input)?;
    let (input, record_game) = flag(input)?;
    let (input, animals_enabled) = flag(input)?;
    let (input, predators_enabled) = flag(input)?;
    let (input, turbo_enabled) = flag(input)?;
    let (input, shared_exploration) = flag(input)?;
    let (input, team_positions) = flag(input)?;
    let (input, sub_game_mode) = cond(save_version >= 13.34f32, le_u32)(input)?;
    let (input, battle_royale_time) = cond(save_version >= 13.34f32, le_u32)(input)?;
    let (input, handicap) = cond(save_version >= 25.06f32, flag)(input)?;
    let (input, unk) = cond(save_version >= 50f32, flag)(input)?;
    let (input, _) = tag(SEPARATOR)(input)?;
    let num_players_slots = if save_version >= 37f32 {
        num_players
    } else {
        8
    };
    let (input, players) = count(mgz_de_player(save_version), num_players_slots as usize)(input)?;
    let (input, _) = take(9_u8)(input)?; // unknown
    let (input, fog_of_war) = u8(input)?;
    let (input, cheat_notifications) = flag(input)?;
    let (input, colored_chat) = flag(input)?;
                                            // Continue at: https://github.com/happyleavesaoc/aoc-mgz/blob/master/mgz/header/de.py#L112

    let de_header = DeHeader {
        build,
        timestamp,
        version,
        interval_version,
        game_options_version,
        dlc_count,
        dlc_ids,
        dataset_ref,
        difficulty_id,
        selected_map_id,
        resolved_map_id,
        reveal_map,
        victory_type_id,
        starting_resources_id,
        starting_age_id,
        ending_age_id,
        game_type,
        speed,
        treaty_length,
        population_limit,
        num_players,
        unused_player_color,
        victory_amount,
        unk_byte,
        trade_enabled,
        team_bonus_disabled,
        random_positions,
        all_techs,
        num_starting_units,
        lock_teams,
        lock_speed,
        multiplayer,
        cheats,
        record_game,
        animals_enabled,
        predators_enabled,
        turbo_enabled,
        shared_exploration,
        team_positions,
        sub_game_mode,
        battle_royale_time,
        handicap,
        unk,
        players,
        fog_of_war,
        cheat_notifications,
        colored_chat,
    };

    Ok((input, de_header))
}
