use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use atri_elo_common::EloMmr;
use comfy_table::Table;
use itertools::Itertools;

fn main() {
    let api_key = std::env::var("OSU_API_KEY").unwrap();

    let mut match_ids_file = File::open("match_ids.json").unwrap();

    let mut match_ids_json = String::new();
    match_ids_file.read_to_string(&mut match_ids_json).unwrap();

    let match_ids = gjson::parse(&match_ids_json)
        .array()
        .into_iter()
        .map(|v| v.i32())
        .unique()
        .sorted_unstable()
        .collect_vec();

    let mut name_map: HashMap<i64, String> = serde_json::from_reader(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("cache/name_map.json")
            .unwrap(),
    )
    .unwrap();

    let mut contest_name_map: HashMap<i64, String> = HashMap::new();
    contest_name_map.insert(-1, "-".to_string());
    let mut next_contest_id = 0;

    let mut system = EloMmr::default();

    for id in match_ids {
        println!("processing contest {}", id);
        let match_json = match File::open(format!("cache/contests/{}.json", id)) {
            Ok(mut file) => {
                let mut body = String::new();
                file.read_to_string(&mut body).unwrap();
                body
            }
            Err(_) => {
                let url = format!("https://osu.ppy.sh/api/get_match?k={}&mp={}", api_key, id);
                let response = reqwest::blocking::get(url).unwrap();
                let body = response.text().unwrap();
                let mut file = File::create(format!("cache/contests/{}.json", id)).unwrap();
                file.write_all(body.as_bytes()).unwrap();
                body
            }
        };

        let match_json = gjson::parse(&match_json);

        let match_name = match_json.get("match.name");

        for (index, round) in match_json.get("games").array().into_iter().enumerate() {
            let mut raw_scores = Vec::new();
            let scores = round.get("scores");
            for score in scores.array().iter() {
                raw_scores.push((
                    score.get("user_id").str().to_owned(),
                    score.get("score").str().to_owned(),
                ));
            }
            if raw_scores.is_empty() {
                continue;
            }
            let raw_scores = raw_scores
                .into_iter()
                .map(|(id, s)| (id.parse::<i64>().unwrap(), s.parse::<i64>().unwrap()))
                .collect_vec();
            contest_name_map.insert(
                next_contest_id,
                format!("{} Round #{}", match_name.str().to_string(), index + 1),
            );
            system.update(next_contest_id, raw_scores);
            next_contest_id += 1;
        }
    }

    let mut table = Table::new();
    table.set_header(vec!["Rank", "ID", "Rating"]);
    for (rank, (id, rating)) in system
        .export_player_ratings()
        .into_iter()
        .sorted_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse())
        .enumerate()
    {
        table.add_row(vec![
            (rank + 1).to_string(),
            get_username(&api_key, &mut name_map, id),
            rating.round().to_string(),
        ]);
    }
    let mut report_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("reports/report.txt")
        .unwrap();
    writeln!(report_file, "Leaderboard").unwrap();
    writeln!(report_file, "{}", table).unwrap();

    for (id, history) in system.export_player_history() {
        let mut report_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!(
                "reports/players/{}.txt",
                get_username(&api_key, &mut name_map, id),
            ))
            .unwrap();
        let mut table = Table::new();
        table.set_header(vec![
            "Contest Name",
            "Performance",
            "Rating",
            "Contest Rank",
            "Rating Rank",
        ]);
        for entry in history {
            table.add_row(vec![
                contest_name_map
                    .get(&entry.contest_id())
                    .unwrap()
                    .to_owned(),
                entry.perf().round().to_string(),
                entry.rating().round().to_string(),
                entry.contest_rank().to_string(),
                entry.rating_rank().to_string(),
            ]);
        }
        writeln!(
            report_file,
            "History of {}",
            get_username(&api_key, &mut name_map, id),
        )
        .unwrap();
        writeln!(report_file, "{}", table).unwrap();
    }

    for (id, history) in system.export_contest_details() {
        let mut report_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!(
                "reports/contests/{}.txt",
                contest_name_map.get(&id).unwrap(),
            ))
            .unwrap();
        let mut table = Table::new();
        table.set_header(vec![
            "Player Name",
            "Score",
            "Performance",
            "Rating",
            "Rating Rank",
        ]);
        for &(id, score, perf, rating, rating_rank) in history.data() {
            table.add_row(vec![
                get_username(&api_key, &mut name_map, id),
                score.to_string(),
                perf.round().to_string(),
                rating.round().to_string(),
                rating_rank.to_string(),
            ]);
        }
        writeln!(
            report_file,
            "Detail of {}",
            contest_name_map.get(&id).unwrap(),
        )
        .unwrap();
        writeln!(report_file, "{}", table).unwrap();
    }
}

fn get_username(api_key: &str, name_map: &mut HashMap<i64, String>, id: i64) -> String {
    match name_map.get(&id) {
        Some(name) => name.clone(),
        None => {
            let url = format!(
                "https://osu.ppy.sh/api/get_user?k={}&u={}&type=id",
                api_key, id
            );
            let response = reqwest::blocking::get(url).unwrap();
            let body = response.text().unwrap();
            let name = gjson::parse(&body);
            if name.array().is_empty() {
                println!("cannot find user {}, ignoring", id);
                name_map.insert(id, format!("Unknown ({})", id));
            } else {
                name_map.insert(id, name.array()[0].get("username").str().to_owned());
            }
            let mut name_map_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("cache/name_map.json")
                .unwrap();
            name_map_file
                .write_all(&serde_json::to_vec(&name_map).unwrap())
                .unwrap();
            name_map.get(&id).unwrap().to_string()
        }
    }
}
