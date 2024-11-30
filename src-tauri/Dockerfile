FROM rust:latest

WORKDIR /app
RUN apt update
RUN apt install -y libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
COPY Cargo.toml ./
COPY ./fdrop-common/Cargo.toml ./fdrop-common/Cargo.toml
COPY ./fdrop-discovery/Cargo.toml ./fdrop-discovery/Cargo.toml
COPY ./fdrop-config/Cargo.toml ./fdrop-config/Cargo.toml
RUN bash -c "mkdir {.,fdrop-common,fdrop-discovery,fdrop-config}/src"
RUN bash -c "touch {.,fdrop-common,fdrop-discovery,fdrop-config}/src/lib.rs"
RUN cargo build

COPY ./fdrop-common ./fdrop-common
COPY ./fdrop-discovery ./fdrop-discovery
COPY ./fdrop-config ./fdrop-config
COPY ./src ./src
CMD cd fdrop-discovery && cargo run --example create_peer
