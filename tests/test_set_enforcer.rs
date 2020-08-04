use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};
use actix_casbin_auth::CasbinService;
use casbin::prelude::*;

#[actix_rt::test]
async fn test_set_enforcer() {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");

    let mut casbin_middleware = CasbinService::new(m, a).await;
    let enforcer = casbin_middleware.get_enforcer().await;

    let addr = CasbinActor::<CachedEnforcer>::set_enforcer(enforcer)
        .await
        .unwrap();
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
