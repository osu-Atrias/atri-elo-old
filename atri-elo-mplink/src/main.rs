use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use atri_elo_common::{Contest, EloMmr};
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

    let mut name_map: HashMap<String, String> = serde_json::from_reader(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("cache/name_map.json")
            .unwrap(),
    )
    .unwrap();

    let mut ignore_ids: HashSet<String> = serde_json::from_reader(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("cache/ignore_ids.json")
            .unwrap(),
    )
    .unwrap();

    let mut system = EloMmr::new(1.0, 1500.0, 50.0, 1500.0, 350.0);

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
                .map(|(id, s)| (id, s.parse::<i64>().unwrap()))
                .map(|(id, score)| {
                    let name = match name_map.get(&id) {
                        Some(name) => name.clone(),
                        None => {
                            if ignore_ids.contains(&id) {
                                println!("cannot find user {}, ignoring", id);
                                return ("".to_owned(), -1);
                            }
                            let url = format!(
                                "https://osu.ppy.sh/api/get_user?k={}&u={}&type=id",
                                api_key, id
                            );
                            let response = reqwest::blocking::get(url).unwrap();
                            let body = response.text().unwrap();
                            let name = gjson::parse(&body);
                            if name.array().is_empty() {
                                println!("cannot find user {}, ignoring", id);
                                ignore_ids.insert(id.clone());
                                return ("".to_owned(), -1);
                            } else {
                                name_map.insert(
                                    id.clone(),
                                    name.array()[0].get("username").str().to_owned(),
                                );
                            }
                            name.array()[0].get("username").str().to_owned()
                        }
                    };
                    (name, score)
                })
                .filter(|(_, score)| *score != -1)
                .collect_vec();
            let mut contest = Contest::new(
                format!("{} Round #{}", match_name.str().to_string(), index + 1),
                raw_scores,
            );
            system.update(&mut contest);
            let mut name_map_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("cache/name_map.json")
                .unwrap();
            name_map_file
                .write_all(&serde_json::to_vec(&name_map).unwrap())
                .unwrap();
            let mut ignore_ids_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("cache/ignore_ids.json")
                .unwrap();
            ignore_ids_file
                .write_all(&serde_json::to_vec(&ignore_ids).unwrap())
                .unwrap();
        }
    }

    let mut table = Table::new();
    table.set_header(vec!["Rank", "ID", "Rating"]);
    for (rank, (id, rating)) in system
        .export_ratings()
        .into_iter()
        .sorted_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse())
        .enumerate()
    {
        table.add_row(vec![(rank + 1).to_string(), id, rating.round().to_string()]);
    }
    let mut report_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("reports/report.txt")
        .unwrap();
    writeln!(report_file, "Leaderboard").unwrap();
    writeln!(report_file, "{}", table).unwrap();

    for (id, history) in system.export_history() {
        let mut report_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("reports/players/{}.txt", id))
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
                entry.contest_name().to_string(),
                entry.perf().round().to_string(),
                entry.rating().round().to_string(),
                entry.contest_place().to_string(),
                entry.rating_place().to_string(),
            ]);
        }
        writeln!(report_file, "Contest History of {}", &id).unwrap();
        writeln!(report_file, "{}", table).unwrap();
    }
}
