use std::str::FromStr;

use lazy_static::lazy_static;
use reqwest::Url;

pub fn get_gif_url(url: &str, provider: &str) -> Option<String> {
    lazy_static! {
        static ref TENOR: regex::Regex =
            regex::Regex::new(r"^https://media\.tenor\.com?/(.+)/(.*)\.\w+$").unwrap();
    }
    lazy_static! {
        static ref GIPHY: regex::Regex =
            regex::Regex::new(r"^https://media\d*\.giphy\.com/media/([\w-]+)/(.*)\.gif$").unwrap();
    }

    match provider {
        "Tenor" => {
            let caps: Vec<_> = TENOR.captures_iter(url).collect();
            let groups = caps.get(0)?;

            let gif_id = &groups[1];
            let gif_id = gif_id[0..gif_id.len() - 1].to_string() + "C";
            let name = &groups[2];
            Some(format!("https://c.tenor.com/{gif_id}/{name}.gif"))
        }
        "Giphy" => {
            let url = url.split('?').next().unwrap();
            let caps: Vec<_> = GIPHY.captures_iter(url).collect();
            let groups = caps.get(0)?;

            let gif_id = &groups[1];
            let name = &groups[2];
            let name = if name.ends_with("_s") {
                &name[0..name.len() - 2]
            } else {
                name
            };

            Some(format!("https://i.giphy.com/media/{gif_id}/{name}.gif"))
        }
        "Gfycat" => {
            let parsed_url = Url::from_str(url).unwrap();

            let mut path_chars = parsed_url.path().chars();
            path_chars.next();
            let gif_id = path_chars.as_str().split('-').next().unwrap();

            let origin = parsed_url.origin().unicode_serialization();
            if !origin.ends_with(".gfycat.com") {
                return None;
            }

            Some(format!("{origin}/{gif_id}-size_restricted.gif"))
        }
        other => {
            eprintln!("Unkown GIFV provider: {other}\n{url}");
            None
        }
    }
}
