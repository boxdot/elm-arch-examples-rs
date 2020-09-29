use elm_arch::{BoxStream, Cmd, Program, Sub};

use futures::prelude::*;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use std::io::{self, BufRead};
use std::thread;

#[derive(Debug, Default)]
struct Model {
    counter: usize,
    input: String,
    ws_sender: Option<Sender<String>>,
    messages: Vec<String>,
}

fn init() -> (Model, Cmd<Msg>) {
    (Model::default(), Cmd::None)
}

#[derive(Debug)]
enum Msg {
    Input(String),
    Send,
    Sent,
    NewMessage(String),
}

fn update(mut model: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    match msg {
        Msg::Input(new_input) => {
            model.input = new_input;
            (model, Cmd::new(|| Msg::Send))
        }
        Msg::Send => {
            let mut input = String::new();
            std::mem::swap(&mut model.input, &mut input);
            let cmd = model
                .ws_sender
                .clone()
                .map(|mut tx| {
                    Cmd::Cmd(Box::pin(async move {
                        tx.send(input).await.unwrap();
                        Msg::Sent
                    }))
                })
                .unwrap_or(Cmd::None);
            (model, cmd)
        }
        Msg::NewMessage(msg) => {
            model.counter += 1;
            model.messages.push(msg);
            (model, Cmd::None)
        }
        Msg::Sent => (model, Cmd::None),
    }
}

fn websocket(url: &str, rx: Receiver<String>) -> BoxStream<String> {
    let ws_stream = connect_async(Url::parse(url).unwrap())
        .map(move |res| {
            let (sink, stream) = res.unwrap().0.split();
            tokio::spawn(async move {
                rx.map(|msg| Ok(Message::Text(msg)))
                    .forward(sink)
                    .await
                    .unwrap();
            });

            stream.filter_map(|msg| {
                future::ready(match msg {
                    Ok(Message::Text(text)) => Some(text),
                    _ => None,
                })
            })
        })
        .flatten_stream();
    Box::pin(ws_stream)
}

fn on_enter_key() -> Receiver<String> {
    let (tx, rx) = channel::<String>(1);
    let handle = Handle::current();
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

fn subscriptions(mut model: Model) -> (Model, Sub<Msg>) {
    let stdin = on_enter_key().map(Msg::Input);

    let (tx, rx) = channel(64);
    let ws_stream = websocket("ws://echo.websocket.org", rx).map(Msg::NewMessage);
    model.ws_sender = Some(tx);

    (model, Box::pin(stream::select(stdin, ws_stream)))
}

fn view(model: &Model) -> String {
    format!(
        r#"
[{:#3}]---------------------------------------------------------
Messages:
{}
--------------------------------------------------------------"#,
        model.counter,
        model.messages.join("\n")
    )
}

#[tokio::main]
async fn main() {
    println!("Enter a message and press enter...");
    Program {
        init,
        view,
        update,
        subscriptions,
    }
    .run()
    .await;
}
