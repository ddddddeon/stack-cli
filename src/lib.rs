use flate2::read::GzDecoder;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::{
    env::{self, args},
    error::Error,
    io::{self, Read, Write},
};

#[derive(Debug)]
pub struct Question {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Answer {
    pub body_markdown: String,
    pub is_accepted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONResponse {
    items: Vec<Answer>,
}

pub fn construct_google_url() -> String {
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    let query_string = args.join("+");

    format!("https://www.google.com/search?q=site:stackoverflow.com {query_string}")
}

pub fn prompt(questions: Vec<Question>) -> Result<Vec<Answer>, Box<dyn Error>> {
    for (i, question) in questions.iter().enumerate() {
        println!("{}. {}", i, question.name);
    }
    print!("> ");
    io::stdout().flush()?;

    let mut num = String::new();
    io::stdin().read_line(&mut num)?;
    let num: i32 = num.trim().parse()?;
    if num as usize >= questions.len() {
        return Err("Number too high".into());
    }

    let question = &questions[num as usize];
    let answers = gather_answers(question)?;

    Ok(answers)
}

pub fn gather_answers(question: &Question) -> Result<Vec<Answer>, Box<dyn Error>> {
    let access_token = env::var("STACKOVERFLOW_API_KEY")?;
    let key = env::var("STACKOVERFLOW_KEY")?;
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
