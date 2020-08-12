#![feature(proc_macro_hygiene, decl_macro)]
use maud::html;
use maud::Markup;

#[macro_use] extern crate rocket;
use rocket::{get, routes};
use rocket::data::Data;
use rocket::request::Form;
use rocket::response::{content::Plain, Debug};
use rocket::response::Redirect;
// use rocket_contrib::serve::StaticFiles;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
// use std::borrow::Cow;

mod paste_id;
use crate::paste_id::PasteID;

#[cfg(test)] mod tests;

const HOST: &str = "https://copy.red";
const ID_LENGTH: usize = 3;

#[post("/api/paste", data = "<paste>")]
fn upload_api(paste: Data) -> Result<String, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = HOST, id = id);
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[derive(Debug, FromForm)]
struct PasteForm {
    paste_text: String,
}
#[post("/", data = "<task>")]
fn upload(task: Form<PasteForm>) -> Result<Redirect, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    fs::write(Path::new(&filename), &task.paste_text)?;
    Ok(Redirect::to(format!("/{id}", id = id)))
}

#[get("/api/<id>")]
fn retrieve_api(id: PasteID<'_>) -> Option<Plain<File>> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).map(|f| Plain(f)).ok()
}

#[get("/<id>")]
fn retrieve(id: PasteID<'_>) -> Option<Markup> {
    let url = format!("{host}/{id}\n", host = HOST, id = id);
    let filename = format!("upload/{id}", id = id);
    match fs::read_to_string(&filename) {
        Ok(f) => Some(default_view(Some(url), Some(f))),
        Err(..) => Some(default_view(None, None))
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

#[get("/")]
fn index() -> Markup {
    let url = None;
    let file = None;
    default_view(url,file)
}

fn default_view(url: Option<String>, file: Option<String>) -> Markup {
  html! {
    head {
        meta charset="utf-8" {}
        meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" {}
        link href="https://unpkg.com/tailwindcss@^1.0/dist/tailwind.min.css" rel="stylesheet" {}
        title { "复制红 - 分享你的云剪切板" }
    }
    body {
      div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8" {
      div class="max-w-lg w-full" {
        form action="/" method="post" id="pasteData"
        {
          div class=r"h-full flex flex-col space-y-6 py-6 bg-white shadow-xl
                  h-full border-2 border-dashed border-gray-200"
          {
              textarea class="border-4 border-red-300 border-opacity-75 focus:border-red-500 hover:border-red-500 p-5"
                  placeholder="Paste your text here"
                  form="pasteData" name="paste_text"
              { ( file.unwrap_or("".into()) ) }
              button type="submit" form="pasteData"
              { "Create New Paste" }
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
          { "使用说明 " }
          p class="mt-1 max-w-2xl text-sm leading-5 text-gray-500"
          { "why copy.red? " }
          dl {
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { "目的" }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             {
               "方便不同设备之间的复制拷贝消息，手机电脑服务器均可，有无图形界面均可"
             }
            }
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { "使用" }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             {
               "粘贴数据至文本框，点击按钮，得到可以分享的在其他设备使用的的链接"
             }
            }
          }
          h3 class="text-lg leading-6 font-medium text-gray-900"
          { "自动化用法 " }
          p class="mt-1 max-w-2xl text-sm leading-5 text-gray-500"
          { "SCRIPT USAGE" }
          dl {
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { "POST /api/paste" }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             {
               "向网站提交任意数据, 返回带有<id>的网址, 等同于复制"
               br{}
               "accepts raw data in the body of the request and responds with a URL of "
               "a page containing the body's content "
               br{}
               "EXAMPLE / 示例: curl --data-binary @file.txt https://copy.red/api/paste"
             }
            }
            div class="bg-white px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
              dt class="text-sm leading-5 font-medium text-gray-500"
              { "GET /api/<id>" }
              dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
              {
                "用<id>取回之前复制的内容, 等同于粘贴"
                br{}
                "retrieves the content for the paste with id `<id>`" 
                br{}
                "EXAMPLE / 示例: wget https://copy.red/api/<id>"
              }
            }
          }
        }}
      }}
      script {
        r#"
          console.log('Send your Resume!');
        "#
      }
  }}
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, favicon, robots, upload, upload_api, retrieve, retrieve_api])
}

fn main() {
    rocket().launch();
}

