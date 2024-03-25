# Configuration

To use `mdbook-docker-run`, you need to have an `mdbook` project. If you do not
already have one, you can create one using `mdbook init`. Refer to the `mdbook`
[documentation][mdbook-docs] for information on how to get started with it.

[mdbook-docs]: https://rust-lang.github.io/mdBook/format/configuration/index.html

## Simple

To tell `mdbook` about the existence of `mdbook-docker-run`, you need to add a
section to your `book.toml` for it.

In general, this section should look like this:

```toml
[preprocessor.docker-run]
command = "mdbook-docker-run"
```

With this setup, you should be ready to go. Running `mdbook build` should
run without failure. You can verify that your installation works by adding
a sample command block somewhere in the book sources, for example this:

~~~
```docker-run
image: alpine
script:
  - echo hi
```
~~~

And building your book, ensuring that it contains a block with `hi`.

## Configuration options

There are further configuration options that you can set here. Here is a complete
example with all possible configuration options, and what they do:

```toml
[preprocessor.docker-run]
# path to mdbook-docker-run
command = "mdbook-docker-run"

# override how to connect to docker.
# by default, it respects DOCKER_HOST
docker = "tcp://127.0.0.1:2375"

# default docker image to use
image = "alpine"

# prefix to use for all paths
prefix = "examples/"

# how many executions to run in parallel
parallel = 2
```
