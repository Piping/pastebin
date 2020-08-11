#![feature(proc_macro_hygiene, decl_macro)]
use maud::html;
use maud::Markup;

#[macro_use] extern crate rocket;
use rocket::{get, routes};
use rocket::data::{Data};
use rocket::request::Form;
use rocket::response::{content::Plain, Debug};
// use rocket_contrib::serve::StaticFiles;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
// use std::borrow::Cow;

mod paste_id;
use crate::paste_id::PasteID;

#[cfg(test)] mod tests;

const HOST: &str = "https://hjkl.bid";
const ID_LENGTH: usize = 3;

#[post("/", data = "<paste>")]
fn upload(paste: Data) -> Result<String, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = HOST, id = id);
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[derive(Debug, FromForm)]
struct PasteForm {
    pasta: String,
}
#[post("/pasteform", data = "<task>")]
fn uploadbyform(task: Form<PasteForm>) -> Result<String, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = HOST, id = id);
    fs::write(Path::new(&filename), &task.pasta)?;
    Ok(url)
}

#[get("/<id>")]
fn retrieve(id: PasteID<'_>) -> Option<Plain<File>> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).map(|f| Plain(f)).ok()
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
    html! {
    head {
        meta charset="utf-8" {}
        meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" {}
        link href="https://unpkg.com/tailwindcss@^1.0/dist/tailwind.min.css" rel="stylesheet" {}
        title { "你的云端粘贴剪切板" }
    }
    body {
      div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8" {
      div class="max-w-lg w-full" {
        form action="/pasteform" method="post" id="pasteData"
        {
          div class=r"h-full flex flex-col space-y-6 py-6 bg-white shadow-xl
                  h-full border-2 border-dashed border-gray-200"
          {
              textarea placeholder="Paste your text here"
                  style="border: 3px solid green; padding: 5px;" 
                  form="pasteData" name="pasta"
              {}
              button type="submit" form="pasteData"
              { "Create New Paste" }
          }
        }
        div class="bg-white shadow overflow-hidden sm:rounded-lg" {
        div class="px-4 py-5 border-b border-gray-200 sm:px-6" {
          h3 class="text-lg leading-6 font-medium text-gray-900"
          { "自动化用法 " }
          p class="mt-1 max-w-2xl text-sm leading-5 text-gray-500"
          { "SCRIPT USAGE" }
          dl {
            div class="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
             dt class="text-sm leading-5 font-medium text-gray-500"
             { "POST /" }
             dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
             {
               "向网站提交任意数据, 返回带有<id>的网址, 等同于复制"
               br{}
               "例子: curl --data-binary @file.txt https://hjkl.bid"
               br{}
               "accepts raw data in the body of the request and responds with a URL of "
               "a page containing the body's content "
               br{}
               "EXAMPLE: curl --data-binary @file.txt https://hjkl.bid"
             }
            }
            div class="bg-white px-4 py-5 sm:grid sm:grid-cols-5 sm:gap-4 sm:px-6" {
              dt class="text-sm leading-5 font-medium text-gray-500"
              { "GET /<id>" }
              dd class="mt-1 text-sm leading-5 text-gray-900 sm:mt-0 sm:col-span-4"
              {
                "用<id>取回之前复制的内容, 等同于粘贴"
                br{}
                "例子: wget https://hjkl.bid/<id>"
                br{}
                "retrieves the content for the paste with id `<id>`" 
                br{}
                "EXAMPLE: wget https://hjkl.bid/<id>"
              }
            }
      }}}}}
      script {
        r#"
          console.log('Send your Resume!');
        "#
      }
    }}
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, favicon, robots, upload, uploadbyform, retrieve])
        // .mount("/layui", StaticFiles::from("static/layui"))
}

fn main() {
    rocket().launch();
}

