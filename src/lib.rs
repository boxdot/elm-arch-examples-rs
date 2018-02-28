extern crate futures;
extern crate tokio_core;

use futures::prelude::*;
use futures::future;
use futures::sync::mpsc::{channel, Sender};
use tokio_core::reactor::{Core, Handle};

pub trait CmdWithHandle<Msg> {
    fn call(self: Box<Self>, handle: Handle) -> Box<Future<Item = Msg, Error = ()>>;
}

pub enum Cmd<Msg> {
    None,
    Cmd(Box<Future<Item = Msg, Error = ()>>),
    WithHandle(Box<CmdWithHandle<Msg>>),
}

impl<Msg: 'static> Cmd<Msg> {
    pub fn new<F>(f: F) -> Cmd<Msg>
    where
        F: FnOnce() -> Msg + 'static,
    {
        Cmd::Cmd(Box::new(future::lazy(|| Ok(f()))))
    }

    pub fn with_handle<F>(f: F) -> Cmd<Msg>
    where
        F: CmdWithHandle<Msg> + 'static,
    {
        Cmd::WithHandle(Box::new(f))
    }
}

pub type Sub<Msg> = Box<Stream<Item = Msg, Error = ()>>;

pub struct Program<Init, View, Update, Subscriptions> {
    pub init: Init,
    pub view: View,
    pub update: Update,
    pub subscriptions: Subscriptions,
}

impl<I, V, U, S> Program<I, V, U, S> {
    pub fn run<Model, Msg>(self)
    where
        Msg: 'static,
        I: FnOnce() -> (Model, Cmd<Msg>),
        V: Fn(&Model) -> String,
        U: Fn(Model, Msg) -> (Model, Cmd<Msg>),
        S: FnOnce(Model, Handle) -> (Model, Box<Stream<Item = Msg, Error = ()>>),
    {
        let Self {
            init,
            view,
            update,
            subscriptions,
        } = self;

        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let (messages_sender, messages) = channel::<Msg>(1);

        let (initial_model, initial_cmd) = init();

        process_cmd(initial_cmd, handle.clone(), messages_sender.clone());

        let (model, subs) = subscriptions(initial_model, handle.clone());
        let program = subs.select(messages).fold(model, |model, msg| {
            let (new_model, cmd) = update(model, msg);
            process_cmd(cmd, handle.clone(), messages_sender.clone());
            println!("{}", view(&new_model));
            Ok(new_model)
        });

        core.run(program).unwrap();
    }
}

fn process_cmd<Msg: 'static>(cmd: Cmd<Msg>, handle: Handle, sender: Sender<Msg>) {
    match cmd {
        Cmd::Cmd(fut) => {
            handle.spawn(fut.and_then(move |msg| {
                sender.send(msg).wait().unwrap();
                Ok(())
            }));
        }
        Cmd::WithHandle(make_fut) => handle.spawn(
            future::ok(handle.clone())
                .and_then(move |handle| make_fut.call(handle))
                .and_then(move |msg| {
                    sender.send(msg).wait().unwrap();
                    Ok(())
                }),
        ),
        Cmd::None => (),
    }
}
