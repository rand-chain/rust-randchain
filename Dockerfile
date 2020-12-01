FROM rust:1.45 as planner
WORKDIR /app
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.45 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.45 as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
RUN cargo build --release

FROM rust:1.45 as runtime
LABEL authors="Haoyu Lin and Runchao Han"
WORKDIR /app
COPY --from=builder /app/target/release/randchaind /bin/randchaind
# show backtraces
ENV RUST_BACKTRACE=1
ENV RUST_LOG=trace
EXPOSE 8333 18333 8332 18332
ENTRYPOINT ["/bin/randchaind"]