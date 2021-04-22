
// Blocks

use chrono::{DateTime, Local, NaiveDate, Timelike};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

const SPECIALS_PATH: &str = "./special.csv";
const SCHED_CLASSES: &str = include_str!("sched_classes.csv");
const SCHED_DATA:    &str = include_str!("sched_data.csv"   );

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    date: String,
    title: String,
    bgcolorcode: String,
    greeting: String,
    day: Option<Day>,
    day_str: String,
    classes: Vec<String>,
    classes_is_some: bool,
    special: Vec<String>,
    special_is_some: bool,
    is_over: bool,
}

impl Block {
    pub fn generate(dt: DateTime<Local>, title: &str) -> Self {
        // what day is it? etc
        let date = dt.naive_local().date();

        let day = Self::day_from_date(&date);
        let day_str = Self::format_day(&day);

        let classes = match &day {
            Some(d) => Self::classes_from_day(d),
            None => None,
        };
        // this needs its own variable so we can operate on it
        // before it gets stolen by the struct to find out if
        // it is some (since i don't think the template is smart
        // enough for this)
        let classes_is_some= classes.is_some();

        let special = get_special(&date);
        let special_is_some = special.is_some();

        let is_over = match &day {
            Some(d) => {
                match d {
                    Day::Day1 |
                    Day::Day2 |
                    Day::Day3 |
                    Day::Day4 |
                    Day::Day5 |
                    Day::Day6 |
                    Day::Day7 |
                    Day::Day8 => {
                        if dt.hour() >= 16
                            && dt.naive_local().date() == chrono::Local::now().naive_local().date() {
                            true
                        } else {
                            false
                        }
                    }
                    Day::Day9 => {
                        if dt.hour() >= 13 // safe estimates
                            && dt.naive_local().date() == chrono::Local::now().naive_local().date() {
                            true
                        } else {
                            false
                        }
                    }
                    Day::Ped |
                    Day::Holiday |
                    Day::Unknown => false
                }
            },
            None => false
        };

        // generate struct
        Block {
            date: date.format("%A, %d-%m-%Y").to_string(),
            title: title.to_string(),
            bgcolorcode: match &day {
                Some(d) => Self::bgcolorcode(d),
                None => "#2b3032".to_string() // default
            },
            greeting: if title == "Today" {"I hope you have a nice day".to_string()} else {Self::rand_greeting()},
            day,
            day_str,
            classes: classes.unwrap_or_default(),
            classes_is_some,
            special: special.unwrap_or_default(),
            special_is_some,
            is_over
        }
    }
    fn classes_from_day(day: &Day) -> Option<Vec<String>> {
        match day {
            Day::Ped |
            Day::Holiday |
            Day::Unknown => {
                return None;
            }
            _ => {}
        }

        let mut reader = csv::Reader::from_reader(SCHED_CLASSES.as_bytes());

        let r = reader.records().nth(match day {
            Day::Day1 => 0,
            Day::Day2 => 1,
            Day::Day3 => 2,
            Day::Day4 => 3,
            Day::Day5 => 4,
            Day::Day6 => 5,
            Day::Day7 => 6,
            Day::Day8 => 7,
            Day::Day9 => 8,
            _ => unreachable!() // i hope
        }).unwrap().unwrap();
        // we have the line
        // turn it into Vec of string
        let mut t: Vec<String> = r.iter().map(|x| x.to_string()).collect();
        for item in t.clone().iter().rev() { // remove empties
            // right it's a csv thing they all have to have the same amount of rows
            // but we don't want blank items
            if item == &"".to_string() {
                t.pop(); // this sounds good but COULD FAIL BE CAREFUL I'M NOT 100% SURE ON THIS ONE
                // basically I think it just needs to be empty only at the end
                // like we can't have empty ones in the middle or this will mess up badly
            }
        }
        Some(t)
    }
    fn day_from_date(now: &NaiveDate) -> Option<Day> {
        let now_str = now.format("%d-%m-%Y").to_string();
        let mut reader = csv::Reader::from_reader(SCHED_DATA.as_bytes());
        for r in reader.records() {
            let record = r.unwrap();
            if record.get(0).is_some() {
                let x =  record.get(0).unwrap();
                if x == now_str {
                    return Some(
                        // we are assuming that our data is all good. since, you know,
                        // it's included in the binary and i'm not about to change it for now
                        // so we can assume stuff like "if there's a value at 0, there will FOR
                        // SURE be one at index 1" stuff like that
                        match record.get(1).unwrap() {
                            "1" => Day::Day1,
                            "2" => Day::Day2,
                            "3" => Day::Day3,
                            "4" => Day::Day4,
                            "5" => Day::Day5,
                            "6" => Day::Day6,
                            "7" => Day::Day7,
                            "8" => Day::Day8,
                            "9" => Day::Day9,
                            "P" => Day::Ped,
                            "C" => Day::Holiday,
                            _ => Day::Unknown,
                        }
                    );
                }
            }
        }
        None
    }

    fn format_day(day: &Option<Day>) -> String {
        match day {
            Some(d) => match d {
                Day::Day1 => "Day 1",
                Day::Day2 => "Day 2",
                Day::Day3 => "Day 3",
                Day::Day4 => "Day 4",
                Day::Day5 => "Day 5",
                Day::Day6 => "Day 6",
                Day::Day7 => "Day 7",
                Day::Day8 => "Day 8",
                Day::Day9 => "Day 9 (half day!)",
                Day::Ped => "a Ped Day",
                Day::Holiday => "a Holiday of Some Sort or Another IDK man Look it Up in the Calendar",
                Day::Unknown => "unknown ???",
            }.to_string(),
            None => "no day (weekend, most likely)".to_string(),
        }
    }

    fn bgcolorcode(day: &Day) -> String {
        match day {
            Day::Day1 => "#5b140b",
            Day::Day2 => "#6a4823",
            Day::Day3 => "#296a33",
            Day::Day4 => "#2f6a5f",
            Day::Day5 => "#29556a",
            Day::Day6 => "#3d386a",
            Day::Day7 => "#6a3a62",
            Day::Day8 => "#79141e",
            Day::Day9 => "#56617a",

            Day::Ped => "#549ac6",
            Day::Holiday => "#c68252",
            Day::Unknown => "#2b3032",
        }.to_string()
    }

    fn rand_greeting() -> String {
        let greetings = vec![
            "I hope you have a great day.",
            "I hope you have a wonderful day.",
            "I hope you have an incredible day.",
            "I hope you have an exciting day.",
            "I hope you have an especially pleasant day.",
            "I especially hope you'll have a nice day.",
            "I especially hope you'll have a pleasant day",
            "I hope you'll have a pleasant day.",
            "I hope you'll have a relaxing day.",
            "I hope you'll have an extremely fun day.",
            //"I hope I won't run out of randomly-generated messages to put here",
            "I hope you have an awesome day.",
            "I hope YOU specifically will have a nice day.",
            "I hope you, more than anyone else, will have a great day.",
            "I hope you have a randomly-generated day",
            "I hope you have a day free from randomly-generated descriptions like this one",
            "I hope you have a day full of randomly-generated descriptions like this one",
            "I wish you a wonderful wonderful day."
            // that's enough I hope
        ];
        greetings.choose(&mut rand::thread_rng()).unwrap().to_string()
    }
}

fn get_special(date: &NaiveDate) -> Option<Vec<String>> {
    let mut reader = match csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(SPECIALS_PATH)
    {
        Ok(r) => r,
        Err(_) => return None, // right, basically this will get updated on runtime so we don't want
                               // it to die on us. so just be careful around here, use default, etc
    };

    let date_str = date.format("%d-%m-%Y").to_string();
    let mut specials: Vec<String> = vec![];
    for r in reader.records() {
        let record = r.unwrap_or_default();
        match record.get(0) {
            Some(r) => {
                if r == date_str {
                    // we good here
                    specials
                        .push(record.get(1)
                            .unwrap_or("oh no! error code 1 or something idk basically I made a mistake")
                            .to_string()
                        )
                }
            },
            None => {}
        }
    }
    if specials.len() > 0 {
        Some(specials)
    } else {
        None
    }

}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum Day {
    Day1,
    Day2,
    Day3,
    Day4,
    Day5,
    Day6,
    Day7,
    Day8,
    Day9,
    Ped,
    Holiday,
    Unknown
}