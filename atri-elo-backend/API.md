# atri-elo-backend APIs

## GET /leaderboard

Example Request:

```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:

### 200 OK

```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 11451,
        "name": "19190504 fanboy",
        "rating": 2942.114514,
        "rank": 4
    }
]
```

### 400 Bad Request

Wrong request format.

### 401 Unauthorized

Invalid api key.

## PUT /contest

Example Request:

```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2",
    "contest": {
        "name": "OWC 2077",
        "scores": [
            ["Jackie Welles", 1919],
            ["Johnny Silverhand", 1145141919810],
            ["V", 114514],
            ["Dexter DeShawn", 888888810]
        ]
    }
}
```

Example Response:

### 201 Created

```json
{
    "id": 3
}
```

### 400 Bad Request

Wrong request format.

### 401 Unauthorized

Invalid api key.

## GET /contest/list

Example Request:

```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:

### 200 OK

```json
[
    {
        "id": 1,
        "name": "OWC 2077",
    },
    {
        "id": 2,
        "name": "MP5 Derankers S5",
    }
]
```

### 400 Bad Request

Wrong request format.

### 401 Unauthorized

Invalid api key.

## GET /contest/{id}

Example Request:

```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:

### 200 OK

```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2942.114514,
        "rank": 4
    }
]
```

## DELETE /contest/{id}

Example Request:
```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:
```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2942.114514,
        "rank": 4
    }
]
```

## PATCH /contest/{id}

Example Request:
```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:
```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2942.114514,
        "rank": 4
    }
]
```

## GET /player/list

Example Request:
```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:
```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2942.114514,
        "rank": 4
    }
]
```

## GET /player/{id}/info

Example Request:
```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:
```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2942.114514,
        "rank": 4
    }
]
```

## GET /player/{id}/history

Example Request:
```json
{
    "api_key": "A73TH37F882NTFDVU28H8GN2"
}
```

Example Response:
```json
[
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2943.114514,
        "rank": 1
    },
    {
        "id": 11451,
        "name": "1919050",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 1145,
        "name": "191905",
        "rating": 2943.11451,
        "rank": 2
    },
    {
        "id": 114514,
        "name": "19190504",
        "rating": 2942.114514,
        "rank": 4
    }
]
```
