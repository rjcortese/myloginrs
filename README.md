# myloginrs
![](https://github.com/rjcortese/myloginrs/workflows/ci/badge.svg)
[![](http://meritbadge.herokuapp.com/myloginrs)](https://crates.io/crates/myloginrs)

Read and parse MySQL's
[.mylogin.cnf](https://dev.mysql.com/doc/refman/5.7/en/mysql-config-editor.html)
file.

### Usage

Add `myloginrs` to `Cargo.toml`:

```toml
[dependencies]
myloginrs = "0.1"
```

### Example

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

If you would rather get a String that contains the whole file,
use read:

```rust
let mylogin_plaintext = myloginrs::read(None);

println!("{}", mylogin_plaintext);
```

This second example passes `None` as the path to use the
default .mylogin.cnf location (`%APPDATA%\MySQL\.mylogin.cnf` on windows or 
`~/.mylogin.cnf` on everything else).

## other stuff
Thanks to
 * [github.com/PyMySQL](https://github.com/PyMySQL/myloginpath)
and
 * [github.com/ocelot-inc](https://github.com/ocelot-inc/ocelotgui/blob/master/readmylogin.c)
for doing all the hard work and from whom I port.

Pull requests welcome. :)
