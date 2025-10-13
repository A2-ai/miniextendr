# `miniextendr`

Experimental repository.

## Setup / Configuration

It is necessary to run

```shell
R CMD INSTALL .
```

to setup this project, as the `rpkg/configure` script in the embedded R-package
`{rpkg}` also sets up `cargo` configurations!

## Developer configuration

If you alter `configure.ac`, then a `/configure`-script has to be recompiled, and that is done via

```shell
autoreconf -vif
```

The `/configure`-script is supposed to be built during developer-time, and thus
users are not expected to run `autoconf`/`autoreconf`.
