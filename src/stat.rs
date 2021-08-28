
// stupid statistics module
// idk if i will even use this

use chrono::{DateTime, Local, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const SCHED_DATA: &str = include_str!("sched_data_11.csv");

#[derive(Serialize, Deserialize, Debug)]
pub struct Stat {
    pub days_rem: usize,
    pub days_rem_pct: String,
    pub ped_rem: usize,
    pub ped_past: usize,
    pub time_ms: f64,
}

pub fn generate_stat(dt: DateTime<Local>) -> Stat {
    let benchmark_dt_start = chrono::Local::now();
    let date = dt.naive_local().date();

    // load it
    let mut whole_schedule: Vec<MiniDay> = vec![];
    let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(SCHED_DATA.as_bytes());
    for r in reader.records() {
        let record = r.unwrap();
        if record.get(0).is_some() {
            let value = record.get(1).expect("corrupt sched csv or sum idk").trim();
            if value == "W" {continue} // NO WEEKENDS
            whole_schedule.push(
                MiniDay {
                    date: NaiveDate::parse_from_str(record.get(0).unwrap(), "%d-%m-%Y").unwrap(),
                    is_ped_etc: if value == "P" || value == "C" {true} else {false},
                }
            );
        }
    }

    let days_total = whole_schedule.len();
    let mut days_passed: usize = 0;
    let mut ped_count: usize = 0;
    let mut ped_rem: usize = 0;

    let mut future = false;

    for day in whole_schedule {
        if day.is_ped_etc {
            ped_count += 1;
        }
        match future {
            false => {
                // already happened up to now
                days_passed += 1;
                if day.date > date {
                    future = true;
                }
            }
            true => {
                if day.is_ped_etc {
                    ped_rem += 1;
                }
            }
        }
    }

    let benchmark_dt_end = chrono::Local::now();
    let benchmark_duration = benchmark_dt_end - benchmark_dt_start;
    let benchmark_duration_ms = benchmark_duration.num_microseconds().unwrap() as f64 / 1000.0;

    // debug // println!("{}, {}",days_passed, days_total);
    let days_rem_pct = (days_passed as f64 / days_total as f64) * 100.0;
    Stat {
        days_rem: days_total - days_passed,
        days_rem_pct: format!("{:.3}", days_rem_pct),
        ped_rem,
        ped_past: ped_count - ped_rem,
        time_ms: benchmark_duration_ms
    }

}

struct MiniDay {
    date: NaiveDate,
    is_ped_etc: bool,
}