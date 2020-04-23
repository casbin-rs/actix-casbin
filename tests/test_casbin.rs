use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};
use casbin::prelude::*;

#[actix_rt::test]
async fn test_enforcer() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::new(m, a).await;
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
    use std::thread;
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::new(m, a).await;

    for i in 0..8 {
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
