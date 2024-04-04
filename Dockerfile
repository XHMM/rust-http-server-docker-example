FROM rust:1.75 as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY .cargo .cargo
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
ARG BUILD_TARGET
RUN echo ${BUILD_TARGET}
COPY . .
RUN cargo build --release  --bin ${BUILD_TARGET}

# Distroless image doesn't have a shell, so it doesn't support docker exec as while as ctrl+c
# As it's not hosted on docker.io, you may need to configure proxy for build stage
FROM gcr.io/distroless/cc-debian12  AS runtime
EXPOSE 8080
ARG BUILD_TARGET
COPY --from=builder /app/target/release/${BUILD_TARGET} /usr/local/bin/app
# Distroless doesn't have a shell, you must use array-style to avoid the container runtime prefixing with a shell.
# CMD /usr/local/bin/app
CMD ["/usr/local/bin/app"]
