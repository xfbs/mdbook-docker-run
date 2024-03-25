# mdBook Docker Run

[![crates.io](https://img.shields.io/crates/v/mdbook-docker-run.svg)](https://crates.io/crates/mdbook-docker-run)
[![CI Pipelines](https://gitlab.com/xfbs/mdbook-docker-run/badges/main/pipeline.svg)](https://gitlab.com/xfbs/mdbook-docker-run/-/pipelines)
[![Documentation](https://img.shields.io/badge/docs-nightly-brightgreen)][book]

Plugin for [mdBook][mdbook] which allows you to run commands inside Docker
containers and render their output inside your book. Use this if you want to
show examples for CLI-based workflows, for example.

## Install

You can install this plugin using `cargo`:

    cargo install mdbook-docker-run --locked

This should get you whatever is the most recent release. If you want to, you
can also specify an exact version using `--version 0.1.0`, for example.

## Example

If you put something like this into your book:

~~~
```docker-run
image: alpine
script:
  - echo hi
```
~~~

Will run the command `echo hi` in a an `alpine` Docker container, and render
the output as a code block. The output will look like this:

```
hi
```

For more information, check out the [book][]. This also explains how you can
override how the output is formatted.

## License

MIT.

[mdbook]: https://rust-lang.github.io/mdBook/
[book]: https://xfbs.gitlab.io/mdbook-docker-run
