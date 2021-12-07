# GET `/leaderboard`

Get current leaderboard.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9",
    "limit": 100
}
```

`limit` can be omitted (defaults to 100).

## Example Response

May return players in **any order**.
May return players counting **more than** `limit`.

### 200 OK

```json
[
    {
        "id": 1,
        "name": "rushbee",
        "rating": 2493.1029,
        "rank": 1
    },
    {
        "id": 2,
        "name": "rushbee fanboy",
        "rating": 2200.102,
        "rank": 4
    },
    {
        "id": 4,
        "name": "WhiteCai",
        "rating": 2300.1029,
        "rank": 3
    },
    {
        "id": 3,
        "name": "WhiteCar",
        "rating": 2493.1,
        "rank": 2
    }
]
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.
