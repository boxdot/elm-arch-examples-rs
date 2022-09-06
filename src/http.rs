use anyhow::{bail, Error};
use elm_arch::{Cmd, Program, Sub};
use futures::{FutureExt, Stream};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;

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
            (model, Cmd::boxed(get_random_gif(topic).map(Msg::NewGif)))
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
    url: String,
}

async fn get_random_gif(topic: String) -> Result<String, Error> {
    let https = HttpsConnector::new();
    let client = hyper::client::Client::builder().build::<_, hyper::Body>(https);
    let uri = format!("https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={topic}",)
        .parse()
        .unwrap();
    let response = client.get(uri).await?;
    let status = response.status();
    let data = hyper::body::to_bytes(response.into_body()).await?;
    if !status.is_success() {
        bail!(
            "request failed with body: {}",
            String::from_utf8_lossy(&data)
        );
    }
    let value: Response = serde_json::from_slice(&data)?;
    Ok(value.data.url)
}

fn view(model: &Model) -> String {
    format!("{model:?}")
}

fn on_enter_key() -> impl Stream<Item = String> {
    let stdin = tokio::io::stdin();
    let buf = tokio::io::BufReader::new(stdin);
    LinesStream::new(buf.lines()).map(Result::unwrap)
}

fn subscriptions(model: Model) -> (Model, Sub<Msg>) {
    (model, Box::pin(on_enter_key().map(|_| Msg::MorePlease)))
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
