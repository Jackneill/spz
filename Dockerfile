FROM rust:1.92-alpine3.23 as builder
LABEL stage="builder"

RUN apk add --no-cache \
	tzdata \
	ca-certificates \
	just \
	bash \
	git \
	openssh-client \
	mold \
	clang

ENV TZ=UTC

RUN mkdir -p -m 0700 ~/.ssh \
	&& ssh-keyscan github.com >> ~/.ssh/known_hosts \
	&& ssh-keyscan bitbucket.com >> ~/.ssh/known_hosts \
	&& ssh-keyscan bitbucket.org >> ~/.ssh/known_hosts \
	&& ssh-keyscan gitlab.com >> ~/.ssh/known_hosts

RUN git config --global url."git@bitbucket.org:".insteadOf "https://bitbucket.org/"

COPY . /app
WORKDIR /app

RUN --mount=type=ssh cargo build --release

FROM alpine:3.23.0

RUN apk add --no-cache \
	tzdata \
	ca-certificates

ENV TZ=UTC

COPY --from=builder /app/target/release/spz /app/
WORKDIR /app

RUN addgroup -S usergroup \
	&& adduser -S user -G usergroup
RUN chown -R user:usergroup /home/user/ \
	&& chown user:usergroup /app/spz
USER user

ENTRYPOINT [ "/app/spz" ]
