FROM rust

COPY . .

RUN cargo build --release &&\
    mv target/release/statustracker-server ./statustracker &&\
    cargo clean

CMD ["./statustracker"]
