use elm_arch::{Cmd, Program, Sub};

use futures::prelude::*;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::LinesStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

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
            (model, Cmd::Msg(Msg::Send))
        }
        Msg::Send => {
            let input = std::mem::take(&mut model.input);
            let cmd = model
                .ws_sender
                .clone()
                .map(|tx| {
                    Cmd::boxed(async move {
                        tx.send(input).await.unwrap();
                        Msg::Sent
                    })
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

fn websocket(url: Url, mut rx: Receiver<String>) -> impl Stream<Item = String> {
    connect_async(url)
        .map(move |res| {
            let (ws_stream, _) = res.unwrap();
            let (mut sink, stream) = ws_stream.split();

            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    sink.send(Message::Text(msg)).await.unwrap();
                }
            });

            stream.filter_map(|msg| async move {
                if let Ok(Message::Text(text)) = msg {
                    Some(text)
                } else {
                    None
                }
            })
        })
        .flatten_stream()
}

fn on_enter_key() -> impl Stream<Item = String> {
    let stdin = tokio::io::stdin();
    let buf = tokio::io::BufReader::new(stdin);
    LinesStream::new(buf.lines()).map(Result::unwrap)
}

fn subscriptions(mut model: Model) -> (Model, Sub<Msg>) {
    let stdin = on_enter_key().map(Msg::Input);

    let (tx, rx) = channel(64);
    let ws_stream =
        websocket("ws://echo.websocket.events".parse().unwrap(), rx).map(Msg::NewMessage);
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
