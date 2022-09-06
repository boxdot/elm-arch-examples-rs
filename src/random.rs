use elm_arch::{Cmd, Program, Sub};
use futures::{Stream, StreamExt};
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;

#[derive(Debug)]
struct Model {
    #[allow(dead_code)]
    die_face: u8,
}

fn view(model: &Model) -> String {
    format!("{model:?}")
}

#[derive(Debug)]
enum Msg {
    Roll,
    NewFace(u8),
}

fn update(model: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    match msg {
        Msg::Roll => (model, Cmd::Msg(Msg::NewFace(roll_dice()))),
        Msg::NewFace(new_face) => (Model { die_face: new_face }, Cmd::None),
    }
}

fn init() -> (Model, Cmd<Msg>) {
    (Model { die_face: 1 }, Cmd::None)
}

fn on_enter_key() -> impl Stream<Item = String> {
    let stdin = tokio::io::stdin();
    let buf = tokio::io::BufReader::new(stdin);
    LinesStream::new(buf.lines()).map(Result::unwrap)
}

fn subscriptions(model: Model) -> (Model, Sub<Msg>) {
    (model, Box::pin(on_enter_key().map(|_| Msg::Roll)))
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
