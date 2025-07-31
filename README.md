This ReadMe is also available in [English](README-EN.md)

---

## Асинхронный JSON API сервер на Rust с Tokio и Serde JSON

### Описание

После запуска сервер начинает слушать локальный TCP-порт 7878. На вход принимаются JSON-запросы, состоящие из идентификатора запроса в формате UUID, команды и опциональных данных. Ответ сервера идентифицируется тем же ID и содержит статус запроса (`Ok`/`Error`) и результат выполнения команды либо описание ошибки соответственно.

Структура запроса:
```js
{
    "request_id": "UUID-строка",
    "command": "имя_команды",
    "payload": { ... }  // опциональные данные
}
```

Успешный ответ имеет следующую структуру:
```js
{
    "request_id": "UUID-строка",
    "status": "ok",
    "response": { ... } // результат выполнения команды
}
```

В случае же ошибки структура ответа меняется:
```js
{
    "request_id": "UUID-строка",
    "status": "error",
    "error": "описание_ошибки"
}
```

### Поддерживаемые команды

#### `ping`

Возвращает `{"response": "pong"}`.

#### `echo`

Возвращает содержимое `payload` запроса:
```js
// запрос
{
    ...
    "payload": {
        "number": 5,
        "nothing": null
    }
}

// ответ
{
    ...
    "response": {
        "number": 5,
        "nothing": null
    }
}
```

#### `time`

Возвращает текущее время по UTC в формате RFC 3339:
```js
{
    ...
    "response": {
        "time": "2025-07-16 17:45:45Z"
    }
}
```

#### `calculate`

Требует наличия поля `payload` и его соответствия заданному формату:
```js
{
    ...
    "payload": {
        "operation": "add|subtract|multiply|divide",
        "a": число,
        "b": число
    }
}
```
Возвращает результат указанной операции `operation` над операндами `a` и `b`:
```js
// запрос
{
    ...
    "payload": {
        "operation": "add",
        "a": 5,
        "b": 3
    }
}

// ответ
{
    ...
    "response": {
        "result": 8.0
    }
}
```

#### `batch`

Требует наличия поля `payload`. Содержимое поле должно быть массивом JSON-объектов, каждый из которых является отдельным запросом.
Служит для объединения нескольких команд в один запрос. Ответ сервера является массивом результатов на каждую из переданных команд:
```js
// запрос
{
    "request_id": "batch_id",
    "command": "batch",
    "payload": [
        {
            "request_id": "id1",
            "command": "ping"
        },
        {
            "request_id": "id2",
            "command": "time"
        },
        {
            "request_id": "id3",
            "command": "calculate",
            "payload": "неправильный_payload"
        }
    ]
}

// ответ
{
    "request_id": "batch_id",
    "status": "ok",
    "response": [
        {
            "request_id": "id1",
            "status": "ok",
            "response": "pong"
        },
        {
            "request_id": "id2",
            "status": "ok",
            "response": {
                "time": "2025-07-16 17:45:45Z"
            }
        },
        {
            "request_id": "id3",
            "status": "error",
            "error": "invalid type: string \"неправильный_payload\", expected struct CalculationPayload"
        }
    ]
}
```

### Установка

Загрузите исполняемый файл из [последнего релиза](https://github.com/rinkulu/testrust/releases/latest).

### Опции запуска

- `-d`/`--debug` - флаг, включающий логирование отладочных сообщений;

- `-l <FILE>`/`--log-file <FILE>` - позволяет задать файл, в который будут записываться логи.
Значение по-умолчанию: `default.log`
