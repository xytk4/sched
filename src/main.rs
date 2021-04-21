#![feature(proc_macro_hygiene, decl_macro)]

mod blocks;

#[macro_use] extern crate rocket;
use rocket_contrib::templates::{Template};

use chrono::{Timelike, Duration};


#[derive(serde::Serialize)]
struct TemplateContext<'r> {
    blocks: &'r Vec<blocks::Block>,
    show_banner: &'r bool,
    nextcount: &'r i32,
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
            if c > 100 {
                return Template::render("silly", &SillyTemplateContext {
                    count: &c
                })
            }
        }
        None => {}
    }

    // figure it out
    let now = chrono::Local::now();
    let date = now.naive_local().date();

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

    // render
    Template::render("sched", &TemplateContext {
        blocks: &bks,
        show_banner: &show_banner,
        nextcount: &match count {
            Some(c) => c + 7,
            None => 10,
        },
    })
}

fn main() {
    rocket::ignite().mount("/", routes![sched]).attach(Template::fairing()).launch();
}