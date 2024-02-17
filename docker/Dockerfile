FROM alpine:3.16.6 as builder

ARG VERSION
ARG TARGETPLATFORM

RUN if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
        echo "aarch64" > arch; \
        echo "musl" > env; \
    elif [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
        echo "x86_64" > arch; \
        echo "musl" > env; \
    elif [ "${TARGETPLATFORM}" = "linux/arm/v7" ]; then \
        echo "armv7" > arch; \
        echo "musleabi" > env; \
    elif [ "${TARGETPLATFORM}" = "linux/arm/v6" ]; then \
        echo "arm" > arch; \
        echo "musleabi" > env; \
    fi
RUN apk update && apk add wget
RUN wget https://github.com/gngpp/ninja/releases/download/v0.9.36/ninja-0.9.36-x86_64-unknown-linux-musl.tar.gz
RUN tar -xvf ninja-0.9.36-x86_64-unknown-linux-musl.tar.gz

FROM alpine:3.16.6

LABEL org.opencontainers.image.authors "gngpp <gngppz@gmail.com>"
LABEL org.opencontainers.image.source https://github.com/gngpp/ninja
LABEL name ninja
LABEL url https://github.com/gngpp/ninja

ENV LANG=C.UTF-8 DEBIAN_FRONTEND=noninteractive LANG=zh_CN.UTF-8 LANGUAGE=zh_CN.UTF-8 LC_ALL=C

COPY --from=builder /ninja /bin/ninja

ENTRYPOINT ["/bin/ninja"]
