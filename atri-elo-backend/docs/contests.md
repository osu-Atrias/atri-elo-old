# GET `/contests`

Get a list of all contests.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9"
}
```

## Example Response

May return contests in **any order**.

### 200 OK

```json
[
    {
        "id": 1,
        "name": "OWC 2077",
        "status": 3,
        "start_time": "2021-12-07 14:00:00.0 +00:00:00",
        "end_time": "2021-12-12 14:00:12.179165142 +00:00:00"
    },
    {
        "id": 2,
        "name": "OWC 2078",
        "status": 2,
        "start_time": "2021-12-07 14:00:00.0 +00:00:00",
        "end_time": "2021-12-12 14:00:12.179165142 +00:00:00"
    },
    {
        "id": 3,
        "name": "OWC 2079",
        "status": 0,
        "start_time": "2021-12-07 14:00:00.0 +00:00:00",
        "end_time": "2021-12-12 14:00:12.179165142 +00:00:00"
    }
]
```

| `status` | Description |
| --- | --- |
| 0 | The contest is open. |
| 1 | The contest is temporarily closed. |
| 2 | The contest is closed and being calculated. |
| 3 | The contest is finished. |

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.



# POST `/contests`

Start a new contest.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9",
    "name": "OWC 2080",
    "start_time": "2021-12-07 14:00:00.0 +00:00:00",
    "duration": 432012.179165143,
}
```

`duration` is represented by a double-precision float number as seconds.

## Example Response

### 201 Created

```json
{
    "id": 4,
    "name": "OWC 2080",
    "status": 0,
    "start_time": "2021-12-07 14:00:00.0 +00:00:00",
    "end_time": "2021-12-12 14:00:12.179165142 +00:00:00"
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.



# GET `/contests/{contest_id}`

Get general information of the specified contest.

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
    "id": 4,
    "name": "OWC 2080",
    "status": 0,
    "start_time": "2021-12-07 14:00:00.0 +00:00:00",
    "end_time": "2021-12-12 14:00:12.179165142 +00:00:00"
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.

### 404 Not Found

There's no contest with `contest_id`.



# PATCH `/contests/{contest_id}`

Modify the info of the specified contest.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9",
    "status": 1,
    "duraion": 2.0,
}
```

Only accepts changes of `status`, `duration` and/or `start_time`.

## Example Response

### 200 OK

```json
{
    "id": 4,
    "name": "OWC 2080",
    "status": 1,
    "start_time": "2021-12-07 14:00:00.0 +00:00:00",
    "end_time": "2021-12-07 14:00:02.0 +00:00:00"
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.

If you try to "reopen" a contest, this status code is also returned.

### 404 Not Found

There's no contest with `contest_id`.



# DELETE `/contests/{contest_id}`

Delete a contest.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9"
}
```

## Example Response

### 204 No Content

Successfully deleted the contest.

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

`api_key` is not provided.

### 403 Forbidden

`api_key` is provided but does not have permission.

### 404 Not Found

There's no contest with `contest_id`.



# GET `/contests/{contest_id}/scores`

Get scores of the specified contest.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9"
}
```

## Example Response

### 200 OK

```json
[
    {
        "id": 1,
        "name": "rushbee",
        "score": 1231235,
    },
    {
        "id": 2,
        "name": "rushbee fanboy",
        "score": 12312352,
    },
    {
        "id": 4,
        "name": "WhiteCai",
        "score": 121235,
    },
    {
        "id": 3,
        "name": "WhiteCar",
        "score": 12335,
    }
]
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

api_key is not provided.

### 403 Forbidden

api_key is provided but does not have permission.

### 404 Not Found

There's no contest with `contest_id`.



# GET `/contests/{contest_id}/scores/{player_id}`

Get the score of the specified player in the specified contest.

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
    "score": 1231235,
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

api_key is not provided.

### 403 Forbidden

api_key is provided but does not have permission.

### 404 Not Found

There's no score with `contest_id` or `player_id`.



# PUT `/contests/{contest_id}/scores/{player_id}`

Upload a score for the specified player.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9",
    "score": 141414
}
```

## Example Response

### 201 Created

```json
{
    "id": 5,
    "name": "u",
    "score": 141414,
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

api_key is not provided.

### 403 Forbidden

api_key is provided but does not have permission.

If the contest is closed, this status code is also returned.

### 404 Not Found

There's no contest with `contest_id`.

### 409 Conflict

The player has already been uploaded.

Use PATCH to update the score.



# PATCH `/contests/{contest_id}/scores/{player_id}`

Upload a score for the specified player.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9",
    "score": 141414
}
```

## Example Response

### 201 Created

```json
{
    "id": 5,
    "name": "u",
    "score": 141414,
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

api_key is not provided.

### 403 Forbidden

api_key is provided but does not have permission.

If the contest is closed, this status code is also returned.

### 404 Not Found

There's no score with `contest_id` or `player_id`.



# GET `/contests/{contest_id}/results`

Get results of the specified contest.

## Example Request

```json
{
    "api_key": "DNVUC45B8G4H94BN9"
}
```

## Example Response

### 200 OK

```json
[
    {
        "id": 1,
        "name": "rushbee",
        "performance": 115.123,
        "rating": 1124.98,
        "contest_rank": 1,
        "rating_rank": 4
    },
    {
        "id": 2,
        "name": "rushbee fanboy",
        "performance": 115.123,
        "rating": 1124.98,
        "contest_rank": 2,
        "rating_rank": 3
    },
    {
        "id": 4,
        "name": "WhiteCai",
        "performance": 115.123,
        "rating": 1124.98,
        "contest_rank": 3,
        "rating_rank": 2
    },
    {
        "id": 3,
        "name": "WhiteCar",
        "performance": 115.123,
        "rating": 1124.98,
        "contest_rank": 1,
        "rating_rank": 4
    }
]
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

api_key is not provided.

### 403 Forbidden

api_key is provided but does not have permission.

### 404 Not Found

There's no contest with `contest_id`.

If the contest is not calculated, this status code is also returned.



# GET `/contests/{contest_id}/results/{player_id}`

Get the score of the specified player in the specified contest.

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
    "performance": 115.123,
    "rating": 1124.98,
    "contest_rank": 1,
    "rating_rank": 4
}
```

### 400 Bad Request

Invalid request format.

### 401 Unauthorized

api_key is not provided.

### 403 Forbidden

api_key is provided but does not have permission.

### 404 Not Found

There's no score with `contest_id` or `player_id`.

If the contest is not calculated, this status code is also returned.
