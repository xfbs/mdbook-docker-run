# mdbook-docker-run

Plugin for [mdBook][mdbook] which allows you to run commands inside Docker
containers and render their output inside your book. Use this if you want to
show examples for CLI-based workflows, for example.

## Example

If you put something like this into your book:

~~~
```docker-run
image = "alpine"
script = [
    "echo hi",
]
```
~~~

It will render as:

```
hi
```

## License

MIT.

[mdbook]: https://rust-lang.github.io/mdBook/

