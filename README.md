# mdBook Docker Run

[![crates.io](https://img.shields.io/crates/v/mdbook-docker-run.svg)](https://crates.io/crates/mdbook-docker-run)
[![CI Pipelines](https://gitlab.com/xfbs/mdbook-docker-run/badges/main/pipeline.svg)](https://gitlab.com/xfbs/mdbook-docker-run/-/pipelines)
[![Documentation](https://img.shields.io/badge/docs-nightly-brightgreen)](https://xfbs.gitlab.io/mdbook-docker-run)

Plugin for [mdBook][mdbook] which allows you to run commands inside Docker
containers and render their output inside your book. Use this if you want to
show examples for CLI-based workflows, for example.

## Example

If you put something like this into your book:

~~~
```docker-run
image: alpine
script:
  - echo hi
```
~~~

Will render as:

```
hi
```

## License

MIT.

[mdbook]: https://rust-lang.github.io/mdBook/
