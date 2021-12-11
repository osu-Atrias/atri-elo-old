# GET `/players/{player_id}`

Get the info of the specified player.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9"
}
```

## Example Response

### 200 OK

```json
{
    "id": 1,
    "name": "rushbee",
    "rating": 2493.1029,
    "rank": 1
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.

### 404 Not Found

There's no player with `player_id`.



# GET `/players/{player_id}/history`

Get the recent finished contest history of the specified player.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9",
    "limit": 1
}
```

In this example, the most recent finished contest of this player will be returned.

`limit` can be omitted (defaults to all).

## Example Response

### 200 OK

May return history in **any order**.

```json
[
    {
        "contest_id": 3,
        "performance": 115.123,
        "rating": 1124.98,
        "contest_rank": 1,
        "rating_rank": 1
    }
]
```

Only the contest that is closed and finished calculating will provide `performance`, `rating` and `rating_rank`.

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.

### 404 Not Found

There's no player with `player_id`.
