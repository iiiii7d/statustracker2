FROM rust

COPY . .

RUN cargo build --release &&\
    mv target/release/statustracker-server ./statustracker &&\
    cargo clean

ENV ROCKET_CONFIG=cfg/Rocket.toml
CMD ./statustracker cfg/statustracker.toml
EXPOSE 8000
