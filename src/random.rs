use futures::prelude::*;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver};

use elm_arch::{Cmd, Program, Sub};

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

fn update(model: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    match msg {
        Msg::Roll => (model, Cmd::new(|| Msg::NewFace(roll_dice()))),
        Msg::NewFace(new_face) => (Model { die_face: new_face }, Cmd::None),
    }
}

fn init() -> (Model, Cmd<Msg>) {
    (Model { die_face: 1 }, Cmd::None)
}

fn on_enter_key(handle: Handle) -> Receiver<String> {
    let (tx, rx) = channel::<String>(1);
    std::thread::spawn(move || {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let mut tx = tx.clone();
            handle.spawn(async move {
                tx.send(line.unwrap()).await.unwrap();
            });
        }
    });
    rx
}

fn subscriptions(model: Model, handle: Handle) -> (Model, Sub<Msg>) {
    (model, Box::pin(on_enter_key(handle).map(|_| Msg::Roll)))
}

fn roll_dice() -> u8 {
    use rand::distributions::{Distribution, Uniform};
    let between = Uniform::from(1..6);
    let mut rng = rand::thread_rng();
    between.sample(&mut rng)
}

#[tokio::main]
async fn main() {
    println!("Please, press enter!");
    Program {
        init,
        view,
        update,
        subscriptions,
    }
    .run()
    .await;
}
