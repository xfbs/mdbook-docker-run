# Syntax

Scripts are written as code blocks with the language set to `docker-run`.
For example, this is what a script block looks like:

~~~
```docker-run
image: alpine
script:
  - echo hi
```
~~~

The output of this looks like this:

```docker-run
image: alpine
script:
  - echo hi
```

The configuration for this is a YAML document. This section explains how the
syntax for this works.

## Full example

Here is a full example, along with comments:

```yaml
# name of docker image to use
image: alpine

# commands to execute before (not shown)
before_script:
  - rustup init

# script commands to run
script:
  - echo hi
  - cargo build

# whether to echo commands
echo: true

# which shell to use
shell: /bin/sh
```
