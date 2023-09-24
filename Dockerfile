FROM rust:1.72-buster as base
RUN rustup target add x86_64-unknown-linux-musl

RUN mkdir /app
WORKDIR /app

ARG USER=phd2_exporter
ARG USER_ID=1000
ARG GROUP_ID=1000

RUN groupadd -g ${GROUP_ID} ${USER} && \
    useradd -l -m -u ${USER_ID} -g ${USER} ${USER}

RUN chown ${USER}:${USER} /app
USER ${USER}

COPY Cargo.toml Cargo.lock /app/

FROM base as dev

USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    tig \
    vim \
    && rm -rf /var/lib/apt/lists/*
USER ${USER}

FROM base as builder
COPY . /app
RUN cargo build --release --target x86_64-unknown-linux-musl 

FROM builder as tester
CMD ["cargo", "test", "--release", "--target", "x86_64-unknown-linux-musl"]

FROM builder as installer
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch as release
COPY --from=installer --chown=root:root /usr/local/cargo/bin/phd2_exporter /bin/phd2_exporter
ENTRYPOINT [ "/bin/phd2_exporter" ]
