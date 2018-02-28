extern crate elm_arch;
extern crate failure;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate url;

use std::io::{self, BufRead};
use std::thread;

use failure::Error;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Sink, Stream};
use tokio_core::reactor::Handle;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use elm_arch::{Cmd, Program, Sub};

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
                .map(|tx| {
                    Cmd::new(move || {
                        tx.send(input).wait().unwrap();
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

fn websocket(
    url: &str,
    rx: Receiver<String>,
    handle: Handle,
) -> Box<Stream<Item = String, Error = ()>> {
    let url = url::Url::parse(url).unwrap();
    let rx = rx.map_err(|_| panic!("error on rx"));
    let stream = connect_async(url, handle.remote().clone())
        .map(move |(ws_stream, _)| {
            let (sink, stream) = ws_stream.split();

            let sink = sink.sink_map_err(|_| ());
            let rx = rx.map(Message::Text)
                .map_err(|_| ())
                .forward(sink)
                .map(|_| ())
                .then(|_| Ok(()));
            handle.spawn(rx);

            stream
                .filter_map(|msg| match msg {
                    Message::Text(text) => Some(text),
                    _ => None,
                })
                .map_err(Error::from)
        })
        .map_err(Error::from)
        .flatten_stream()
        .map_err(|e| {
            eprintln!("Error during the websocket handshake occured: {}", e);
        });
    Box::new(stream)
}

fn on_enter_key() -> Receiver<String> {
    let (tx, rx) = channel::<String>(0);
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            tx.clone().send(line.unwrap()).wait().unwrap();
        }
    });
    rx
}

fn subscriptions(mut model: Model, handle: Handle) -> (Model, Sub<Msg>) {
    let stdin = on_enter_key().map(Msg::Input);

    let (tx, rx) = channel(0);
    let ws_stream = websocket("ws://echo.websocket.org", rx, handle).map(Msg::NewMessage);
    model.ws_sender = Some(tx);

    (model, Box::new(stdin.select(ws_stream)))
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

fn main() {
    println!("Enter a message and press enter...");
    Program {
        init,
        view,
        update,
        subscriptions,
    }.run()
}
