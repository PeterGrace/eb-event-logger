commit := `git rev-parse HEAD`
shortcommit := `git rev-parse HEAD`
transport := "docker://"
registry := "docker.io"
image := "petergrace/eb-event-logger"
tag := `git describe --tags || echo "dev"`

all: build make-image

build:
  cross build --release --target x86_64-unknown-linux-gnu
make-image:
  docker buildx build --push --platform linux/amd64 \
  -t {{registry}}/{{image}}:latest \
  -t {{registry}}/{{image}}:{{shortcommit}} \
  -t {{registry}}/{{image}}:{{commit}} \
  -t {{registry}}/{{image}}:{{tag}} \
  .

release-patch:
  cargo release --no-publish --no-verify patch --execute
release-minor:
  cargo release --no-publish --no-verify minor --execute
release-major:
  cargo release --no-publish --no-verify minor --execute
