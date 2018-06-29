use reqwest::*;
use reqwest::header::{Headers, UserAgent, ContentType, Accept, qitem};
use reqwest::mime::*;

// Endpoints
const DOG: &str = "https://dog.ceo/api/breeds/image/random";
const CAT: &str = "http://aws.random.cat/meow";
const URBAN: &str = "https://api.urbandictionary.com/v0/define";
const DAD_JOKE: &str = "https://icanhazdadjoke.com";
const FURRY: &str = "https://e621.net/post/index.json";
const WEATHER: &str = "https://api.openweathermap.org/data/2.5/weather";
const DANBOORU: &str = "https://danbooru.donmai.us/posts.json";
const MAL_ANIME: &str = "https://myanimelist.net/api/anime/search.xml";
const MAL_MANGA: &str = "https://myanimelist.net/api/manga/search.xml";
const DBOTS: &str = "http://discordbots.org/api/bots/{}/stats";

// Deserialization structs
#[derive(Deserialize, Debug)]
pub struct Dog {
    pub message: String,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct Cat {
    pub file: String,
}

#[derive(Deserialize, Debug)]
pub struct Urban {
    pub tags: Vec<String>,
    pub list: Vec<UrbanListItem>,
    pub sounds: Vec<String>,
    pub result_type: String,
}

#[derive(Deserialize, Debug)]
pub struct UrbanListItem {
    pub definition: String,
    pub permalink: String,
    pub thumbs_up: usize,
    pub thumbs_down: usize,
    pub author: String,
    pub word: String,
    pub defid: usize,
    pub current_vote: String,
    pub written_on: String,
    pub example: String,
}

#[derive(Deserialize, Debug)]
pub struct CreatedAt {
    pub json_class: String,
    pub s: usize,
    pub n: usize,
}

#[derive(Deserialize, Debug)]
pub struct FurPost {
    pub id: usize,
    pub tags: String,
    pub locked_tags: Option<String>,
    pub description: String,
    pub created_at: CreatedAt,
    pub creator_id: usize,
    pub author: String,
    pub change: usize,
    pub source: Option<String>,
    pub score: isize,
    pub fav_count: usize,
    pub md5: Option<String>,
    pub file_size: Option<usize>,
    pub file_url: String,
    pub file_ext: Option<String>,
    pub preview_url: String,
    pub preview_width: Option<usize>,
    pub preview_height: Option<usize>,
    pub sample_url: Option<String>,
    pub sample_width: Option<usize>,
    pub sample_height: Option<usize>,
    pub rating: String,
    pub status: String,
    pub width: usize,
    pub height: usize,
    pub has_comments: bool,
    pub has_notes: bool,
    pub has_children: Option<bool>,
    pub children: Option<String>,
    pub parent_id: Option<usize>,
    pub artist: Vec<String>,
    pub sources: Option<Vec<String>>,
}

// The client
pub struct ApiClient {
    pub client: Client,
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::new();
        ApiClient {
            client,
        }
    }

    pub fn dog(&self) -> Result<Dog> {
        match self.client.get(DOG).send() {
            Ok(mut res) => {
                res.json::<Dog>()
            },
            Err(why) => {
                error!("{:?}", why);
                Err(why)
            },
        }
    }

    pub fn cat(&self) -> Result<Cat> {
        match self.client.get(CAT).send() {
            Ok(mut res) => {
                res.json::<Cat>()
            },
            Err(why) => {
                error!("{:?}", why);
                Err(why)
            },

        }
    }

    pub fn joke(&self) -> Result<String> {
        let mut headers = Headers::new();
        headers.set(UserAgent::new("reqwest"));
        headers.set(Accept(vec![qitem(mime::TEXT_PLAIN)]));

        match self.client.get(DAD_JOKE)
            .headers(headers)
            .send() {
                Ok(mut res) => {
                    res.text()
                },
                Err(why) => {
                    error!("{:?}", why);
                    Err(why)
                },
        }
    }

    pub fn urban<S: Into<String>>(&self, input: S) -> Result<Urban> {
        let mut headers = Headers::new();
        headers.set(UserAgent::new("reqwest"));

        match self.client.get(URBAN)
            .headers(headers)
            .query(&[("term", input.into())])
            .send() {
                Ok(mut res) => {
                    res.json::<Urban>()
                },
                Err(why) => {
                    error!("{:?}", why);
                    Err(why)
                },
        }
    }

    pub fn furry<S: Into<String>>(&self, input: S, count: u32) -> Result<Vec<FurPost>> {
        let mut headers = Headers::new();
        headers.set(UserAgent::new("reqwest"));

        match self.client.get(FURRY)
            .headers(headers)
            .query(&[("tags", input.into()+" order:random"), ("limit", format!("{}",count))])
            .send() {
                Ok(mut res) => {
                    res.json::<Vec<FurPost>>()
                },
                Err(why) => {
                    error!("{:?}", why);
                    Err(why)
                },
        }
    }
}
