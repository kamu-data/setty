## `MyConfig`

| Field | Type | Required | Description |
|---|---|---|---|
| `database` | [`DatabaseConfig`](#databaseconfig) |  | Database configuration |
| `encryption` | [`EncryptionConfig`](#encryptionconfig) |  | Optional encryption |


## `DatabaseConfig`

| Variants |
|---|
| [`Sqlite`](#databaseconfigsqlite) |
| [`Postgres`](#databaseconfigpostgres) |


## `DatabaseConfig::Sqlite`

Sqlite driver

| Field | Type | Required | Description |
|---|---|---|---|
| `database_path` | `string` |  | Path to the database file |
| `kind` | `string` | `V` |  |


## `DatabaseConfig::Postgres`

Postgres driver

| Field | Type | Required | Description |
|---|---|---|---|
| `schema_name` | `string` |  | Name of the schema |
| `host` | `string` |  | Host name |
| `kind` | `string` | `V` |  |


## `EncryptionConfig`

| Field | Type | Required | Description |
|---|---|---|---|
| `key` | `string` | `V` | Encryption key |
| `algo` | [`EncryptionAlgo`](#encryptionalgo) |  | Encryption algorythm |


## `EncryptionAlgo`

| Variants |
|---|
| `Aes` |
| `Rsa` |
