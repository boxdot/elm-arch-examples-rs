use std::time::Duration;

use elm_arch::{Cmd, Program, Sub};
use futures::prelude::*;
use tokio::time::Instant;
use tokio_stream::wrappers::IntervalStream;

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

fn tick() -> impl Stream<Item = Instant> {
    let interval = tokio::time::interval(Duration::from_secs(1));
    IntervalStream::new(interval)
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
