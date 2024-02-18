# Actix Casbin

[![Crates.io](https://img.shields.io/crates/d/actix-casbin)](https://crates.io/crates/actix-casbin)
[![Docs](https://docs.rs/actix-casbin/badge.svg)](https://docs.rs/actix-casbin)
[![Auto Build](https://github.com/casbin-rs/actix-casbin/workflows/Auto%20Build/badge.svg)](https://github.com/casbin-rs/actix-casbin/actions/)
[![codecov](https://codecov.io/gh/casbin-rs/actix-casbin/branch/master/graph/badge.svg)](https://codecov.io/gh/casbin-rs/actix-casbin)

[Casbin](https://github.com/casbin/casbin-rs) intergration for [actix](https://github.com/actix/actix) framework

## Install

Add it to `Cargo.toml`

```rust
actix-casbin = "1.0.0"
actix-rt = "2.7.0"
```


## Example

1. Using actix-casbin as actor alone

```rust
use actix_casbin::casbin::{DefaultModel, FileAdapter, Result, Enforcer};
use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};

#[actix_rt::main]
async fn main() -> Result<()> {
    let m = DefaultModel::from_file("examples/rbac_model.conf").await?;

    let a = FileAdapter::new("examples/rbac_policy.csv");

    let addr = CasbinActor::<Enforcer>::new(m, a).await?;

    let res = addr
        .send(CasbinCmd::Enforce(
            vec!["alice", "data1", "read"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        ))
        .await;

    let test_enforce = match res {
        Ok(Ok(CasbinResult::Enforce(result))) => result,
        _ => panic!("Actor Error"),
    };
    if test_enforce {
        println!("Enforce Pass");
    } else {
        println!("Enforce Fail");
    }
    Ok(())
}
```
2. Use actix-casbin with casbin actix middleware [actix-casbin-auth](https://github.com/casbin-rs/actix-casbin-auth)
```rust
use actix_casbin::casbin::{DefaultModel, FileAdapter, Result, CachedEnforcer};
use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};
use actix_casbin_auth::CasbinService;

#[actix_rt::main]
async fn main() -> Result<()> {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await?;
    let a = FileAdapter::new("examples/rbac_policy.csv");

    let mut casbin_middleware = CasbinService::new(m, a).await;
    let enforcer = casbin_middleware.get_enforcer();

    let addr = CasbinActor::<CachedEnforcer>::set_enforcer(enforcer)?;
    if let CasbinResult::Enforce(test_enforce) = addr
        .send(CasbinCmd::Enforce(
            vec!["alice", "data1", "read"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ))
        .await;
    let test_enforce = match res {
        Ok(Ok(CasbinResult::Enforce(result))) => result,
        _ => panic!("Actor Error"),
    };
    if test_enforce {
        println!("Enforce Pass");
    } else {
        println!("Enforce Fail");
    }

    Ok(())
}
```

## License

This project is licensed under

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
