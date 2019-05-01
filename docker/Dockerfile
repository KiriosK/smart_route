FROM rustlang/rust:nightly

RUN USER=root cargo new --bin smart_route
WORKDIR /smart_route

RUN rustc --version

RUN cargo install diesel_cli --no-default-features --features postgres

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY . .

# build for release
RUN rm ./target/release/deps/smart_route*
RUN cargo build --release




# WORKDIR /var/www/smart_route

# RUN rustc --version

# RUN cargo install diesel_cli --no-default-features --features postgres

# COPY ./Cargo.lock ./Cargo.lock
# COPY ./Cargo.toml ./Cargo.toml

# RUN mkdir src/

# RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

# # this build step will cache your dependencies
# RUN cargo build --release
# RUN rm src/*.rs

# # copy your source tree
# COPY ./src ./src

# # build for release
# RUN cargo build