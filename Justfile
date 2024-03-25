
dind_port := "2375"

dind:
    docker run -it --rm --privileged -p 127.0.0.1:{{dind_port}}:2375 -e DOCKER_TLS_CERTDIR="" docker:dind --tls=false

docs-dind:
    DOCKER_HOST=http://localhost:2375 mdbook build

docs:
    mdbook build
