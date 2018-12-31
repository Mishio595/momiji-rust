use forecast::{
    ApiClient as DSClient,
    ApiResponse,
    ForecastRequestBuilder,
    Lang,
    Units
};
use geocoding::Opencage;
use kitsu::model::{
    Response,
    Anime,Manga
};
use kitsu::{
    KitsuReqwestRequester,
    Error as KitsuErr
};
use reqwest::header::{
    Headers,
    UserAgent,
    ContentType,
    Accept,
    Authorization,
    qitem
};
use reqwest::mime;
use reqwest::{
    Client,
    Result as ReqwestResult,
    Response as ReqwestResponse
};
use std::collections::HashMap;
use std::env;
use urbandictionary::{
    ReqwestUrbanDictionaryRequester,
    Result as UrbanResult,
    model::Response as UrbanResponse
};

const UA: &str = "momiji-bot";

// Endpoints
const DOG: &str = "https://dog.ceo/api/breeds/image/random";
const CAT: &str = "http://aws.random.cat/meow";
const DAD_JOKE: &str = "https://icanhazdadjoke.com";
const FURRY: &str = "https://e621.net/post/index.json";
const DBOTS: &str = "http://discordbots.org/api/bots";

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
    pub oc_client: Opencage,
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::new();
        let oc_key = env::var("OPENCAGE_KEY").expect("No key for OpenCage in env");
        let oc_client = Opencage::new(oc_key);
        ApiClient {
            client,
            oc_client
        }
    }

    pub fn stats_update(&self, bot_id: u64, server_count: usize) -> ReqwestResult<ReqwestResponse> {
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        headers.set(Authorization(env::var("DBOTS_TOKEN").expect("No DiscordBots.org token in env")));
        let mut data = HashMap::new();
        data.insert("server_count", server_count);

        self.client.post(format!("{}/{}/stats", DBOTS, bot_id).as_str())
            .json(&data)
            .send()
    }

    pub fn dog(&self) -> ReqwestResult<Dog> {
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

    pub fn cat(&self) -> ReqwestResult<Cat> {
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

    pub fn joke(&self) -> ReqwestResult<String> {
        let mut headers = Headers::new();
        headers.set(UserAgent::new(UA));
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

    pub fn urban(&self, input: &str) -> UrbanResult<UrbanResponse> {
        self.client.definitions(input)
    }

    pub fn furry<S: Into<String>>(&self, input: S, count: u32) -> ReqwestResult<Vec<FurPost>> {
        let mut headers = Headers::new();
        headers.set(UserAgent::new(UA));

        match self.client.get(FURRY)
            .headers(headers)
            .query(&[("tags", input.into()+" order:random"), ("limit", count.to_string())])
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

    pub fn anime<S: Into<String>>(&self, input: S) -> Result<Response<Vec<Anime>>, KitsuErr> {
        self.client.search_anime(|f| f.filter("text", input.into().trim()))
    }

    pub fn manga<S: Into<String>>(&self, input: S) -> Result<Response<Vec<Manga>>, KitsuErr> {
        self.client.search_manga(|f| f.filter("text", input.into().trim()))
    }

    pub fn weather(&self, input: &str, units: Units) -> Option<(String, ReqwestResult<ApiResponse>)> {
        match self.oc_client.forward_full(input, &None) {
            Ok(data) => {
                if !data.results.is_empty() {
                    let first = data.results.first().unwrap();
                    let city_info = format!("{}, {}, {}",
                        first.components.get("city").unwrap(),
                        first.components.get("state").unwrap(),
                        first.components.get("country").unwrap()
                    );
                    let ds_key = env::var("DARKSKY_KEY").expect("No DarkSky API Key found in env");
                    let fc_req = Some(ForecastRequestBuilder::new(ds_key.as_str(), *first.geometry.get("lat").unwrap(), *first.geometry.get("lng").unwrap())
                        .lang(Lang::English)
                        .units(units)
                        .build());
                    if let Some(req) = fc_req {
                        let ds_client = DSClient::new(&self.client);
                        match ds_client.get_forecast(req) {
                            Ok(mut res) => {
                               return Some((city_info, res.json::<ApiResponse>()));
                            },
                            Err(why) => { return Some((city_info, Err(why))); },
                        }
                    }
                }
            },
            Err(why) => { trace!("Failed to resolve location: {:?}", why); }
        }
        None
    }
}
