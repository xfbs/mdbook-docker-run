# Installation

This section shows you how to install `mdbook-docker-run` and its
prerequisites.

## Prerequisites

To use `mdbook-docker-run`, you need two prerequisites: [mdBook][mdbook] and
[Docker][docker]. You can use the installation instructions from their respective
websites. 

Typically, you can install mdBook using Cargo like this:

    cargo install mdbook

And you can install Docker using your operating system's package manager:

    apt install docker.io

You can verify that both are installed and working by running them:

    mdbook --version
    docker info

If both of these commands execute without failing, then you are all ready to
go.

## Installing mdBook Docker Run

To install `mdbook-docker-run`, you can use Cargo:

    cargo install mdbook-docker-run --locked

If you wish, you can install a specific version using `--version` followed by a
specific version, for example `--version 0.1.0`. By default it will install the
latest version.

[mdbook]: https://rust-lang.github.io/mdBook/
[docker]: https://docs.docker.com/
