# Actix Casbin

[![Crates.io](https://meritbadge.herokuapp.com/actix-casbin)](https://crates.io/crates/actix-casbin)
[![Docs](https://docs.rs/actix-casbin/badge.svg)](https://docs.rs/actix-casbin)
[![Auto Build](https://github.com/casbin-rs/actix-casbin/workflows/Auto%20Build/badge.svg)](https://github.com/casbin-rs/actix-casbin/actions/)
[![codecov](https://codecov.io/gh/casbin-rs/actix-casbin/branch/master/graph/badge.svg)](https://codecov.io/gh/casbin-rs/actix-casbin)

[Casbin](https://github.com/casbin/casbin-rs) intergration for [actix](https://github.com/actix/actix) framework

## Install

Add it to `Cargo.toml`

```rust
actix-casbin = "0.1.1"
actix-rt = "1.1.0"
```


## Example

```rust
use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};
use actix_casbin::casbin::prelude::*;

#[actix_rt::main]
async fn main() -> Result<()> {
    let m = DefaultModel::from_file("examples/rbac_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("examples/rbac_policy.csv");

    let addr = CasbinActor::new(m, a).await.unwrap();

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
        if test_enforce {
            println!("Enforce Pass");
        } else {
            println!("Enforce Fail");
        }
        Ok(())
    } else {
        panic!("Actor Error");
    }
}
```

## License

This project is licensed under

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
