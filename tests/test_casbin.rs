use actix_casbin::{CasbinActor, CasbinCmd};
use casbin::prelude::*;

#[actix_rt::test]
async fn test_enforcer() {
    //let m = DefaultModel::from_file("examples/rbac_model.conf")
    //    .await
    //    .unwrap();
    //let a = FileAdapter::new("examples/rbac_policy.csv");
    let addr = CasbinActor::new("examples/rbac_model.conf", "examples/rbac_policy.csv").await;
    let test_enforce = addr
        .send(CasbinCmd::Enforce(
            vec!["alice", "data1", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(true, test_enforce);
}
