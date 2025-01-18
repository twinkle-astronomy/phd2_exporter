FROM rust:1.80-bullseye as base

ARG USER=phd2_exporter
ARG USER_ID=1000
ARG GROUP_ID=1000
ARG TARGET=x86_64-unknown-linux-musl

RUN mkdir /app
WORKDIR /app

RUN apt-get update
RUN rustup target add ${TARGET}
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu

RUN groupadd -g ${GROUP_ID} ${USER} && \
    useradd -l -m -u ${USER_ID} -g ${USER} ${USER}

RUN chown ${USER}:${USER} /app
USER ${USER}

COPY Cargo.toml Cargo.lock /app/

FROM base as dev

USER root
RUN apt-get install -y --no-install-recommends \
    git \
    tig \
    vim \
    jq
USER ${USER}

FROM base as builder
COPY . /app
RUN cargo build --release --target ${TARGET}

FROM builder as tester
ENV TARGET=${TARGET}
CMD cargo test --release --target $TARGET

FROM builder as installer
RUN cargo install --target ${TARGET} --path .

FROM scratch as release
COPY --from=installer --chown=root:root /usr/local/cargo/bin/phd2_exporter /bin/phd2_exporter
ENTRYPOINT [ "/bin/phd2_exporter" ]
