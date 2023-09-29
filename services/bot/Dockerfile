FROM rust:1.71.0-slim-bullseye as prereq

RUN rustup default nightly

RUN apt-get update
RUN apt-get install -y pkg-config libssl-dev

RUN cargo install cargo-chef

FROM prereq AS planner
WORKDIR /app

COPY crates crates
COPY sqlx-data.json sqlx-data.json
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo chef prepare --recipe-path recipe.json

FROM prereq as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --bin starboard --recipe-path recipe.json

FROM prereq as builder
WORKDIR /app

COPY crates crates
COPY sqlx-data.json sqlx-data.json
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# copy dependecies
COPY --from=cacher /app/target /app/target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release --bin starboard

# use googles distroless as runtime image
FROM gcr.io/distroless/cc-debian11
# copy app form builder
COPY --from=builder /app/target/release/starboard /

CMD ["./starboard"]
