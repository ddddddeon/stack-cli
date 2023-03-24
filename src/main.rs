use bat::PrettyPrinter;
use regex::Regex;
use reqwest::blocking::get;
use select::{document::Document, predicate::Name};
use stack_cli::{construct_google_url, prompt, Question};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
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

    let answers = prompt(questions)?;
    answers
        .iter()
        .filter(|item| {
            if answers.len() == 1 {
                return true;
            }
            item.is_accepted
        })
        .map(
            |answer| match htmlescape::decode_html(&answer.body_markdown) {
                Ok(d) => {
                    dbg!(&d);
                    d
                }
                Err(_) => answer.body_markdown.clone(),
            },
        )
        .fold(Vec::new(), |mut acc, answer| {
            acc.push(answer);
            acc
        })
        .iter()
        .for_each(move |answer| {
            let mut pp = PrettyPrinter::new();
            pp.input_from_bytes(answer.as_bytes())
                .language("markdown")
                .print()
                .unwrap();
        });

    Ok(())
}
