FROM docker.io/library/node:lts-alpine AS frontend

WORKDIR /app/backend/web

# update corepack
RUN npm install --global corepack@latest
# Install pnpm
RUN corepack enable pnpm

COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install

COPY web ./
RUN pnpm build

FROM docker.io/library/golang:1.25.6-alpine AS backend

WORKDIR /app/backend/web/dist
COPY --from=frontend /app/backend/web/dist .

WORKDIR /app/backend

COPY go.mod go.sum ./
RUN go mod download

# copies over web again...
COPY . .
RUN CGO_ENABLED=0 GOOS=linux \
  go build \
  -o dist/ \
  github.com/kaaax0815/smart-hot-water-tank/backend/cmd/backend

FROM gcr.io/distroless/static-debian13
WORKDIR /etc/backend

COPY --from=backend /app/backend/dist /usr/bin/
EXPOSE 8080/tcp

ENTRYPOINT ["/usr/bin/backend"]