use regex::Regex;
use reqwest::blocking::get;
use select::document::Document;
use select::predicate::Name;
use std::env::args;
use std::error::Error;

#[derive(Debug)]
struct Link {
    name: String,
    href: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let link_regex = Regex::new(r"/url\?q=(https://stackoverflow\.com.*)\&sa")?;
    let mut args: Vec<String> = args().collect();
    args.remove(0);

    let query_string = args.join("+");
    let url = format!("https://www.google.com/search?q=site:stackoverflow.com {query_string}");

    let mut links: Vec<Link> = Vec::new();
    get(url)
        .and_then(|d| d.text())
        .map(|text| Document::from(text.as_str()))
        .map(|doc| {
            doc.find(Name("a")).for_each(|el| {
                if let Some(link) = el
                    .attr("href")
                    .and_then(|href| link_regex.captures(href))
                    .and_then(|captured| captured.get(1))
                    .map(|l| Link {
                        name: el.text(),
                        href: String::from(l.as_str()),
                    })
                {
                    links.push(link);
                }
            });
        })?;

    dbg!(&links);

    Ok(())
}
