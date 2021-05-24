FROM node:14-alpine as builder
WORKDIR /usr/src/app
COPY package.json ./
COPY yarn.lock ./
RUN yarn install --frozen-lockfile
COPY tsconfig*.json ./
COPY src src
RUN yarn build
RUN yarn package

FROM alpine
ENV NODE_ENV=production
RUN apk add --no-cache tini
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/menu-proxy ./
ENV ADDRESS=0.0.0.0
ENV PORT=80
EXPOSE ${PORT}
ENTRYPOINT [ "/sbin/tini", "--", "./menu-proxy" ]
