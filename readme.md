# Cron Expressions For DuckDB

## Motivation

Cron jobs are a fundamental tool for scheduling tasks, often used for analyzing system behavior or verifying the completeness of data collections. For instance, a task might be scheduled to run daily at 10:30 AM, while another runs every Tuesday at 3 PM. These times might be in different time zones, such as UTC or New York time.

To efficiently compare these scheduled tasks against actual data, it’s beneficial to generate a list of expected timestamps directly within DuckDB. While DuckDB's `generate_series()` function can create a series of timestamps, it lacks the expressiveness needed for complex recurring patterns. I want to fill this with a new DuckDB extension that interprets cron expressions, leveraging the Rust crate [`croner`](https://crates.io/crates/croner).

This extension introduces a table-returning function `cron()`, which calculates upcoming or past times that satisfy a given cron expression.

## Examples

#### Basic Usage

The `cron()` function returns the next timestamp for a given cron expression:

```sql
-- This expression occurs every day at 5 AM.
SELECT * FROM cron('0 5 * * *');
┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2024-05-26 05:00:00 │
└─────────────────────┘
```

#### Future Timestamps

To retrieve all future occurrences up to a specific date:

```sql
select * from cron('0 5 * * *', until='2024-06-05');
┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2024-05-26 05:00:00 │
│ 2024-05-27 05:00:00 │
│ 2024-05-28 05:00:00 │
│ 2024-05-29 05:00:00 │
│ 2024-05-30 05:00:00 │
│ 2024-05-31 05:00:00 │
│ 2024-06-01 05:00:00 │
│ 2024-06-02 05:00:00 │
│ 2024-06-03 05:00:00 │
│ 2024-06-04 05:00:00 │
├─────────────────────┤
│       10 rows       │
└─────────────────────┘
````

#### Past Timestamps

To retrieve occurrences within a past date range:

```sql
select * from cron('0 5 * * *', until='2020-08-04', start='2020-08-01');
┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2020-08-01 05:00:00 │
│ 2020-08-02 05:00:00 │
│ 2020-08-03 05:00:00 │
└─────────────────────┘
```

#### Timezone Handling

Cron expressions can be evaluated in specific time zones:

```sql
select * from
cron(
  '0 5 * * *',
  until='2024-03-14 00:00:00',
  start='2024-03-08 01:58:00',
  timezone='America/New_York');

┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2024-03-08 10:00:00 │
│ 2024-03-09 10:00:00 │
│ 2024-03-10 09:00:00 │
│ 2024-03-11 09:00:00 │
│ 2024-03-12 09:00:00 │
│ 2024-03-13 09:00:00 │
└─────────────────────┘
```

Changing DuckDB's timezone to match.

```sql
set timezone to 'America/New_York';

select * from
cron(
  '0 5 * * *',
  until='2024-03-14 00:00:00',
  start='2024-03-08 01:58:00',
  timezone='America/New_York');
┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2024-03-08 10:00:00 │
│ 2024-03-09 10:00:00 │
│ 2024-03-10 09:00:00 │
│ 2024-03-11 09:00:00 │
│ 2024-03-12 09:00:00 │
│ 2024-03-13 09:00:00 │
└─────────────────────┘
```

#### Second Level Precision

```sql
select * from cron('*/3 0 5 * * *', until='2024-05-30') limit 5;
┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2024-05-26 05:00:00 │
│ 2024-05-26 05:00:03 │
│ 2024-05-26 05:00:06 │
│ 2024-05-26 05:00:09 │
│ 2024-05-26 05:00:12 │
└─────────────────────┘
```

#### More complicated cron expressions

Using cron expressions for specific days of the week:

```sql
select * from cron('0 5 * * Mon', until='2024-09-30') limit 5;
┌─────────────────────┐
│        cron         │
│     timestamp_s     │
├─────────────────────┤
│ 2024-05-27 05:00:00 │
│ 2024-06-03 05:00:00 │
│ 2024-06-10 05:00:00 │
│ 2024-06-17 05:00:00 │
│ 2024-06-24 05:00:00 │
└─────────────────────┘
```

## Function Documentation

### `cron(VARCHAR, start=TIMESTAMP, until=TIMESTAMP, timezone=VARCHAR)`

#### Parameters:

* `pattern` (VARCHAR): The cron pattern to evaluate.

#### Optional Named Parameters:

* `start` (TIMESTAMP): The timestamp at which to begin evaluating the cron pattern.
* `until` (TIMESTAMP): The timestamp at which to stop evaluating the cron pattern (exclusive).
* `timezone` (VARCHAR): The time zone in which to evaluate the cron pattern (e.g., 'America/New_York', 'America/Chicago').

#### Returning

A single column `cron`, which contains timestamps when the cron pattern is satisfied.
