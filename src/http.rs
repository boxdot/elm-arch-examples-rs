use elm_arch::{Cmd, Program, Sub};

use anyhow::Error;
use futures::prelude::*;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver};

use std::io::{self, BufRead};
use std::thread;

#[derive(Debug)]
struct Model {
    topic: String,
    gif_url: String,
}

fn init() -> (Model, Cmd<Msg>) {
    (
        Model {
            topic: "cats".into(),
            gif_url: "waiting.gif".into(),
        },
        Cmd::None,
    )
}

#[derive(Debug)]
enum Msg {
    MorePlease,
    NewGif(Result<String, Error>),
}

fn update(mut model: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    match msg {
        Msg::MorePlease => {
            let topic = model.topic.clone();
            (
                model,
                Cmd::Cmd(Box::pin(get_random_gif(topic).map(Msg::NewGif))),
            )
        }
        Msg::NewGif(res) => {
            match res {
                Ok(gif_url) => {
                    model.gif_url = gif_url;
                }
                Err(err) => {
                    eprintln!("Failed to get gif url: {:?}", err);
                }
            };
            (model, Cmd::None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct Response {
    data: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    id: String,
    slug: String,
    url: String,
}

async fn get_random_gif(topic: String) -> Result<String, Error> {
    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let uri = format!(
        "https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}",
        topic
    )
    .parse()
    .unwrap();
    let mut response = client.get(uri).await?;

    let body = response.body_mut();
    let mut output = Vec::new();

    while let Some(chunk) = body.next().await {
        let bytes = chunk?;
        output.extend(&bytes[..]);
    }

    let value: Response = serde_json::from_slice(&output)?;
    Ok(value.data.url)
}

fn view(model: &Model) -> String {
    format!("{:?}", model)
}

fn on_enter_key(handle: Handle) -> Receiver<String> {
    let (tx, rx) = channel::<String>(1);
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let mut tx = tx.clone();
            handle.spawn(async move {
                tx.send(line.unwrap()).await.unwrap();
            });
        }
    });
    rx
}

fn subscriptions(model: Model, handle: Handle) -> (Model, Sub<Msg>) {
    (
        model,
        Box::pin(on_enter_key(handle).map(|_| Msg::MorePlease)),
    )
}

#[tokio::main]
async fn main() {
    println!("Press any key...");
    Program {
        init,
        view,
        update,
        subscriptions,
    }
    .run()
    .await;
}
