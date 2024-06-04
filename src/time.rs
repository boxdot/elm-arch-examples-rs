//! This example shows a cancellable subscription and a model managing its state.

use std::future::ready;
use std::time::Duration;

use elm_arch::{Cmd, Program, Sub};
use futures::{
    stream::{self, StreamExt},
    FutureExt, Stream,
};
use tokio::io::AsyncBufReadExt;
use tokio::time::Instant;
use tokio_stream::wrappers::{IntervalStream, LinesStream};
use tokio_util::sync::{CancellationToken, DropGuard};

#[derive(Default)]
struct Model {
    last_tick: Option<Instant>,
    /// when Some, the clock is ticking; stops the clock on drop
    cancel: Option<DropGuard>,
}

fn init() -> (Model, Cmd<Msg>) {
    (Model::default(), Cmd::None)
}

enum Msg {
    /// Enabled or disable the ticking clock
    Toggle,
    /// A tick of the clock
    Tick(Instant),
}

fn update(mut model: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    match msg {
        Msg::Toggle => match model.cancel.take() {
            Some(_) => (model, Cmd::None),
            None => {
                let (cancel, ticks) = tick();
                let model = Model {
                    last_tick: None,
                    cancel: Some(cancel.drop_guard()),
                };
                (model, Cmd::Sub(ticks.map(Msg::Tick).boxed()))
            }
        },
        Msg::Tick(new_time) => {
            model.last_tick = Some(new_time);
            (model, Cmd::None)
        }
    }
}

fn subscriptions(model: Model) -> (Model, Sub<Msg>) {
    (model, on_enter_key().map(|_| Msg::Toggle).boxed())
}

fn view(model: &Model) -> String {
    format!("last tick: {:?}", model.last_tick)
}

fn tick() -> (CancellationToken, impl Stream<Item = Instant>) {
    let interval = tokio::time::interval(Duration::from_secs(1));
    let ticks = IntervalStream::new(interval);
    let cancel = CancellationToken::new();
    let stream = stream::select(
        ticks.map(Some),
        cancel.clone().cancelled_owned().into_stream().map(|_| None),
    )
    .take_while(|time| ready(time.is_some()))
    .filter_map(|time| ready(time));
    (cancel, stream)
}

fn on_enter_key() -> impl Stream<Item = String> {
    let stdin = tokio::io::stdin();
    let buf = tokio::io::BufReader::new(stdin);
    LinesStream::new(buf.lines()).map(Result::unwrap)
}

#[tokio::main]
async fn main() {
    println!("Press Enter to toggle the clock");
    Program {
        init,
        view,
        update,
        subscriptions,
    }
    .run()
    .await;
}
