<p align="center">
  <h1 align="center">Menu Proxy</h1>
</p>

<p align="center">
  <a href="https://codeclimate.com/github/inteskolplattformen/menu-proxy/test_coverage">
    <img src="https://api.codeclimate.com/v1/badges/bc6aeee3f21af2b46e7a/test_coverage" />
  </a>
</p>

An API used to digest the various cafeteria menus from Swedish schools.

## API reference

The API is far from complete, and breaking changes should be expected. Therefore, we provide no documentation at the moment.

## Getting started

The easiest way to get started is to clone this repository and then run this program with Docker:

```
docker run -d -p 8000:80 ghcr.io/inteskolplattformen/menu-proxy:<VERSION>
```

## Customizing

### Environment variables

| Variable  | Description              | Default   |
| --------- | ------------------------ | --------- |
| `PORT`    | What port to listen to.  | `8000`    |
| `ADDRESS` | What address to bind to. | `0.0.0.0` |
