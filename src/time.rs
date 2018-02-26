extern crate futures;
extern crate time;
extern crate tokio_core;

use futures::prelude::*;
use futures::sync::mpsc::{channel, Receiver};
use time::{now_utc, Tm};

use std::thread;

mod program;

use program::{Cmd, Program, Sub};

struct Model(Tm);

fn init() -> (Model, Cmd<Msg>) {
    (Model(now_utc()), Cmd::None)
}

enum Msg {
    Tick(Tm),
}

fn update(_: Model, msg: &Msg) -> (Model, Cmd<Msg>) {
    match *msg {
        Msg::Tick(new_time) => (Model(new_time), Cmd::None),
    }
}

fn tick() -> Receiver<Tm> {
    let (tx, rx) = channel::<Tm>(1);
    thread::spawn(move || -> Result<(), ()> {
        loop {
            let now = now_utc();
            tx.clone().send(now).wait().unwrap();
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    rx
}

fn subscriptions() -> Sub<Msg> {
    Box::new(tick().map(Msg::Tick))
}

fn view(model: &Model) -> String {
    format!("{:?}", model.0)
}

fn main() {
    Program {
        init,
        view,
        update,
        subscriptions,
    }.run()
}
