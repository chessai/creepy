#[macro_use]
extern crate clap;
extern crate html5ever;
extern crate reqwest;
extern crate scraper;
extern crate select;
extern crate serde;
extern crate serde_regex;
mod types;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use types::*;
use scraper::{Html, Selector};

fn read_file_contents(file_name: &str) -> std::io::Result<String> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents);
}

fn decode_toml<D: serde::de::DeserializeOwned>(file_name: &str) -> Result<D, toml::de::Error> {
    let contents = read_file_contents(file_name).expect("Could not read toml file");
    return toml::from_str(&contents);
}

fn main() {
    let matches = clap_app!( creepy =>
        (version: "1.0")
        (author: "Zachary Churchill <zacharyachurchill@gmail.com>")
        (about: "🐛 Creepy crawly web crawler")
        (@subcommand crawly =>
            (about: "Web crawling")
            (@arg CONFIG: --config -c +required +takes_value "Config file: domains, blacklist, etc")
        )
        (@subcommand configure =>
            (about: "Configuration options")
            (@arg default: --default conflicts_with("full") "generate a default configuration")
            (@arg full: --full conflicts_with("default") "generate a full default configuration")
        )
    )
    .get_matches();

    // CONFIGURE
    if let Some(configure_matches) = matches.subcommand_matches("configure") {
        if configure_matches.is_present("default") {
            let default_config: Config = Config {
                domains: Vec::new(),
                blacklist: Vec::new(),
                whitelist: Vec::new(),
                respect_robots_txt: false,
                link_criteria: Vec::new(),
                match_criteria: Vec::new(),
                period: Duration::from_secs(0),
            };
            println!("{}", toml::to_string_pretty(&default_config).unwrap())
        }
        if configure_matches.is_present("full") {
            let full_config: Config = Config {
                domains: vec![String::from("https://github.com/goolord")],
                blacklist: vec![Regex::new(".*").unwrap()],
                whitelist: vec![Regex::new("https://github.com/goolord.*").unwrap()],
                respect_robots_txt: true,
                link_criteria: Vec::new(),
                match_criteria: Vec::new(),
                period: Duration::from_secs(1),
            };
            println!("{}", toml::to_string_pretty(&full_config).unwrap())
        }
    }

    // CRAWLY
    if let Some(crawly_matches) = matches.subcommand_matches("crawly") {
        // unwrap required args
        let config_file_name: &str = crawly_matches.value_of("CONFIG").unwrap();
        let config: Config = decode_toml(config_file_name).expect("Could not decode config");
        // validate the config
        if config.domains.iter().fold(false, |acc, domain| !config.valid_domain(domain) || acc) {
            panic!("Your blacklist overrides domains you have set")
        }
        // crawl
        let mut hits: Vec<&str> = Vec::new();
        let mut misses: Vec<&str> = Vec::new();
        fn crawl_single(domain: &str) -> Option<Vec<&str>> {
            let mut response: reqwest::Response = reqwest::get(domain).unwrap();
            let body: String = response.text().unwrap();
            let document: Html = Html::parse_document(&body);
            let default_link_selector: Selector = Selector::parse("a[href]").unwrap();
            let links = document.select(&default_link_selector).next().unwrap().value();
            println!("{:?}", links);
            return None
        }
        for domain in config.domains {
            crawl_single(&domain);
        }
    }
}

