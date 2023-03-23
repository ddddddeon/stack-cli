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

fn construct_google_url() -> String {
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    let query_string = args.join("+");
    return format!("https://www.google.com/search?q=site:stackoverflow.com {query_string}");
}

fn gather_answers(questions: Vec<Question>) -> Result<Vec<Answer>, Box<dyn Error>> {
    let access_token = env::var("STACKOVERFLOW_API_KEY")?;
    let key = env::var("STACKOVERFLOW_KEY")?;

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
    GzDecoder::new(&contents[..]).read_to_string(&mut decoded)?;

    let answers: Vec<Answer> = serde_json::from_str::<JSONResponse>(&decoded)?.items;
    Ok(answers)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut pp = PrettyPrinter::new();
    let link_regex =
        Regex::new(r"/url\?q=https://stack(overflow|exchange)\.com/questions/([0-9]+)/")?;

    let url = construct_google_url();

    let questions = get(url)
        .and_then(|d| d.text())
        .map(|text| Document::from(text.as_str()))
        .map(|doc| {
            doc.find(Name("a"))
                .filter_map(|el| {
                    el.attr("href")
                        .and_then(|href| link_regex.captures(href))
                        .and_then(|captured| captured.get(2))
                        .map(|id| {
                            let text = el.text();
                            let text: Vec<&str> = text.split("stackoverflow.com").collect();
                            return Question {
                                name: text[0].to_string(),
                                id: String::from(id.as_str()),
                            };
                        })
                })
                .fold(Vec::new(), |mut acc, link| {
                    acc.push(link);
                    acc
                })
        })?;

    let answers = gather_answers(questions)?;

    answers
        .iter()
        .filter(|item| {
            if answers.len() == 1 {
                return true;
            }
            item.is_accepted
        })
        .map(|answer| htmlescape::decode_html(&answer.body_markdown).unwrap())
        .fold(Vec::new(), |mut acc, answer| {
            acc.push(answer);
            acc
        })
        .iter()
        .for_each(move |answer| {
            pp.input_from_bytes(answer.as_bytes())
                .language("markdown")
                .print()
                .unwrap();
        });

    Ok(())
}
