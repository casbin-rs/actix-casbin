use actix::prelude::*;
use casbin::prelude::*;
use casbin::{Error as CasbinError, Result};
use std::io::{Error, ErrorKind};
use std::sync::Arc;

#[cfg(feature = "runtime-tokio")]
use tokio::sync::RwLock;

#[cfg(feature = "runtime-async-std")]
use async_std::sync::RwLock;

pub enum CasbinCmd {
    Enforce(Vec<String>),
    AddPolicy(Vec<String>),
    AddPolicies(Vec<Vec<String>>),
    RemovePolicy(Vec<String>),
    RemovePolicies(Vec<Vec<String>>),
    RemoveFilteredPolicy(usize, Vec<String>),
    AddRoleForUser(String, String, Option<String>),
    AddRolesForUser(String, Vec<String>, Option<String>),
    DeleteRoleForUser(String, String, Option<String>),
    DeleteRolesForUser(String, Option<String>),
    GetImplicitRolesForUser(String, Option<String>),
    GetImplicitPermissionsForUser(String, Option<String>),
}

pub enum CasbinResult {
    Enforce(bool),
    AddPolicy(bool),
    AddPolicies(bool),
    RemovePolicy(bool),
    RemovePolicies(bool),
    RemoveFilteredPolicy(bool),
    AddRoleForUser(bool),
    AddRolesForUser(bool),
    DeleteRoleForUser(bool),
    DeleteRolesForUser(bool),
    GetImplicitRolesForUser(Vec<String>),
    GetImplicitPermissionsForUser(Vec<Vec<String>>),
}

impl Message for CasbinCmd {
    type Result = Result<CasbinResult>;
}

pub struct CasbinActor {
    enforcer: Option<Arc<RwLock<Enforcer>>>,
}

impl CasbinActor {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(m: M, a: A) -> Result<Addr<CasbinActor>> {
        let enforcer: Enforcer = Enforcer::new(m, a).await?;
        Ok(Supervisor::start(|_| CasbinActor {
            enforcer: Some(Arc::new(RwLock::new(enforcer))),
        }))
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
    type Result = ResponseActFuture<Self, Result<CasbinResult>>;

    fn handle(&mut self, msg: CasbinCmd, _: &mut Self::Context) -> Self::Result {
        let e = match &self.enforcer {
            Some(x) => x,
            None => {
                return Box::new(actix::fut::err(CasbinError::IoError(Error::new(
                    ErrorKind::NotConnected,
                    "Enforcer droped!",
                ))))
            }
        };
        let cloned_enforcer = Arc::clone(e);
        Box::new(
            async move {
                let mut lock = cloned_enforcer.write().await;
                match msg {
                    CasbinCmd::Enforce(str) => lock.enforce(&str).await.map(CasbinResult::Enforce),
                    CasbinCmd::AddPolicy(str) => {
                        lock.add_policy(str).await.map(CasbinResult::AddPolicy)
                    }
                    CasbinCmd::AddPolicies(str) => {
                        lock.add_policies(str).await.map(CasbinResult::AddPolicies)
                    }
                    CasbinCmd::RemovePolicy(str) => lock
                        .remove_policy(str)
                        .await
                        .map(CasbinResult::RemovePolicy),
                    CasbinCmd::RemovePolicies(str) => lock
                        .remove_policies(str)
                        .await
                        .map(CasbinResult::RemovePolicies),
                    CasbinCmd::RemoveFilteredPolicy(idx, str) => lock
                        .remove_filtered_policy(idx, str)
                        .await
                        .map(CasbinResult::RemoveFilteredPolicy),
                    CasbinCmd::AddRoleForUser(user, roles, domain) => lock
                        .add_role_for_user(&user, &roles, domain.as_deref())
                        .await
                        .map(CasbinResult::AddRoleForUser),
                    CasbinCmd::AddRolesForUser(user, roles, domain) => lock
                        .add_roles_for_user(&user, roles, domain.as_deref())
                        .await
                        .map(CasbinResult::AddRolesForUser),
                    CasbinCmd::DeleteRoleForUser(user, roles, domain) => lock
                        .delete_role_for_user(&user, &roles, domain.as_deref())
                        .await
                        .map(CasbinResult::DeleteRoleForUser),
                    CasbinCmd::DeleteRolesForUser(user, domain) => lock
                        .delete_roles_for_user(&user, domain.as_deref())
                        .await
                        .map(CasbinResult::DeleteRolesForUser),
                    CasbinCmd::GetImplicitRolesForUser(name, domain) => {
                        Ok(CasbinResult::GetImplicitRolesForUser(
                            lock.get_implicit_roles_for_user(&name, domain.as_deref()),
                        ))
                    }
                    CasbinCmd::GetImplicitPermissionsForUser(name, domain) => {
                        Ok(CasbinResult::GetImplicitPermissionsForUser(
                            lock.get_implicit_permissions_for_user(&name, domain.as_deref()),
                        ))
                    }
                }
            }
            .into_actor(self)
            .map(|res, _act, _ctx| res),
        )
    }
}
