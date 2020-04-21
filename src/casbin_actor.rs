use casbin::prelude::*;
use actix::prelude::*;


pub struct CasbinActor {
    enforcer: Option<Enforcer>
}

impl CasbinActor {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(m: M, a:A)
    -> Result<Addr<CasbinActor>> {
        let enforcer: Enforcer = Enforcer::new(m,a).await?;
        Supervisor::start(|_| CasbinActor {
            enforcer: Some(enforcer)
        })?
    }
}

impl Actor for CasbinActor {
    type Context = Context<self>;

}


impl Supervised for CasbinActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        self.enforcer.take();
    }
}


pub enum CasbinCmd {
    Enforce(Vec<String>),
    AddPolicy(Vec<String>),
    AddPolicies(Vec<Vec<String>>),
    RemovePolicy(Vec<String>),
    RemovePolicies(Vec<Vec<String>>),
    RemoveFilteredPolicy(usize,Vec<String>),
    LoadPolicy(),
}

impl Message for CasbinCmd {
    type Result = Result<bool>;
}

impl Handler<CasbinCmd> for CasbinActor {
    type Result = ResponseActFuture<Self, Result<bool>>;

    fn handle(&mut self, msg: CasbinCmd, _: &mut Self::Context) -> Self::Result {
        match msg {
            CasbinCmd::Enforce(str) => self.enforcer.unwrap().enforce(&str),
            CasbinCmd::AddPolicy(str) => self.enforcer.unwrap().add_policy(str),
            CasbinCmd::AddPolicies(str) => self.enforcer.unwrap().add_policies(str),
            CasbinCmd::RemovePolicy(str) => self.enforcer.unwrap().remove_policy(str),
            CasbinCmd::RemovePolicies(str) => self.enforcer.unwrap().remove_policies(str),
            CasbinCmd::RemoveFilteredPolicy(index,str) => self.enforcer.unwrap().remove_filtered_policy(usize,str),
            CasbinCmd::LoadPolicy() => self.enforcer.unwrap().load_policy()?,
        }
    }
}
