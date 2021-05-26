<p align="center">
  <h1 align="center">Menu Proxy</h1>
</p>

<p align="center">
  <a href="https://codecov.io/gh/inteskolplattformen/menu-proxy">
    <img src="https://codecov.io/gh/inteskolplattformen/menu-proxy/branch/main/graph/badge.svg?token=rUytc5q58t"/>
  </a>
</p>

Menu Proxy aggregates the various cafeteria menus from Swedish schools.

## Reference

### Terminology

- A **school** is associated with one or more **menus**.
- A **menu** consists of many **days**.
- A day lists the meals served for a specific date on a particular menu.

### API

_Sorry for the poor documentation. It will be fixed._

- List all menus at `/menus`.
- Query individual menus at `/menus/:menuId`.
- Query the days of a menu at `/menus/:menuId/days`. Use query parameters `?first` and `?last` (ISO8601 dates) to specify the range.

## Getting started

The easiest way to get started is to run this program with Docker:

```
docker run -d -p 8000:80 ghcr.io/inteskolplattformen/menu-proxy:<VERSION>
```

## Customizing

### Environment variables

| Variable  | Description              | Default   |
| --------- | ------------------------ | --------- |
| `PORT`    | What port to listen to.  | `8000`    |
| `ADDRESS` | What address to bind to. | `0.0.0.0` |
