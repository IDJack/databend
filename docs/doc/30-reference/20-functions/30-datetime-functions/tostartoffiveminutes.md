---
title: to_start_of_five_minutes
---

Rounds down a date with time to the start of the five-minute interval.
## Syntax

```sql
to_start_of_five_minutes( <expr> )
```

## Arguments

| Arguments   | Description |
| ----------- | ----------- |
| `<expr>` | datetime |

## Return Type

Datetime object, returns date in “YYYY-MM-DD hh:mm:ss” format.

## Examples

```sql
SELECT to_start_of_five_minutes(now());
+-----------------------------+
| to_start_of_five_minutes(now()) |
+-----------------------------+
| 2022-03-29 06:45:00         |
+-----------------------------+

SELECT to_start_of_five_minutes(to_datetime(1630812366));
+----------------------------------------------+
| to_start_of_five_minutes(to_datetime(1630812366)) |
+----------------------------------------------+
| 2021-09-05 03:25:00                          |
+----------------------------------------------+
```
