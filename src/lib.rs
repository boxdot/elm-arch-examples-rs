extern crate futures;
extern crate tokio_core;

use futures::prelude::*;
use futures::future;
use futures::sync::mpsc::channel;
use tokio_core::reactor::Core;

pub enum Cmd<Msg> {
    None,
    Cmd(Box<Future<Item = Msg, Error = ()>>),
}

impl<Msg: 'static> Cmd<Msg> {
    pub fn new<F>(f: F) -> Cmd<Msg>
    where
        F: FnOnce() -> Msg + 'static,
    {
        Cmd::Cmd(Box::new(future::lazy(|| Ok(f()))))
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
        U: Fn(Model, &Msg) -> (Model, Cmd<Msg>),
        S: FnOnce() -> Box<Stream<Item = Msg, Error = ()>>,
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

        let tx = messages_sender.clone();
        if let Cmd::Cmd(fut) = initial_cmd {
            handle.spawn(fut.and_then(move |msg| {
                tx.clone().send(msg).wait().unwrap();
                Ok(())
            }));
        }

        let program = subscriptions()
            .select(messages)
            .fold(initial_model, |model, msg| {
                let (new_model, cmd) = update(model, &msg);
                if let Cmd::Cmd(fut) = cmd {
                    let tx = messages_sender.clone();
                    handle.spawn(fut.and_then(move |msg| {
                        tx.clone().send(msg).wait().unwrap();
                        Ok(())
                    }));
                }

                println!("{}", view(&new_model));
                Ok(new_model)
            });

        core.run(program).unwrap();
    }
}
