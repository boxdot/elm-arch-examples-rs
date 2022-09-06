use futures::prelude::*;
use tokio::sync::mpsc::{channel, Sender};

pub type BoxFuture<T> = future::BoxFuture<'static, T>;
pub type BoxStream<T> = stream::BoxStream<'static, T>;

pub enum Cmd<Msg> {
    None,
    Future(BoxFuture<Msg>),
    Msg(Msg),
}

impl<Msg> Cmd<Msg> {
    pub fn boxed<F>(f: F) -> Self
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        Self::Future(Box::pin(f))
    }
}

pub type Sub<Msg> = BoxStream<Msg>;

pub struct Program<Init, View, Update, Subscriptions> {
    pub init: Init,
    pub view: View,
    pub update: Update,
    pub subscriptions: Subscriptions,
}

impl<I, V, U, S> Program<I, V, U, S> {
    pub async fn run<Model, Msg>(self)
    where
        Msg: Send + 'static,
        I: FnOnce() -> (Model, Cmd<Msg>),
        V: Fn(&Model) -> String,
        U: Fn(Model, Msg) -> (Model, Cmd<Msg>),
        S: FnOnce(Model) -> (Model, BoxStream<Msg>),
    {
        let Self {
            init,
            view,
            update,
            subscriptions,
        } = self;

        let (msg_tx, msg_rx) = channel::<Msg>(1);

        let (initial_model, initial_cmd) = init();

        process_cmd(initial_cmd, msg_tx.clone());

        let (model, subs) = subscriptions(initial_model);
        let msgs = tokio_stream::wrappers::ReceiverStream::new(msg_rx);
        let program = stream::select(subs, msgs).fold(model, |model, msg| {
            let (new_model, cmd) = update(model, msg);
            process_cmd(cmd, msg_tx.clone());
            println!("{}", view(&new_model));
            future::ready(new_model)
        });

        program.await;
    }
}

fn process_cmd<Msg: Send + 'static>(cmd: Cmd<Msg>, tx: Sender<Msg>) {
    match cmd {
        Cmd::Future(fut) => {
            tokio::spawn(async move {
                let msg = fut.await;
                if let Err(e) = tx.send(msg).await {
                    panic!("channel closed: {}", e);
                }
            });
        }
        Cmd::Msg(msg) => {
            tokio::spawn(async move {
                if let Err(e) = tx.send(msg).await {
                    panic!("channel closed: {}", e);
                }
            });
        }
        Cmd::None => (),
    }
}
