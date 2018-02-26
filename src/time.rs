extern crate futures;
extern crate time;

use futures::prelude::*;
use futures::sync::mpsc::{channel, Receiver};
use time::{now_utc, Tm};

use std::thread;

#[derive(Debug)]
struct Program<Init, View, Update, Subscriptions> {
    init: Init,
    view: View,
    update: Update,
    subscriptions: Subscriptions,
}

struct Model(Tm);

fn init() -> (Model, Cmd) {
    (Model(now_utc()), Cmd::None)
}

enum Msg {
    Tick(Tm),
}

enum Cmd {
    None,
    _Msg(Msg),
}

fn update(_model: Model, msg: Msg) -> (Model, Cmd) {
    match msg {
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

fn subscriptions() -> Box<Stream<Item = Msg, Error = ()>> {
    Box::new(tick().map(Msg::Tick))
}

fn view(model: &Model) -> String {
    format!("{:?}", model.0)
}

fn main() {
    let program = Program {
        init: init,
        view: view,
        update: update,
        subscriptions: subscriptions,
    };

    (program.subscriptions)()
        .fold((program.init)().0, |model, msg| {
            let (new_model, _cmd) = (program.update)(model, msg);
            println!("{}", (program.view)(&new_model));
            Ok(new_model)
        })
        .wait()
        .unwrap();
}
