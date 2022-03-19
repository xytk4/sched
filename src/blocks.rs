
// Blocks

use chrono::{DateTime, Local, NaiveDate, Timelike};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

const SPECIALS_PATH: &str = "./special.csv";
const ONLINE_PATH:   &str = "./online.csv";
const LOOKUP_PATH:   &str = "./lookup.csv";
const SCHED_CLASSES: &str = include_str!("sched_classes.csv");
const SCHED_DATA:    &str = include_str!("sched_data_11.csv");

#[derive(Serialize, Deserialize, Debug)]
enum BlockStatus {
    NotStarted = 0,
    Normal = 1,
    IsOver = 2,
}

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
    status: i32,
    //is_online: bool,
}

impl Block {
    pub fn generate(dt: DateTime<Local>, now: DateTime<Local>, title: &str) -> Self {
        // what day is it? etc
        let date = dt.naive_local().date();

        let day = Self::day_from_date(&date);
        let mut day_str = Self::format_day(&day);


        let mut classes = match &day {
            Some(d) => Self::classes_from_day(d),
            None => None,
        };
        // this needs its own variable so we can operate on it
        // before it gets stolen by the struct to find out if
        // it is some (since i don't think the template is smart
        // enough for this)
        let classes_is_some = classes.is_some();

        let special = Self::get_special(&date);
        let special_is_some = special.is_some();

        //let is_online = Self::check_online(&date);


        // stupid hack (?) to make a very clear way to cancel a day
        // or other VERY SPECIAL EVENTS that require a whole block change
        if special_is_some {
            let s = special.clone().unwrap();
            let first_spec = s.first();

            match first_spec {
                Some(c) => match c.as_str() {
                    "*CANC" => {
                        // day cancelled
                        return Block {
                            date: date.format("%A, %d-%b-%Y").to_string(),
                            title: title.to_string(),
                            bgcolorcode: "#aaaaaa".to_string(),
                            greeting:   if title == "Today" {
                                "I hope you have a nice day.".to_string()
                            } else {
                                Self::rand_greeting()
                            },
                            day,
                            day_str: "CANCELLED".to_string(),
                            classes: vec![],
                            classes_is_some: false,
                            special: special.unwrap_or_default(),
                            special_is_some: false,
                            status: BlockStatus::Normal as i32,
                            //is_online
                        }
                    },
                    "*CANCSNOW" => {
                        // day cancelled (snow day)
                        // ... just in case.
                        return Block {
                            date: date.format("%A, %d-%b-%Y").to_string(),
                            title: title.to_string(),
                            bgcolorcode: "#bf6565".to_string(),
                            greeting: "I hope you have a nice day.".to_string(),
                            day,
                            day_str: "Snow day!".to_string(),
                            classes: vec![],
                            classes_is_some: false,
                            special: special.unwrap_or_default(),
                            special_is_some: false,
                            status: BlockStatus::Normal as i32,
                            //is_online
                        }
                    },
                    _ => {}
                },
                None => {}
            };
        }


        let status = match &day {
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
                        if dt.hour() < 8
                            && dt.naive_local().date() == now.naive_local().date() {
                            BlockStatus::NotStarted
                        } else if dt.hour() >= 16
                            && dt.naive_local().date() == now.naive_local().date() {
                            BlockStatus::IsOver
                        } else {
                            BlockStatus::Normal
                        }
                    }
                    Day::Day9 => {
                        if dt.hour() < 8
                            && dt.naive_local().date() == now.naive_local().date() {
                            BlockStatus::NotStarted
                        } else if dt.hour() >= 13 // safe estimates
                            && dt.naive_local().date() == now.naive_local().date() {
                            BlockStatus::IsOver
                        } else {
                            BlockStatus::Normal
                        }
                    }
                    Day::Ped |
                    Day::Holiday |
                    Day::HolidayDontCount |
                    Day::Weekend |
                    Day::Unknown => BlockStatus::Normal // not over, it never started
                }
            },
            None => BlockStatus::Normal
        };


        // alter
        let mut bgcolorcode = "#2b3032".to_string();
        match classes {
            Some(x) => {
                let alter = LookupAlter::alter(&date, x).unwrap();
                classes = Some(alter.html()); // やべ
                if let Some(c) = alter.ctd {
                    day_str = c;
                }
                bgcolorcode = if let Some(ctdc) = alter.ctdcolor {
                    ctdc
                } else {
                    match &day {
                        Some(d) => Self::bgcolorcode(d),
                        None => "#2b3032".to_string() // default
                    }
                }
            }
            None => {}
        }

        // generate struct
        Block {
            date: date.format("%A, %d-%b-%Y").to_string(),
            title: title.to_string(),
            bgcolorcode,
            greeting:   if title == "Today" {
                            "I hope you have a nice day.".to_string()
                        } else {
                            Self::rand_greeting()
                        },
            day,
            day_str,
            classes: classes.unwrap_or_default(),
            classes_is_some,
            special: special.unwrap_or_default(),
            special_is_some,
            status: status as i32,
            //is_online
        }
    }
    pub fn classes_from_day(day: &Day) -> Option<Vec<String>> {
        match day {
            Day::Ped |
            Day::Holiday |
            Day::HolidayDontCount |
            Day::Weekend |
            Day::Unknown => {
                return None;
            }
            _ => {}
        }

        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(SCHED_CLASSES.as_bytes());

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
    pub fn day_from_date(now: &NaiveDate) -> Option<Day> {
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
                        match record.get(1).expect("corrupt sched csv or sum idk").trim() {
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
                            "D" => Day::HolidayDontCount,
                            "W" => Day::Weekend,
                            _ => Day::Unknown,
                        }
                    );
                }
            }
        }
        None
    }

    pub fn get_special(date: &NaiveDate) -> Option<Vec<String>> {
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

    // i want to remove this so bad. i commented all usages out
    // (ao Fri 27-Aug-2021 09:32 PM) but i'll leave the function etc here.
    // just in case.
    pub fn check_online(date: &NaiveDate) -> bool {
        let mut reader = match csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(ONLINE_PATH)
        {
            Ok(r) => r,
            Err(_) => return false, // like specials, this will get updated. assume false as much as possible
        };

        let date_str = date.format("%d-%m-%Y").to_string();
        for r in reader.records() {
            let record = r.unwrap_or_default();
            match record.get(0) {
                Some(r) => {
                    if r == date_str {
                        return true
                    }
                }
                None => {}
            }
        }
        false
    }

    pub fn format_day(day: &Option<Day>) -> String {
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
                Day::Holiday | Day::HolidayDontCount
                    => "a Holiday of Some Sort, idk look it up in the calendar",
                Day::Weekend => "the weekend",
                Day::Unknown => "unknown ???",
            }.to_string(),
            None => "no day (this is probably an error!!)".to_string(),
        }
    }

    fn bgcolorcode(day: &Day) -> String {
        match day {
            Day::Day1 => "#ad253e", //"#5b140b",
            Day::Day2 => "#6a4823",
            Day::Day3 => "#296a33",
            Day::Day4 => "#2f6a5f",
            Day::Day5 => "#29556a",
            Day::Day6 => "#3d386a",
            Day::Day7 => "#6a3a62",
            Day::Day8 => "#79141e",
            Day::Day9 => "#56617a",

            Day::Ped => "#549ac6",
            Day::Holiday | Day::HolidayDontCount => "#c68252",
            Day::Weekend => "#2b3032",
            Day::Unknown => "#FF0000", // should never see this
        }.to_string()
    }

    fn rand_greeting() -> String {
        let greetings = vec![
            "I hope you have a great day.",
            "I hope you have a wonderful day.",
            "I hope you have an incredible day.",
            "I hope you have an exciting day.",
            "I hope you have an especially pleasant day.",
            "I especially hope you will have a nice day.",
            "I especially hope you will have a pleasant day.",
            "I hope you will have a pleasant day.",
            "I hope you will have a relaxing day.",
            "I hope you will have an extremely fun day.",
            //"I hope I won't run out of randomly-generated messages to put here",
            "I hope you have an awesome day.",
            "I hope YOU specifically will have a nice day.",
            "I hope you, more than anyone else, will have a great day.",
            "I hope you have a randomly-generated day.",
            //"I hope you have a day free from randomly-generated descriptions like this one.",
            //"I hope you have a day full of randomly-generated descriptions like this one.",
            "I wish you a wonderful wonderful day.",
            "I hope you will have a reasonably normal day.",
            "I hope you won't have a bad day.",
            "I hope you excel academically today.",
            "I hope you will have a very unpredictable day.",
            "I hope you will have a very predictable day.",
            "J'espère que vous passerez une journée extraordinaire.",
            "今日、がんばってね"
            // that's enough I hope
        ];
        greetings.choose(&mut rand::thread_rng()).unwrap().to_string()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Day {
    Day1 = 0,
    Day2 = 1,
    Day3 = 2,
    Day4 = 3,
    Day5 = 4,
    Day6 = 5,
    Day7 = 6,
    Day8 = 7,
    Day9 = 8,
    Ped,
    Holiday,
    HolidayDontCount,
    Weekend,
    Unknown
}

struct LookupAlter {
    pub classes: Vec<String>,
    pub ctd: Option<String>,
    pub ctdcolor: Option<String>,
}

impl LookupAlter {
    pub fn alter(date: &NaiveDate, classes: Vec<String>) -> Result<Self, ()> {
        let mut reader = match csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(LOOKUP_PATH)
        {
            Ok(r) => r,
            Err(_) => return Err(()),
        };
        let date_str = date.format("%d-%m-%Y").to_string();

        let mut newclasses = classes.clone();
        let mut ctd = ""; // "change the day"
        let mut ctdcolor = "";

        for r in reader.records() {
            let record = r.unwrap_or_default();
            if record.get(0).unwrap_or_default() == date_str {
                let r1 = record.get(1).unwrap_or_default();
                let p = r1.parse::<usize>();
                if p.is_ok() {
                    newclasses[p.unwrap() - 1] = "$".to_owned() + record.get(2).unwrap_or_default();
                } else if r1 == "CTD" {
                    match record.get(2) {
                        Some(x) => {
                            match x {
                                "ProductionWeek" => {
                                    for (i, class) in classes.iter().enumerate() {
                                        if !(class == "Chant" || class == "Instro" || class == "Lunch") {
                                            newclasses[i] = " ".to_string();
                                        }
                                    }
                                    ctd = "a Production Day";
                                    ctdcolor = "#4e94af";
                                },
                                "ProductionWeekShow" => {
                                    ctd = "a Show!";
                                    ctdcolor = "#cb762d";
                                }
                                _ => {}
                            }
                        }
                        None => {}
                    }
                }
            }
        }

        Ok(Self {
            classes: newclasses,
            ctd: if ctd == "" {None} else {Some(ctd.to_string())},
            ctdcolor: if ctdcolor == "" {None} else {Some(ctdcolor.to_string())} //4e94af
        })
    }

    pub fn html(&self) -> Vec<String> {
        let mut htmlclasses: Vec<String> = vec![];
        for class in &self.classes {
            if class.starts_with("$") { // ya ok this kinda sucks
                htmlclasses.push(format!("<b><i>{}</i></b>", class.trim_matches('$')));
            } else {
                htmlclasses.push(class.to_string());
            }
        }
        htmlclasses
    }
}