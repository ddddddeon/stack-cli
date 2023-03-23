use bat::PrettyPrinter;
use flate2::read::GzDecoder;
use regex::Regex;
use reqwest::blocking::get;
use select::{document::Document, predicate::Name};
use serde::{Deserialize, Serialize};
use std::{
    env::{self, args},
    error::Error,
    io::{self, Read, Write},
};

#[derive(Debug)]
struct Question {
    name: String,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Answer {
    body_markdown: String,
    is_accepted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct JSONResponse {
    items: Vec<Answer>,
}

fn url_decode(input: &mut str) {
    let mut chars = input.chars();

    while let Some(mut c) = chars.next() {
        if c == '%' {
            let hex = format!("{}{}", chars.next().unwrap(), chars.next().unwrap());
            let decoded = u8::from_str_radix(&hex, 16).unwrap();
            c = decoded as char;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let access_token = env::var("STACKOVERFLOW_API_KEY")?;
    let key = env::var("STACKOVERFLOW_KEY")?;
    let mut pp = PrettyPrinter::new();
    let link_regex =
        Regex::new(r"/url\?q=https://stack(overflow|exchange)\.com/questions/([0-9]+)/")?;

    let mut args: Vec<String> = args().collect();
    args.remove(0);

    let query_string = args.join("+");
    let url = format!("https://www.google.com/search?q=site:stackoverflow.com {query_string}");

    let questions = get(url)
        .and_then(|d| d.text())
        .map(|text| Document::from(text.as_str()))
        .map(|doc| {
            doc.find(Name("a"))
                .filter_map(|el| {
                    el.attr("href")
                        .and_then(|href| link_regex.captures(href))
                        .and_then(|captured| captured.get(2))
                        .map(|id| Question {
                            name: el.text(),
                            id: String::from(id.as_str()),
                        })
                })
                .fold(Vec::new(), |mut acc, link| {
                    acc.push(link);
                    acc
                })
        })?;

    for (i, question) in questions.iter().enumerate() {
        println!("{}. {}", i, question.name);
    }

    print!("> ");
    io::stdout().flush()?;

    let mut num = String::new();
    io::stdin().read_line(&mut num)?;
    let num: i32 = num.trim().parse()?;
    let question = &questions[num as usize];

    let mut contents = Vec::new();
    get(
        format!("https://api.stackexchange.com/2.3/questions/{}/answers?site=stackoverflow&sort=activity&filter=!nOedRLr0Wi&access_token={}&key={}",
                question.id, access_token, key))?
        .read_to_end(&mut contents)?;

    let mut decoded = String::new();
    let mut decoder = GzDecoder::new(&contents[..]);
    decoder.read_to_string(&mut decoded)?;

    let response: JSONResponse = serde_json::from_str(&decoded)?;
    let answers = response.items;
    let mut decoded_answers: Vec<String> = Vec::new();

    answers
        .iter()
        .filter(|item| {
            if answers.len() == 1 {
                return true;
            }
            item.is_accepted
        })
        .for_each(|answer| {
            let url_decoded = htmlescape::decode_html(&answer.body_markdown).unwrap();
            decoded_answers.push(url_decoded);
        });

    decoded_answers.iter().for_each(move |answer| {
        pp.input_from_bytes(answer.as_bytes())
            .language("markdown")
            .print()
            .unwrap();
    });

    Ok(())
}
