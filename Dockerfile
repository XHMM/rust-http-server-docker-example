ARG BUILD_TARGET=change-me-or-from-cli

FROM rust:1.75 as chef
COPY .cargo .cargo
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
ARG BUILD_TARGET
COPY .cargo .cargo
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release  --bin ${BUILD_TARGET}

# Distroless image doesn't have a shell, so it doesn't support docker exec as while as ctrl+c
# As it's not hosted on docker.io, you may need to configure proxy for build stage
FROM gcr.io/distroless/cc-debian12  AS runtime
ARG BUILD_TARGET
EXPOSE 8080
COPY --from=builder /app/target/release/${BUILD_TARGET} /usr/local/bin/app
# Distroless doesn't have a shell, you must use array-style to avoid the container runtime prefixing with a shell.
# CMD /usr/local/bin/app
CMD ["/usr/local/bin/app"]
