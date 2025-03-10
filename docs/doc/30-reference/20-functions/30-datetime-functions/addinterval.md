---
title: Add Time Interval
description: Add time interval function
title_includes: add_years, add_months, add_days, add_hours, add_minutes, add_seconds
---

Add time interval to a date or datetime, return the result of date or datetime type.
## Syntax

```sql
add_years(exp0, expr1)
add_months(exp0, expr1)
add_days(exp0, expr1)
add_hours(exp0, expr1)
add_minutes(exp0, expr1)
add_seconds(exp0, expr1)
```

## Return Type

Date, Timestamp, depends on the input.

## Examples

```sql
SELECT to_date(18875), add_years(to_date(18875), 2);
+---------------+-----------------------------+
| to_date(18875) | add_years(to_date(18875), 10) |
+---------------+-----------------------------+
| 2021-09-05    | 2023-09-05                  |
+---------------+-----------------------------+

SELECT to_date(18875), add_months(to_date(18875), 2);
+---------------+-----------------------------+
| to_date(18875) | add_months(to_date(18875), 2) |
+---------------+-----------------------------+
| 2021-09-05    | 2021-11-05                  |
+---------------+-----------------------------+

SELECT to_date(18875), add_days(to_date(18875), 2);
+---------------+---------------------------+
| to_date(18875) | add_days(to_date(18875), 2) |
+---------------+---------------------------+
| 2021-09-05    | 2021-09-07                |
+---------------+---------------------------+

SELECT to_datetime(1630833797), add_hours(to_datetime(1630833797), 2);
+------------------------+-------------------------------------+
| to_datetime(1630833797) | add_hours(to_datetime(1630833797), 2) |
+------------------------+-------------------------------------+
| 2021-09-05 09:23:17    | 2021-09-05 11:23:17                 |
+------------------------+-------------------------------------+

SELECT to_datetime(1630833797), add_minutes(to_datetime(1630833797), 2);
+------------------------+---------------------------------------+
| to_datetime(1630833797) | add_minutes(to_datetime(1630833797), 2) |
+------------------------+---------------------------------------+
| 2021-09-05 09:23:17    | 2021-09-05 09:25:17                   |
+------------------------+---------------------------------------+

SELECT to_datetime(1630833797), add_seconds(to_datetime(1630833797), 2);
+------------------------+---------------------------------------+
| to_datetime(1630833797) | add_seconds(to_datetime(1630833797), 2) |
+------------------------+---------------------------------------+
| 2021-09-05 09:23:17    | 2021-09-05 09:23:19                   |
+------------------------+---------------------------------------+
```
