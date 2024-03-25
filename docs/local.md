# Running locally

For running locally, this repository contains a `Justfile`. You can use
this to spawn a Docker-in-Docker instance, and build this documentation
using `mdbook-docker-run` using that DinD instance.

    just dind
    just docs-dind

This is a good way to test if changes work.
