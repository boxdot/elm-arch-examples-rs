extern crate futures;
extern crate rand;
extern crate tokio_core;

mod program;

use futures::prelude::*;
use futures::sync::mpsc::{channel, Receiver};
use rand::distributions::{Range, Sample};

use std::io::{self, BufRead};
use std::thread;

use program::{Cmd, Program, Sub};

#[derive(Debug)]
struct Model {
    die_face: u8,
}

fn view(model: &Model) -> String {
    format!("{:?}", model)
}

#[derive(Debug)]
enum Msg {
    Roll,
    NewFace(u8),
}

fn update(model: Model, msg: &Msg) -> (Model, Cmd<Msg>) {
    match *msg {
        Msg::Roll => (model, Cmd::new(|| Msg::NewFace(roll_dice()))),
        Msg::NewFace(new_face) => (Model { die_face: new_face }, Cmd::None),
    }
}

fn init() -> (Model, Cmd<Msg>) {
    (Model { die_face: 1 }, Cmd::None)
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

fn subscriptions() -> Sub<Msg> {
    Box::new(on_enter_key().map(|_| Msg::Roll))
}

fn roll_dice() -> u8 {
    let mut between = Range::new(1, 6);
    let mut rng = rand::thread_rng();
    between.sample(&mut rng)
}

fn main() {
    Program {
        init,
        view,
        update,
        subscriptions,
    }.run()
}
