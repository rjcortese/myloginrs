# myloginrs
[![Build Status](https://travis-ci.com/rjcortese/myloginrs.svg?branch=master)](https://travis-ci.com/rjcortese/myloginrs)
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
let client_info = myloginrs::parse("client", None);

let mut opts = OptsBuilder::new();
opts.ip_or_hostname(Some(opts.host));
opts.tcp_port(opts.port);
opts.user(Some(opts.user));
opts.pass(Some(opts.pass));

let conn = Conn::new(opts)?;
```

If you would rather get a String that contains the whole file,
use read:

```rust
let mylogin_plaintext = myloginrs::read(None);

println!("{}", mylogin_plaintext);
```

Both of these examples pass `None` as the path to use the
default .mylogin.cnf location (`%APPDATA%\MySQL\.mylogin.cnf` on windows or 
`~/.mylogin.cnf` on everything else).

## other stuff
Thanks to
 * [github.com/PyMySQL](https://github.com/PyMySQL/myloginpath)
and
 * [github.com/ocelot-inc](https://github.com/ocelot-inc/ocelotgui/blob/master/readmylogin.c)
for doing all the hard work and from whom I port.

Pull requests welcome. :)
