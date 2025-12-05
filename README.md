# myloginrs
[![Build status](https://img.shields.io/github/actions/workflow/status/rjcortese/myloginrs/ci.yml?branch=master)](https://github.com/rjcortese/myloginrs/actions)
[![](https://img.shields.io/crates/v/myloginrs)](https://crates.io/crates/myloginrs)
![Maintenance](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)


Read and parse MySQL's
[.mylogin.cnf](https://dev.mysql.com/doc/refman/5.7/en/mysql-config-editor.html)
file.

## Installation
Add `myloginrs` to `Cargo.toml`:

```toml
[dependencies]
myloginrs = "0.1"
```


## Examples

To get a HashMap of login info for `"client"` just use the parse function:

```rust
let file_path = PathBuf::from(
    "tests/test_mylogin.cnf",
);

let client_info = myloginrs::parse("client", Some(&file_path));
```

Then you can use that HashMap with an `OptsBuilder` or other structs
from the [mysql](https://crates.io/crates/mysql):

```rust
let opts = OptsBuilder::new()
    .ip_or_hostname(Some(&client_info["host"]))
    .tcp_port(u16::from_str_radix(&client_info["port"], 10)?)
    .user(Some(&client_info["user"]))
    .pass(Some(&client_info["password"]));

let _conn = Conn::new(opts);
```

Starting with [mysql 20.1.0](https://crates.io/crates/mysql/20.1.0),
you can do the even simpler:

```rust
let opts = OptsBuilder::new().from_hash_map(&client_info).unwrap();
let _conn = Conn::new(opts);
```

If you would rather get a String that contains the whole file,
use read:

```rust
let mylogin_plaintext = myloginrs::read(None);

println!("{}", mylogin_plaintext);
```

This second example passes `None` as the path to use the
default .mylogin.cnf location (`%APPDATA%\MySQL\.mylogin.cnf` on windows or
`~/.mylogin.cnf` on everything else).


## Other Stuff

Thanks to
 * [github.com/PyMySQL](https://github.com/PyMySQL/myloginpath)
and
 * [github.com/ocelot-inc](https://github.com/ocelot-inc/ocelotgui/blob/master/readmylogin.c)
for doing all the hard work and from whom I port.


## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.


### Contribution

Pull requests welcome. :)

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
