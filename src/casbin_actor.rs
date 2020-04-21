use actix::prelude::*;
use casbin::prelude::*;
use casbin::Result;
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

pub struct CasbinActor {
    enforcer: Option<Arc<async_std::sync::RwLock<Enforcer>>>,
}

impl CasbinActor {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(m: M, a: A) -> Addr<CasbinActor> {
        let enforcer: Enforcer = Enforcer::new(m, a).await.unwrap();
        Supervisor::start(|_| CasbinActor {
            enforcer: Some(Arc::new(async_std::sync::RwLock::new(enforcer))),
        })
    }
}

impl Actor for CasbinActor {
    type Context = Context<Self>;
}

impl Supervised for CasbinActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        self.enforcer.take();
    }
}

impl Handler<CasbinCmd> for CasbinActor {
    type Result = ResponseActFuture<Self, Result<bool>>;

    fn handle(&mut self, msg: CasbinCmd, _: &mut Self::Context) -> Self::Result {
        let e = match &self.enforcer {
            Some(x) => x,
            None => panic!("Enforcer needed"),
        };
        let cloned_enforcer = Arc::clone(e);
        Box::new(
            async move {
                let mut lock = cloned_enforcer.try_write().unwrap();
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
