use futures::prelude::*;

pub struct Program<Init, View, Update, Subscriptions> {
    pub init: Init,
    pub view: View,
    pub update: Update,
    pub subscriptions: Subscriptions,
}

impl<I, V, U, S> Program<I, V, U, S> {
    pub fn run<Model, Cmd, Msg>(self)
    where
        I: FnOnce() -> (Model, Cmd),
        V: Fn(&Model) -> String,
        U: Fn(Model, Msg) -> (Model, Cmd),
        S: FnOnce() -> Box<Stream<Item = Msg, Error = ()>>,
    {
        let Self {
            init,
            view,
            update,
            subscriptions,
        } = self;

        let (initial_model, _cmd) = init();
        subscriptions()
            .fold(initial_model, |model, msg| {
                let (new_model, _cmd) = update(model, msg);
                println!("{}", view(&new_model));
                Ok(new_model)
            })
            .wait()
            .unwrap();
    }
}
