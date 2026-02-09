## `Cfg`

| Field | Type | Required | Description |
|---|---|---|---|
| `connectionTimeout` | [`DurationString`](#durationstring) |  | Connection timeout |
| `hosts` | `array` |  | List of hosts to index |
| `mode` | [`Mode`](#mode) |  | Indexing mode |


## `DurationString`

Base type: `string`


## `Mode`

| Variants |
|---|
| [`Serial`](#modeserial) |
| [`Parallel`](#modeparallel) |


## `Mode::Serial`

Index in a single thread

| Field | Type | Required | Description |
|---|---|---|---|
| `kind` | `string` | `V` |  |


## `Mode::Parallel`

Index in multiple threads

| Field | Type | Required | Description |
|---|---|---|---|
| `concurrency` | `integer` |  | Maximum concurrent requests |
| `kind` | `string` | `V` |  |
