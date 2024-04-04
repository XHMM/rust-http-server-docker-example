# rust http server example

A dockerized rust http server, it implements an endpoint using tinyapi for image compression.

- Caching support by [cargo-chef](https://github.com/LukeMathWalker/cargo-chef/tree/main)
- Minimal image size and runtime by [distroless](https://github.com/GoogleContainerTools/distroless/tree/main)
- Cargo registry configured under `.cargo/config.toml`

## Build and run

### Build

You need to set the build argument `BUILD_TARGET` into your target name.

```shell
docker build --build-arg BUILD_TARGET=image-compress -t rust-http-demo .
```

### Run

Get tiny api key from the [official doc](https://tinypng.com/developers)

```shell
docker run -p 8080:8080 -e TINY_API_KEY=<YOUR KEY> rust-http-demo
```

### Visit

Health check:

```shell
curl http://localhost:8080/health
```

Compress:

```shell
curl -X POST --data-binary @image.jpg http://localhost:8080/compress -OJ
```

## Change cargo registry

Cargo registry was changed for China users, if yor network works well with the default cargo registry, you can remove `source` config in the `.cargo/config.toml` file.
