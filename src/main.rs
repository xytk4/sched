#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use chrono::{Duration, Timelike, NaiveDate, DateTime, Local, TimeZone};
use rocket_contrib::templates::Template;
use crate::blocks::{Block, Day};
use crate::stat::*;
use serde::{Deserialize, Serialize};

mod blocks;
mod stat;

#[derive(serde::Serialize)]
struct TemplateContext<'r> {
    blocks: &'r Vec<blocks::Block>,
    stat: &'r Stat,
    show_banner: &'r bool,
    nextcount: &'r i32,
    benchmark_duration_ms: &'r f64,
    benchmark_stat_pct: &'r f64
}

#[derive(serde::Serialize)]
struct SillyTemplateContext<'r> {
    count: &'r i32,
}

#[get("/sched?<count>")]
fn sched(count: Option<i32>) -> Template {
    // first, prevent silly nonsense like requesting a BILLION things
    match count {
        Some(c) => {
            if c > 80 {
                return Template::render("silly", &SillyTemplateContext {
                    count: &c
                })
            }
        }
        None => {}
    }

    let benchmark_dt_start = chrono::Local::now();

    // figure it out
    let now = chrono::Local::now();

    let show_banner = match now.hour() {
        22 | 23 | 24 | 0 | 1 | 2 => true, // good enough for me
        _ => false,
    };

    let mut bks= vec![
        blocks::Block::generate(now, "Today"),
        blocks::Block::generate(now + Duration::days(1), "Tomorrow"),
        blocks::Block::generate(now + Duration::days(2), "Day after tomorrow"),
        blocks::Block::generate(now + Duration::days(3), "Day after day after tomorrow"),
    ];

    // let's try this
    match count {
        Some(c) => {
            for i in 4..=c {
                // oh hell yeah that's what i'm talking about
                bks.push(blocks::Block::generate(now + Duration::days(i as i64), ""))
            }
        }
        None => {}
    }

    let benchmark_dt_end = chrono::Local::now();
    let benchmark_duration = benchmark_dt_end - benchmark_dt_start;
    let benchmark_duration_ms = (benchmark_duration.num_microseconds().unwrap() as f64 / 1000.0) + 0.5;

    let s = generate_stat(now);

    let benchmark_stat_pct = &s.time_ms / benchmark_duration_ms * 100.0;

    // render
    Template::render("sched", &TemplateContext {
        blocks: &bks,
        stat: &s,
        show_banner: &show_banner,
        nextcount: &match count {
            Some(c) => c + 7,
            None => 10,
        },
        benchmark_duration_ms: &benchmark_duration_ms,
        benchmark_stat_pct: &benchmark_stat_pct
    })
}

#[get("/api?<date>")]
fn api(date: String) -> String {
    let date = if date == "now" {
        // lazy
        chrono::Local::now().naive_local().date()
    } else {
        let date_p = NaiveDate::parse_from_str(date.as_str(), "%d-%m-%Y");
        match date_p {
            Ok(d) => d,
            Err(_) => return "bad_date".to_string(),
        }
    };

    let day = match Block::day_from_date(&date) {
        Some(d) => d,
        None => return "no_day".to_string(),
    };
    match day {
        Day::Ped |
        Day::Holiday |
        Day::Unknown => return "no_school_day".to_string(),
        _ => {}
    }

    let classes = blocks::Block::classes_from_day(&day).unwrap();
    let special = blocks::Block::get_special(&date).unwrap_or_default();

    // ok so we know it's a valid day with classes
    let a = ApiBlock {
        date: date.format("%A, %d-%b-%Y").to_string(),
        day: blocks::Block::format_day(&Some(day)),
        classes,
        special,
        is_online: blocks::Block::check_online(&date)
    };

    let j = serde_json::to_string(&a).unwrap_or("balls".to_string());

    j
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiBlock {
    date: String,
    day: String,
    classes: Vec<String>,
    special: Vec<String>,
    is_online: bool
}

#[get("/api")]
fn api_help() -> String {
    "api for sched. i'll write docs later.".to_string()
}


fn main() {
    rocket::ignite().mount("/", routes![sched, api, api_help]).attach(Template::fairing()).launch();
}