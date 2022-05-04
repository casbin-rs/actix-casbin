use actix::prelude::*;
use casbin::prelude::*;
use casbin::{Error as CasbinError, IEnforcer, Result};
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
    AddNamedPolicy(String, Vec<String>),
    AddNamedPolicies(String, Vec<Vec<String>>),
    AddGroupingPolicy(Vec<String>),
    AddGroupingPolicies(Vec<Vec<String>>),
    AddNamedGroupingPolicy(String, Vec<String>),
    AddNamedGroupingPolicies(String, Vec<Vec<String>>),
    RemovePolicy(Vec<String>),
    RemovePolicies(Vec<Vec<String>>),
    RemoveNamedPolicy(String, Vec<String>),
    RemoveNamedPolicies(String, Vec<Vec<String>>),
    RemoveGroupingPolicy(Vec<String>),
    RemoveGroupingPolicies(Vec<Vec<String>>),
    RemoveNamedGroupingPolicy(String, Vec<String>),
    RemoveNamedGroupingPolicies(String, Vec<Vec<String>>),
    RemoveFilteredNamedPolicy(String, usize, Vec<String>),
    RemoveFilteredNamedGroupingPolicy(String, usize, Vec<String>),
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
    AddNamedPolicy(bool),
    AddNamedPolicies(bool),
    AddGroupingPolicy(bool),
    AddGroupingPolicies(bool),
    AddNamedGroupingPolicy(bool),
    AddNamedGroupingPolicies(bool),
    RemovePolicy(bool),
    RemovePolicies(bool),
    RemoveNamedPolicy(bool),
    RemoveNamedPolicies(bool),
    RemoveGroupingPolicy(bool),
    RemoveGroupingPolicies(bool),
    RemoveNamedGroupingPolicy(bool),
    RemoveNamedGroupingPolicies(bool),
    RemoveFilteredNamedPolicy(bool),
    RemoveFilteredNamedGroupingPolicy(bool),
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

pub struct CasbinActor<T: IEnforcer + 'static> {
    pub enforcer: Option<Arc<RwLock<T>>>,
}

impl<T: IEnforcer + 'static> CasbinActor<T> {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(
        m: M,
        a: A,
    ) -> Result<Addr<CasbinActor<T>>> {
        let enforcer = T::new(m, a).await?;
        Ok(Supervisor::start(|_| CasbinActor {
            enforcer: Some(Arc::new(RwLock::new(enforcer))),
        }))
    }

    pub fn set_enforcer(e: Arc<RwLock<T>>) -> Result<CasbinActor<T>> {
        Ok(CasbinActor { enforcer: Some(e) })
    }

    pub fn get_enforcer(&mut self) -> Option<Arc<RwLock<T>>> {
        self.enforcer.as_ref().map(Arc::clone)
    }
}

impl<T: IEnforcer + 'static> Actor for CasbinActor<T> {
    type Context = Context<Self>;
}

impl<T: IEnforcer + 'static> Supervised for CasbinActor<T> {
    fn restarting(&mut self, _: &mut Self::Context) {
        self.enforcer.take();
    }
}

impl<T: IEnforcer + 'static> Handler<CasbinCmd> for CasbinActor<T> {
    type Result = ResponseActFuture<Self, Result<CasbinResult>>;

    fn handle(&mut self, msg: CasbinCmd, _: &mut Self::Context) -> Self::Result {
        let e = match &self.enforcer {
            Some(x) => x,
            None => {
                return Box::pin(actix::fut::err(CasbinError::IoError(Error::new(
                    ErrorKind::NotConnected,
                    "Enforcer dropped!",
                ))))
            }
        };
        let cloned_enforcer = Arc::clone(e);
        Box::pin(
            async move {
                let mut lock = cloned_enforcer.write().await;
                let result = match msg {
                    CasbinCmd::Enforce(policy) => lock.enforce(policy).map(CasbinResult::Enforce),
                    CasbinCmd::AddPolicy(policy) => {
                        lock.add_policy(policy).await.map(CasbinResult::AddPolicy)
                    }
                    CasbinCmd::AddPolicies(policy) => lock
                        .add_policies(policy)
                        .await
                        .map(CasbinResult::AddPolicies),
                    CasbinCmd::AddNamedPolicy(ptype, policy) => lock
                        .add_named_policy(&ptype, policy)
                        .await
                        .map(CasbinResult::AddNamedPolicy),
                    CasbinCmd::AddNamedPolicies(ptype, policy) => lock
                        .add_named_policies(&ptype, policy)
                        .await
                        .map(CasbinResult::AddNamedPolicies),
                    CasbinCmd::AddGroupingPolicy(policy) => lock
                        .add_grouping_policy(policy)
                        .await
                        .map(CasbinResult::AddGroupingPolicy),
                    CasbinCmd::AddGroupingPolicies(policy) => lock
                        .add_grouping_policies(policy)
                        .await
                        .map(CasbinResult::AddGroupingPolicies),
                    CasbinCmd::AddNamedGroupingPolicy(ptype, policy) => lock
                        .add_named_grouping_policy(&ptype, policy)
                        .await
                        .map(CasbinResult::AddNamedGroupingPolicy),
                    CasbinCmd::AddNamedGroupingPolicies(ptype, policy) => lock
                        .add_named_grouping_policies(&ptype, policy)
                        .await
                        .map(CasbinResult::AddNamedGroupingPolicies),
                    CasbinCmd::RemoveNamedPolicy(ptype, policy) => lock
                        .remove_named_policy(&ptype, policy)
                        .await
                        .map(CasbinResult::RemoveNamedPolicy),
                    CasbinCmd::RemoveNamedPolicies(ptype, policy) => lock
                        .remove_named_policies(&ptype, policy)
                        .await
                        .map(CasbinResult::RemoveNamedPolicies),
                    CasbinCmd::RemoveGroupingPolicy(policy) => lock
                        .remove_grouping_policy(policy)
                        .await
                        .map(CasbinResult::RemoveGroupingPolicy),
                    CasbinCmd::RemoveGroupingPolicies(policy) => lock
                        .remove_grouping_policies(policy)
                        .await
                        .map(CasbinResult::RemoveGroupingPolicies),
                    CasbinCmd::RemoveNamedGroupingPolicy(ptype, policy) => lock
                        .remove_named_grouping_policy(&ptype, policy)
                        .await
                        .map(CasbinResult::RemoveNamedGroupingPolicy),
                    CasbinCmd::RemoveNamedGroupingPolicies(ptype, policy) => lock
                        .remove_named_grouping_policies(&ptype, policy)
                        .await
                        .map(CasbinResult::RemoveNamedGroupingPolicies),
                    CasbinCmd::RemovePolicy(policy) => lock
                        .remove_policy(policy)
                        .await
                        .map(CasbinResult::RemovePolicy),
                    CasbinCmd::RemovePolicies(policy) => lock
                        .remove_policies(policy)
                        .await
                        .map(CasbinResult::RemovePolicies),
                    CasbinCmd::RemoveFilteredNamedPolicy(ptype, idx, policy) => lock
                        .remove_filtered_named_policy(&ptype, idx, policy)
                        .await
                        .map(CasbinResult::RemoveFilteredNamedPolicy),
                    CasbinCmd::RemoveFilteredNamedGroupingPolicy(ptype, idx, policy) => lock
                        .remove_filtered_named_grouping_policy(&ptype, idx, policy)
                        .await
                        .map(CasbinResult::RemoveFilteredNamedGroupingPolicy),
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
                };
                drop(lock);
                result
            }
            .into_actor(self)
            .map(|res, _act, _ctx| res),
        )
    }
}
