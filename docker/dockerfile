FROM amazonlinux:latest AS builder

RUN yum -y groupinstall "Development Tools"

ENV BUILD_DIR=/build \
    OUTPUT_DIR=/output \
    RUST_BACKTRACE=1 \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    PREFIX=/musl \
    MUSL_VERSION=1.1.19

RUN mkdir -p /usr/local/cargo/bin \
  && mkdir -p $BUILD_DIR \
  && mkdir -p $OUTPUT_DIR \
  && mkdir -p $PREFIX

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

ENV BUILD_TARGET=x86_64-unknown-linux-musl
RUN rustup target add $BUILD_TARGET

WORKDIR $PREFIX

# Build any dependencies that aren't part of your build, e.g. thrift compiler

# Build Musl
RUN curl -O http://www.musl-libc.org/releases/musl-$MUSL_VERSION.tar.gz
RUN tar -xvzf musl-$MUSL_VERSION.tar.gz \
    && cd musl-$MUSL_VERSION \
    && ./configure --prefix=$PREFIX \
    && make install \
    && cd ..

# Set environment for musl
ENV CC=$PREFIX/bin/musl-gcc \
    C_INCLUDE_PATH=$PREFIX/include/ \
    CPPFLAGS=-I$PREFIX/include \
    LDFLAGS=-L$PREFIX/lib

WORKDIR $BUILD_DIR

RUN mkdir .cargo
ADD docker/cargo_config .cargo/config

# Install literally anything to download the cargo index and cache it in a docker layer
RUN cargo install serde || true

ADD ./Cargo.toml .
RUN mkdir src && touch src/lib.rs
RUN cargo build --target $BUILD_TARGET --release

ADD ./src src
RUN cargo build --target $BUILD_TARGET --release --bin kirby-s3

RUN find target/$BUILD_TARGET/release -maxdepth 1 -type f -executable -exec cp '{}' $OUTPUT_DIR \;

FROM amazonlinux:latest AS package

RUN yum -y install zip
ENV OUTPUT_DIR=/output \
    ARTIFACTS_DIR=/artifacts

RUN mkdir -p $ARTIFACTS_DIR

COPY --from=builder $OUTPUT_DIR $ARTIFACTS_DIR

WORKDIR $ARTIFACTS_DIR

RUN find . -maxdepth 1 -type f -executable -exec zip aws_lambda.zip '{}' \;

RUN ls -a $ARTIFACTS_DIR

FROM package

ENV ARTIFACTS_DIR=/artifacts \
    EXPORT_DIR=/export

RUN mkdir -p $EXPORT_DIR

#Snapshot the directory
VOLUME $EXPORT_DIR

CMD find $ARTIFACTS_DIR -type f -name "aws_lambda.zip" -exec cp '{}' $EXPORT_DIR \;