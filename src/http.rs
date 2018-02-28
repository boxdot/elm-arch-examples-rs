extern crate elm_arch;
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

use elm_arch::{Cmd, CmdWithHandle, Program, Sub};

use futures::prelude::*;
use futures::sync::mpsc::{channel, Receiver};
use tokio_core::reactor::Handle;
use failure::Error;

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
            (model, Cmd::with_handle(GetRandomGif(topic)))
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

struct GetRandomGif(String);

impl CmdWithHandle<Msg> for GetRandomGif {
    fn call(self: Box<Self>, handle: Handle) -> Box<Future<Item = Msg, Error = ()>> {
        let client = hyper::Client::configure()
            .connector(hyper_tls::HttpsConnector::new(1, &handle).unwrap())
            .build(&handle);

        let uri = format!(
            "https://api.giphsy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}",
            self.0
        ).parse()
            .unwrap();

        Box::new(
            client
                .get(uri)
                .map_err(Error::from)
                .and_then(|resp| resp.body().concat2().map_err(Error::from))
                .and_then(|chunk| Ok(serde_json::from_slice(&chunk)?))
                .map(|value: Response| value.data.url)
                .then(|res| {
                    Ok(match res {
                        Ok(gif_url) => Msg::NewGif(Ok(gif_url)),
                        Err(e) => Msg::NewGif(Err(e)),
                    })
                })
                .map_err(|_: Error| ()),
        )
    }
}

fn view(model: &Model) -> String {
    format!("{:?}", model)
}

fn on_enter_key() -> Receiver<String> {
    let (tx, rx) = channel::<String>(1);
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            tx.clone().send(line.unwrap()).wait().unwrap();
        }
    });
    rx
}

fn subscriptions(model: Model, _: Handle) -> (Model, Sub<Msg>) {
    (model, Box::new(on_enter_key().map(|_| Msg::MorePlease)))
}

fn main() {
    println!("Press any key...");
    Program {
        init,
        view,
        update,
        subscriptions,
    }.run()
}
