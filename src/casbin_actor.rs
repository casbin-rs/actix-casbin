use actix::prelude::*;
use casbin::prelude::*;
use casbin::{Error as CasbinError, Result};
use std::io::{Error, ErrorKind};
use std::marker::Unpin;
use std::sync::Arc;

pub enum CasbinCmd {
    Enforce(Vec<String>),
    AddPolicy(Vec<String>),
    AddPolicies(Vec<Vec<String>>),
    RemovePolicy(Vec<String>),
    RemovePolicies(Vec<Vec<String>>),
    RemoveFilteredPolicy(usize, Vec<String>),
}

impl Message for CasbinCmd {
    type Result = Result<bool>;
}

pub struct CasbinActor<
    M: TryIntoModel + Clone + Unpin + 'static,
    A: TryIntoAdapter + Clone + Unpin + 'static,
> {
    model: M,
    adapter: A,
    enforcer: Option<Arc<async_std::sync::RwLock<Enforcer>>>,
}

impl<M: TryIntoModel + Clone + Unpin + 'static, A: TryIntoAdapter + Clone + Unpin + 'static>
    CasbinActor<M, A>
{
    pub async fn new(m: M, a: A) -> Addr<CasbinActor<M, A>> {
        let clone_m = m.clone();
        let clone_a = a.clone();
        let enforcer: Enforcer = Enforcer::new(m, a).await.unwrap();
        Supervisor::start(|_| CasbinActor {
            model: clone_m,
            adapter: clone_a,
            enforcer: Some(Arc::new(async_std::sync::RwLock::new(enforcer))),
        })
    }
}

impl<M: TryIntoModel + Clone + Unpin + 'static, A: TryIntoAdapter + Clone + Unpin + 'static> Actor
    for CasbinActor<M, A>
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {}
}

impl<M: TryIntoModel + Clone + Unpin + 'static, A: TryIntoAdapter + Clone + Unpin + 'static>
    Supervised for CasbinActor<M, A>
{
    fn restarting(&mut self, _: &mut Self::Context) {
        self.enforcer.take();
    }
}

impl<M: TryIntoModel + Clone + Unpin + 'static, A: TryIntoAdapter + Clone + Unpin + 'static>
    Handler<CasbinCmd> for CasbinActor<M, A>
{
    type Result = ResponseActFuture<Self, Result<bool>>;

    fn handle(&mut self, msg: CasbinCmd, _: &mut Self::Context) -> Self::Result {
        let e = match &self.enforcer {
            Some(x) => x,
            None => {
                return Box::new(actix::fut::err(CasbinError::IoError(Error::new(
                    ErrorKind::NotConnected,
                    "Enforcer needed!",
                ))))
            }
        };
        let cloned_enforcer = Arc::clone(e);
        Box::new(
            async move {
                let mut lock = cloned_enforcer.write().await;
                let result = match msg {
                    CasbinCmd::Enforce(str) => lock.enforce(&str).await,
                    CasbinCmd::AddPolicy(str) => lock.add_policy(str).await,
                    CasbinCmd::AddPolicies(str) => lock.add_policies(str).await,
                    CasbinCmd::RemovePolicy(str) => lock.remove_policy(str).await,
                    CasbinCmd::RemovePolicies(str) => lock.remove_policies(str).await,
                    CasbinCmd::RemoveFilteredPolicy(idx, str) => {
                        lock.remove_filtered_policy(idx, str).await
                    }
                };
                result
            }
            .into_actor(self)
            .map(|res, _act, _ctx| res),
        )
    }
}
