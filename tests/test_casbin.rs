use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};
use casbin::prelude::*;

#[actix_rt::test]
async fn test_enforcer() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::<Enforcer>::new(m, a).await.unwrap();

    if let CasbinResult::Enforce(test_enforce) = addr
        .send(CasbinCmd::Enforce(
            vec!["alice", "data1", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, test_enforce);
    }
}

#[actix_rt::test]
async fn test_enforcer_threads() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::<Enforcer>::new(m, a).await.unwrap();

    for _ in 0..8 {
        let clone_addr = addr.clone();
        tokio::spawn(async move {
            if let CasbinResult::Enforce(test_enforce) = clone_addr
                .send(CasbinCmd::Enforce(
                    vec!["alice", "data1", "read"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ))
                .await
                .unwrap()
                .unwrap()
            {
                assert_eq!(true, test_enforce);
            }
        });
    }
}

#[actix_rt::test]
async fn test_policy_command() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::<Enforcer>::new(m, a).await.unwrap();

    if let CasbinResult::RemovePolicy(remove_policy) = addr
        .send(CasbinCmd::RemovePolicy(
            vec!["alice", "data1", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, remove_policy);
    }

    if let CasbinResult::RemoveFilteredNamedPolicy(remove_filtered_policy) = addr
        .send(CasbinCmd::RemoveFilteredNamedPolicy(
            "p".to_string(),
            1,
            vec!["data2"].iter().map(|s| s.to_string()).collect(),
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, remove_filtered_policy);
    }

    if let CasbinResult::AddPolicy(add_policy) = addr
        .send(CasbinCmd::AddPolicy(
            vec!["eve", "data3", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, add_policy);
    }

    if let CasbinResult::AddPolicy(add_policies) = addr
        .send(CasbinCmd::AddPolicies(vec![
            vec!["lucy", "data3", "write"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            vec!["jack", "data4", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ]))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, add_policies);
    }

    if let CasbinResult::RemovePolicies(remove_policies) = addr
        .send(CasbinCmd::RemovePolicies(vec![
            vec!["lucy", "data3", "write"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            vec!["jack", "data4", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ]))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, remove_policies);
    }
}

#[actix_rt::test]
async fn test_roles_command() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::<Enforcer>::new(m, a).await.unwrap();

    if let CasbinResult::AddRoleForUser(add_role_for_user) = addr
        .send(CasbinCmd::AddRoleForUser(
            "alice".to_string(),
            "data1_admin".to_string(),
            None,
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, add_role_for_user);
    }

    if let CasbinResult::AddRolesForUser(add_roles_for_user) = addr
        .send(CasbinCmd::AddRolesForUser(
            "bob".to_string(),
            vec!["data1_admin", "data2_admin"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            None,
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, add_roles_for_user);
    }

    if let CasbinResult::DeleteRoleForUser(delete_role_for_user) = addr
        .send(CasbinCmd::DeleteRoleForUser(
            "alice".to_string(),
            "data1_admin".to_string(),
            None,
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, delete_role_for_user);
    }

    if let CasbinResult::DeleteRolesForUser(delete_roles_for_user) = addr
        .send(CasbinCmd::DeleteRolesForUser("bob".to_string(), None))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(true, delete_roles_for_user);
    }
}

#[actix_rt::test]
async fn test_implicit_roles_command() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_with_hierarchy_policy.csv");
    let addr = CasbinActor::<Enforcer>::new(m, a).await.unwrap();

    if let CasbinResult::GetImplicitRolesForUser(implicit_roles_alice) = addr
        .send(CasbinCmd::GetImplicitRolesForUser(
            "alice".to_string(),
            None,
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(
            vec!["admin", "data1_admin", "data2_admin"],
            sort_unstable(implicit_roles_alice)
        );
    }

    if let CasbinResult::GetImplicitRolesForUser(implicit_roles_bob) = addr
        .send(CasbinCmd::GetImplicitRolesForUser("bob".to_string(), None))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(vec![String::new(); 0], implicit_roles_bob);
    }
}

#[actix_rt::test]
async fn test_implicit_permissions_command() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_with_hierarchy_policy.csv");
    let addr = CasbinActor::<Enforcer>::new(m, a).await.unwrap();

    if let CasbinResult::GetImplicitPermissionsForUser(implicit_permissions_alice) = addr
        .send(CasbinCmd::GetImplicitPermissionsForUser(
            "alice".to_string(),
            None,
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(
            vec![
                vec!["alice", "data1", "read"],
                vec!["data1_admin", "data1", "read"],
                vec!["data1_admin", "data1", "write"],
                vec!["data2_admin", "data2", "read"],
                vec!["data2_admin", "data2", "write"],
            ],
            sort_unstable(implicit_permissions_alice)
        );
    }

    if let CasbinResult::GetImplicitPermissionsForUser(implicit_permissions_bob) = addr
        .send(CasbinCmd::GetImplicitPermissionsForUser(
            "bob".to_string(),
            None,
        ))
        .await
        .unwrap()
        .unwrap()
    {
        assert_eq!(
            vec![vec!["bob", "data2", "write"]],
            implicit_permissions_bob
        );
    }
}

fn sort_unstable<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort_unstable();
    v
}
