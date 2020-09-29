use elm_arch::{Cmd, Program, Sub};
use futures::prelude::*;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver};

use std::thread;
use std::time::Instant;

struct Model(Instant);

fn init() -> (Model, Cmd<Msg>) {
    (Model(Instant::now()), Cmd::None)
}

enum Msg {
    Tick(Instant),
}

fn update(_: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    match msg {
        Msg::Tick(new_time) => (Model(new_time), Cmd::None),
    }
}

fn tick() -> Receiver<Instant> {
    let (tx, rx) = channel::<Instant>(1);
    let handle = Handle::current();
    thread::spawn(move || loop {
        let mut tx = tx.clone();
        let now = Instant::now();
        handle.spawn(async move {
            tx.send(now).await.unwrap();
        });
        thread::sleep(std::time::Duration::from_secs(1));
    });
    rx
}

fn subscriptions(model: Model) -> (Model, Sub<Msg>) {
    (model, Box::pin(tick().map(Msg::Tick)))
}

fn view(model: &Model) -> String {
    format!("{:?}", model.0)
}

#[tokio::main]
async fn main() {
    Program {
        init,
        view,
        update,
        subscriptions,
    }
    .run()
    .await;
}
