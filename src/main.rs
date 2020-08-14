#![feature(proc_macro_hygiene, decl_macro)]
use maud::html;
use maud::Markup;

#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
use rocket::{get, routes};
use rocket::data::Data;
use rocket::request::{self, Form, Request, FromRequest, FromParam};
use rocket::response::{content::Plain, Debug, Redirect};
use rocket::Outcome;
use rocket::http::RawStr;

use std::io;
use std::fs;
use std::fmt;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
// use std::borrow::Cow;

mod paste_id;
use crate::paste_id::PasteID;

#[cfg(test)] mod tests;

lazy_static!{
    static ref TEXT: HashMap<ServerAcceptLangauge, HashMap<&'static str, &'static str>> = [
        (ServerAcceptLangauge::SimpliedChinese,
         [
            ("lang-id", "中文"),
            ("site-title", "复制红 - 分享你的云剪切板"),
            ("paste-button", "新建粘贴"),
            ("help-h1", "使用说明"),
            ("help-h2", "为什么用copy.red?"),
            ("help-msg1-h1", "目的"),
            ("help-msg1-h2", "方便不同设备之间的复制拷贝消息，手机电脑服务器均可，有无图形界面均可"),
            ("help-msg2-h1", "使用"),
            ("help-msg2-h2", "粘贴数据至文本框，点击按钮，得到可以分享的在其他设备使用的的链接"),
            ("info-h1", "自动化用法"),
            ("info-h2", "脚本参考"),
            ("post-api-doc", "向网站提交任意数据, 返回带有<id>的网址, 等同于复制"),
            ("get-api-doc", "用<id>取回之前复制的内容, 等同于粘贴"),
         ].iter().copied().collect()
        ),
        (ServerAcceptLangauge::Japananese,
         [
            ("lang-id", "日文"),
            ("site-title", "复制红 - 分享你的云剪切板"),
            ("paste-button", "新建粘贴"),
            ("help-h1", "USAGE"),
            ("help-h2", "Why copy.red?"),
            ("help-msg1-h1", "GOAL"),
            ("help-msg1-h2", "Share your data between devices, e.g phone, laptop, desktop, server etc."),
            ("help-msg2-h1", "USE"),
            ("help-msg2-h2", "Paste your data into textbox, click new paste, get the link you can share."),
            ("info-h1", "Automation"),
            ("info-h2", "Script usage"),
            ("get-api-doc", "retrieves the content for the paste with id `<id>`"),
            ("post-api-doc", "accepts raw data in the body of the request and responds with a URL of a page containing the body's content "),
         ].iter().copied().collect()
        ),
        (ServerAcceptLangauge::English,
         [
            ("lang-id", "En"),
            ("site-title", "copy.red - your sharable cloud clipboard"),
            ("paste-button", "Create New Paste"),
            ("help-h1", "USAGE"),
            ("help-h2", "Why copy.red?"),
            ("help-msg1-h1", "GOAL"),
            ("help-msg1-h2", "Share your data between devices, e.g phone, laptop, desktop, server etc."),
            ("help-msg2-h1", "USE"),
            ("help-msg2-h2", "Paste your data into textbox, click new paste, get the link you can share."),
            ("info-h1", "Automation"),
            ("info-h2", "Script usage"),
            ("get-api-doc", "retrieves the content for the paste with id `<id>`"),
            ("post-api-doc", "accepts raw data in the body of the request and responds with a URL of a page containing the body's content "),
         ].iter().copied().collect()
        ),
    ].iter().cloned().collect();
}

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
enum ServerAcceptLangauge {
    SimpliedChinese,
    Japananese,
    English,
}
impl<'a, 'r> FromRequest<'a, 'r> for ServerAcceptLangauge {
    type Error = &'r RawStr;
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let first_lang:  Option<&str> = request.headers().get("accept-language").next();
        match first_lang {
            // TODO process raw string here
            Some(lang) => {
                if lang.contains("zh") {
                    Outcome::Success(ServerAcceptLangauge::SimpliedChinese)
                } else if lang.contains("jp") {
                    Outcome::Success(ServerAcceptLangauge::Japananese)
                } else {
                    Outcome::Success(ServerAcceptLangauge::English)
                }
            }
            None => {
                Outcome::Success(ServerAcceptLangauge::English)
            }
        }
    }
}
impl<'r> FromParam<'r> for ServerAcceptLangauge {
    type Error = &'r RawStr;
    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        match &param[..] {
            "zh" => Ok(ServerAcceptLangauge::SimpliedChinese),
            "jp" => Ok(ServerAcceptLangauge::Japananese),
            "en" => Ok(ServerAcceptLangauge::English),
            _ => Ok(ServerAcceptLangauge::English),
        }
    }
}
impl fmt::Display for ServerAcceptLangauge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
              ServerAcceptLangauge::SimpliedChinese => write!(f, "zh")
            , ServerAcceptLangauge::Japananese => write!(f, "jp")
            , ServerAcceptLangauge::English => write!(f, "en")
        }
    }
}

const HOST: &str = "https://copy.red";
const ID_LENGTH: usize = 3;

#[post("/api/paste", data = "<paste>")]
fn upload_api(paste: Data) -> Result<String, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/api/{id}\n", host = HOST, id = id);
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[derive(Debug, FromForm)]
struct PasteForm {
    paste_text: String,
}
#[post("/<lang>", data = "<task>")]
fn upload(lang: ServerAcceptLangauge, task: Form<PasteForm>) -> Result<Redirect, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    fs::write(Path::new(&filename), &task.paste_text)?;
    Ok(Redirect::to(format!("/{lang}/{id}", lang=lang, id = id)))
}

#[get("/api/<id>", rank=1)]
fn retrieve_api(id: PasteID<'_>) -> Option<Plain<File>> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).map(|f| Plain(f)).ok()
}

#[get("/<lang>/<id>")]
fn retrieve(id: PasteID<'_>, lang: ServerAcceptLangauge) -> Option<Markup> {
    let url = format!("{host}/{lang}/{id}\n", host = HOST, lang=lang, id = id);
    let filename = format!("upload/{id}", id = id);
    match fs::read_to_string(&filename) {
        Ok(f) => Some(default_view(Some(url), Some(f), lang)),
        Err(..) => Some(default_view(None, None, lang))
    }
}

#[get("/favicon.ico")]
fn favicon() -> Option<Plain<File>> {
    let filename = format!("static/icons/favicon.ico");
    File::open(&filename).map(|f| Plain(f)).ok()
}

#[get("/robots.txt")]
fn robots() -> &'static str {
    "
    User-agent: *
    Allow: /
    "
}

#[get("/<lang>")]
fn localized_index(lang: ServerAcceptLangauge) -> Markup {
    let url = None;
    let file = None;
    default_view(url,file,lang)
}

#[get("/")]
fn index(lang:ServerAcceptLangauge) -> Redirect {
    Redirect::to(format!("/{}",lang))
}

fn language_switch_link(url: &Option<String>, lang: &ServerAcceptLangauge) -> String {
    match url {
        Some(url) => {
            match url.rsplit("/").nth(0) {
                Some(paste) => format!("/{}/{}", lang, paste),
                None => format!("/{}", lang),
            }
        },
        None => format!("/{}", lang),
    }
}

fn default_view(url: Option<String>, file: Option<String>, lang: ServerAcceptLangauge) -> Markup {
  html! {
    head {
        meta charset="utf-8" {}
        meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" {}
        link href="https://unpkg.com/tailwindcss@^1.0/dist/tailwind.min.css" rel="stylesheet" {}
        title { (TEXT[&lang]["site-title"]) }
    }
    body {
      div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8" {
      div class="max-w-lg w-full" {
        ul id="language_switcher" class="flex leading-3 divide-x-2 divide-gray-400 mb-2 text-sm" {
          li class="px-2 pl-0" {
              a href=(language_switch_link(&url,&ServerAcceptLangauge::SimpliedChinese))
                title="使用说明"
              { "中文" }
          }
          li class="px-2 " {
              a href=(language_switch_link(&url,&ServerAcceptLangauge::Japananese))
                title="日文"
              { "日文 "}
          }
          li class="active px-2" {
              a href=(language_switch_link(&url,&ServerAcceptLangauge::English))
                title="Use English Text"
              { "En "}
          }
        }
        div id="visitor_data" class="leading-3 text-gray-500 text-xs"
        { "1,664 unique visitors (Aug)" }
        form action=(format!("/{}",lang)) method="post" id="pasteData"
        {
          div class=r"flex flex-col space-y-6 py-6 bg-white shadow-xl border-2 border-dashed border-gray-200"
          {
              textarea class=r"border-4 border-red-300 border-opacity-75
                               focus:border-red-500 hover:border-red-500 p-5"
                  placeholder="Paste your text here"
                  form="pasteData" name="paste_text"
              { ( file.unwrap_or("".into()) ) }
              button type="submit" form="pasteData"
              { (TEXT[&lang]["paste-button"]) }
          }
        }
        @match url {
          Some(url) => {
            div class="text-center py-4 px-4" {
            div class=r"p-2 bg-green-400 items-center text-indigo-100
                        leading-none rounded-full flex" role="alert" {
                span class="flex rounded-full bg-green-500 uppercase px-2 py-1 text-xs font-bold mr-3"
                { "PASTED" }
                br;
                span id="copy2board" class="font-semibold mr-2 text-left flex-auto text-green-800"
                { (url) }
                i class="hover:text-teal-600 text-indigo-100" onclick=r#"
                    (function copyToClipboard() {
                      var aux = document.createElement("input");
                      aux.setAttribute("value", document.getElementById("copy2board").innerHTML);
                      document.body.appendChild(aux);
                      aux.select();
                      document.execCommand("copy");
                      document.body.removeChild(aux);
                      document.getElementById("copy2boardIcon").classList.add("animate-bounce");
                      setTimeout(function(){
                        document.getElementById("copy2boardIcon").classList.remove("animate-bounce");
                      }, 300);
                    })();
                    "#
                {
                  svg id="copy2boardIcon" class="h-8 w-8 "  fill="none" viewBox="0 0 24 24" stroke="currentColor"
                  {
                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                         d=r"M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002
                             2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2
                             2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3";
                  }
                }
            }}
          },
          None => {}
        }
        div class="bg-white shadow overflow-hidden sm:rounded-lg" {
        div class="px-4 py-5 border-b border-gray-200 sm:px-6" {
          h3 class="text-lg leading-6 font-medium text-gray-900"
          { (TEXT[&lang]["help-h1"]) }
          p class="mt-1 max-w-2xl text-sm leading-5 text-gray-500"
          { (TEXT[&lang]["help-h2"]) }
          dl {
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { (TEXT[&lang]["help-msg1-h1"]) }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             { (TEXT[&lang]["help-msg1-h2"]) }
            }
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { (TEXT[&lang]["help-msg2-h1"]) }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             { (TEXT[&lang]["help-msg2-h2"]) }
            }
          }
          h3 class="text-lg leading-6 font-medium text-gray-900"
          { (TEXT[&lang]["info-h1"]) }
          p class="mt-1 max-w-2xl text-sm leading-5 text-gray-500"
          { (TEXT[&lang]["info-h2"]) }
          dl {
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { "POST /api/paste" }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             { (TEXT[&lang]["get-api-doc"]) br; "curl --data-binary @file.txt https://copy.red/api/paste" }
            }
            div class="bg-white px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
              dt class="text-sm leading-5 font-medium text-gray-500"
              { "GET /api/<id>" }
              dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
              { (TEXT[&lang]["get-api-doc"]) br; "curl https://copy.red/api/<id>" }
            }
          }
        }}
        div class="flex justify-center" {
        a href="https://github.com/piping/pastebin" alt="Github repo link for this page"
        {
          svg class="h-12 w-12 p-2 mt-2"
              xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
              aria-hidden="true" focusable="false" width="1em" height="1em"
              style="-ms-transform: rotate(360deg); -webkit-transform: rotate(360deg); transform: rotate(360deg);"
              preserveAspectRatio="xMidYMid meet" viewBox="0 0 64 64" {
                  path fill="#626262" d=r"M32 0C14 0 0 14 0 32c0 21 19 30 22 30c2
                      0 2-1 2-2v-5c-7 2-10-2-11-5c0 0 0-1-2-3c-1-1-5-3-1-3c3 0
                      5 4 5 4c3 4 7 3 9 2c0-2 2-4 2-4c-8-1-14-4-14-15c0-4 1-7
                      3-9c0 0-2-4 0-9c0 0 5 0 9 4c3-2 13-2 16 0c4-4 9-4 9-4c2 7
                      0 9 0 9c2 2 3 5 3 9c0 11-7 14-14 15c1 1 2 3 2 6v8c0 1 0 2
                      2 2c3 0 22-9 22-30C64 14 50 0 32 0z";
              }
        }
        }
      }}
      script {
        r#"
          console.log('Send your Resume!');
        "#
      }
      ( development_script_tag() )
  }}
}

#[cfg(debug_assertions)]
fn development_script_tag() -> Markup {
    html! {
      script src="http://127.0.0.3:35729/livereload.js" {}
    }
}

#[cfg(not(debug_assertions))]
fn development_script_tag() -> Markup {
    html! { }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, localized_index, favicon, instantclick,
            robots, upload, upload_api, retrieve, retrieve_api])
}

fn main() {
    rocket().launch();
}

