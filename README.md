<p align="center">
  <h1 align="center">Menu Proxy</h1>
</p>

<p align="center">
  <a href="https://codecov.io/gh/inteskolplattformen/menu-proxy">
    <img src="https://codecov.io/gh/inteskolplattformen/menu-proxy/branch/main/graph/badge.svg?token=rUytc5q58t"/>
  </a>
</p>

Menu Proxy aggregates the various cafeteria menus from Swedish schools.

## API reference

### Menu object

| Field | Type   | Description       | Example                    |
| ----- | ------ | ----------------- | -------------------------- |
| id    | string | unique menu id    | `"sodexo.aaa-bbb-ccc-ddd"` |
| title | string | title of the menu | `"Södermalmsskolan"`       |

### Meal object

| Field | Type   | Description | Example           |
| ----- | ------ | ----------- | ----------------- |
| value | string | meal value  | `"Fisk Björkeby"` |

### Day object

| Field | Type                                                    | Description | Example        |
| ----- | ------------------------------------------------------- | ----------- | -------------- |
| date  | [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601) date | date        | `"2020-06-08"` |
| meals | array of [meal](#meal-object) objects                   | meals       |                |

### Get menus

`GET /menus`

Returns all the menus. The requests often take a long time to complete (and puts a strain on upstream servers), so the responses should be cached for around 24 hours.

```json
[
	{ "id": "sodexo.0", "title": "Skola 0" },
	{ "id": "sodexo.1", "title": "Skola 1" },
	{ "id": "sodexo.2", "title": "Skola 2" },
	{ "id": "sodexo.3", "title": "Skola 3" }
]
```

### Get menu

`GET /menus/{menu.id}`

Returns a single [menu](#menu-object).

### Get menu days

`GET /menus/{menu.id}/days`

Get the [days](#day-object) of a specific menu.

#### Query string parameters

| Field | Type           | Description                        | Default                  |
| ----- | -------------- | ---------------------------------- | ------------------------ |
| first | ?ISO 8601 date | date of the first day (inclusive). | the current date (UTC)   |
| last  | ?ISO 8601 date | date of the last day (inclusive).  | `?first` plus four weeks |

### Health check

`GET /health`

Returns `200 OK` if the API is healthy.

## Getting started

The easiest way to get started is to run this program with Docker:

```
docker run -d -p 8000:80 ghcr.io/inteskolplattformen/menu-proxy
```

## Configuring

Use environment variables to configure Menu Proxy.

| Environment variable | Description              | Default   |
| -------------------- | ------------------------ | --------- |
| `PORT`               | What port to listen to.  | `8000`    |
| `ADDRESS`            | What address to bind to. | `0.0.0.0` |
