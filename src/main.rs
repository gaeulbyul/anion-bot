extern crate iron;
#[macro_use(router)] extern crate router;
extern crate hyper;
extern crate serde_json;
extern crate rustc_serialize;
extern crate telegram_bot;
extern crate regex;

use std::env;
use std::fmt;
use std::io::Read;
// use std::io::Write;

// chatbot server
// use router::Router;
use iron::prelude::*;
use iron::status;

// anissia api
use hyper::{
    Client,
    Error as HyperError,
    Url,
};
use serde_json::Value as JsonValue;
use rustc_serialize::json as RJson;

// telegram api
use telegram_bot::{
    Api as TelegramBot,
    Update as TUpdate,
    MessageType,
    ParseMode,
};

use regex::Regex;

const VERSION: &'static str = "v0.0.1-alpha";
const API_URL: &'static str = "https://anion.herokuapp.com/api/";

#[derive(Debug)]
struct Ani {
    title: String,
    id: i32,
    time: String,
}
impl fmt::Display for Ani {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "Ani#{} ", &self.id)
        write!(f, "Ani('{}')#{} ", &self.title, &self.id)
    }
}

#[derive(Debug)]
struct Cap {
    author: String,
    episode: String,
    url: String,
}
impl fmt::Display for Cap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "Cap#{} ", &self.id)
        write!(f, "Cap(ep. {}, by. '{}') {} ", &self.episode, &self.author, &self.url)
    }
}

fn make_querystring_from_weekday(w: &str) -> Option<String> {
    if w == "오" {
        return Some(String::new());
    }
    let weekday_as_number = match w {
        "일" => 0,
        "월" => 1,
        "화" => 2,
        "수" => 3,
        "목" => 4,
        "금" => 5,
        "토" => 6,
        //"외" => 7,
        //"신" => 8,
        _ => -1,
    };
    if weekday_as_number > 0 {
        Some(format!("weekday={}", weekday_as_number))
    } else {
        None
    }
}

fn fetch_anilist(querystring: &str) -> Result<Vec<Ani>, HyperError> {
    let url = format!("{}anilist?{}", API_URL, querystring);
    // ---------------------
    println!("DBG: url= {}", url);
    // ---------------------
    let client = Client::new();
    let request = client.get(&url);
    let mut response = try!(request.send());
    let mut body = String::new();
    response.read_to_string(&mut body).expect("서버 요청을 읽지 못했습니다.");
    let read_json: JsonValue = serde_json::from_str(&mut body).unwrap();
    let anilist_jv: &JsonValue = read_json.as_object().unwrap().get("result").unwrap();
    let anilist_ja = anilist_jv.as_array().unwrap();
    let anilist = anilist_ja.into_iter().map(|ani| {
        let ao = ani.as_object().unwrap();
        let ani_time = ao.get("time").unwrap().as_str().unwrap().to_string();
        let (hour, minute) = ani_time.split_at(2);
        return Ani {
            title: ao.get("title").unwrap().as_str().unwrap().to_string(),
            id: ao.get("id").unwrap().as_i64().unwrap() as i32,
            time: format!("{}:{}", &hour, &minute),
        };
    }).collect();
    Ok(anilist)
}

fn fetch_anicaps(ani_id: i32) -> Result<Vec<Cap>, HyperError> {
    let url = format!("{}cap?id={}", API_URL, ani_id);
    let client = Client::new();
    let request = client.get(&url);
    let mut response = try!(request.send());
    let mut body = String::new();
    response.read_to_string(&mut body).expect("서버 요청을 읽지 못했습니다.");
    let read_json: JsonValue = serde_json::from_str(&mut body).unwrap();
    let caplist = read_json.as_array().unwrap().into_iter().map(|cap| {
        let co = cap.as_object().unwrap();
        return Cap {
            author: co.get("name").unwrap().as_str().unwrap().to_string(),
            episode: co.get("episode").unwrap().as_str().unwrap().to_string(),
            url: co.get("url").unwrap().as_str().unwrap().to_string(),
        };
    }).collect();
    Ok(caplist)
}

fn bot_intro() -> String {
    format!("Ani-ON 봇 {}
**아직 불안정한 버전입니다. 예상치 못한 출력이 나오거나 (오류로 인해) 반응이 없을 수 있습니다.**
Ani-ON 봇은 애니편성표 봇입니다. 데이터는 [애니시아](http://anissia.net)에서 가져옵니다.
모바일 웹용 애니편성표: [Ani-ON](https://anion.herokuapp.com)
----------
사용법:
`/list`: 오늘의 목록
`/list <요일>`: 요일별 목록
=> 요일=일월화수목금토
(비정기, 신작 목록 미구현)
`/cap <ID>`: 자막 목록
=> ID=애니별 고유번호 (#xxxx)
`/help`: 이 도움말 메시지 출력", VERSION)
}

fn bot_list_today() -> String {
    let anilist = fetch_anilist("").unwrap();
    let mut response = String::from("오늘의 애니 목록:\n-----\n");
    for ani in anilist {
        response += &format!("{} {} (#{})\n", &ani.time, &ani.title, &ani.id);
    }
    response
}

fn bot_list_weekday(weekday: &str) -> String {
    let qs = make_querystring_from_weekday(weekday).unwrap_or("".to_string());
    let anilist = fetch_anilist(&qs).unwrap();
    let mut response = String::new();
    response += &format!("애니 목록: ({})\n-----\n", weekday);
    for ani in anilist {
        response += &format!("{} {} (#{})\n", &ani.time, &ani.title, &ani.id);
    }
    response
}

// TODO: Wrap with Result<>
fn bot_cap_byid(ani_id: i32) -> String {
    if let Ok(caplist) = fetch_anicaps(ani_id) {
        let mut response = String::new();
        response += &format!("자막 목록: (#{})\n-----\n", ani_id);
        if caplist.len() == 0 {
            return "(등록된 자막이 없습니다.)".to_string();
        }
        for cap in caplist {
            response += &format!("{}화 by {} ( {} )\n", &cap.episode, &cap.author, &cap.url);
        }
        return response;
    } else {
        return "해당하는 애니를 찾을 수 없습니다.".to_string();
    }
}

/* maybe?
fn line_handler(token: &str, req: &mut Request) -> IronResult<Response> {
    ...
}
*/

fn telegram_handler(bot: &TelegramBot, req: &mut Request) -> IronResult<Response> {
    println!("processing...");
    let command_list_pattern = Regex::new(r"/list (?P<weekday>[오일월화수목금토])").unwrap();
    let command_cap_pattern = Regex::new(r"/cap #?(?P<id>\d+)").unwrap();
    let mut body = String::new();
    req.body.read_to_string(&mut body).expect("can't read request??");
    // println!("raw json:\n\n{}\n\n-------------------------", &body);
    let update: TUpdate = RJson::decode(&body).unwrap();
    if let Some(m) = update.message {
        let id = m.chat.id();
        let mut response = String::new();
        if let MessageType::Text(text) = m.msg {
            let mut markdown: Option<ParseMode> = None;
            if text == "/start" || text == "/help" {
                response = bot_intro();
                markdown = Some(ParseMode::Markdown);
            }
            else if text == "/list" {
                response = bot_list_today();
            } else if let Some(pmatch) = command_list_pattern.captures(&text) {
                let weekday = pmatch.name("weekday").unwrap();
                response = bot_list_weekday(&weekday);
            } else if let Some(pmatch) = command_cap_pattern.captures(&text) {
                let id_s = pmatch.name("id").unwrap();
                if let Ok(id) = id_s.parse::<i32>() {
                    response = bot_cap_byid(id);
                } else {
                    response = "인식할 수 없는 고유번호(ID)입니다.".to_string();
                }
            } else {
                response = "모르는 명령어?".to_string();
            }
            println!("will response: {}", &response);
            bot.send_message(
                id, response,
                markdown, None, None, None).unwrap();
            Ok(Response::with((status::Ok, "ok")))
        } else {
            response = "모르는 명령어?".to_string();
            println!("will response... {}", &response);
            Ok(Response::with((status::Ok, "ok")))
        }
    } else {
        println!("will response FAIL!");
        Ok(Response::with((status::BadRequest, "fail")))
    }
}

fn main() {
    println!("bot start!");
    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("5000".to_string());
    let telegram_token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
    let bot = TelegramBot::from_token(&telegram_token).unwrap();
    let webhook_url = Url::parse(format!("https://{}/telegram-bot~{}", host, telegram_token).as_str()).unwrap();
    bot.set_webhook(Some(webhook_url)).unwrap();
    let router = router!(
        post format!("/telegram-bot~{}", telegram_token) => move |req: &mut Request| -> IronResult<Response> {
            telegram_handler(&bot, req)
        },
    );
    Iron::new(router).http(format!("0.0.0.0:{}", &port).as_str()).expect("server fail??");
    /*
    server.handle(move |req: Request, res: Response| {
        telegram_handler(&telegram_token, &bot, req, res);
    }).expect("f");
    */
    /*
    server.handle(|req: Request, mut res: Response| {
        line_handler(&line_token, req, res);
    }).expect("f");
    */
}
