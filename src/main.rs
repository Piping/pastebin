#![feature(proc_macro_hygiene, decl_macro)]
use maud::html;
use maud::Markup;
use rocket::{get, routes};

#[macro_use] extern crate rocket;

mod paste_id;
#[cfg(test)] mod tests;

use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
// use std::borrow::Cow;

use rocket::data::{Data};
use rocket::request::Form;
use rocket::response::{content::Plain, Debug};

use crate::paste_id::PasteID;

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
    let filename = format!("icons/favicon.ico");
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
    form action="/pasteform" method="post" id="pasteData" {
        textarea
            placeholder="Paste your text here"
            style="margin: auto;width: 80%; border: 3px solid green; padding: 5px;" 
            form="pasteData"
            name="pasta" rows="20" cols="80"
            {}
        br {}
        button type="submit" { "Create New Paste" }
    }
    pre {
        "
        USAGE

          POST /

              accepts raw data in the body of the request and responds with a URL of
              a page containing the body's content

              EXAMPLE: curl --data-binary @file.txt https://hjkl.bid

          GET /<id>

              retrieves the content for the paste with id `<id>`

        用法

          POST /

              向网站提交任意数据, 返回带有<id>的网址, 等同于复制

              例子: curl --data-binary @file.txt https://hjkl.bid

          GET /<id>

              用<id>取回之前复制的内容, 等同于粘贴

              例子: wget https://hjkl.bid/<id>
        "
    }
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, favicon, robots, upload, uploadbyform, retrieve])
}

fn main() {
    rocket().launch();
}

