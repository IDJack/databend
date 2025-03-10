#!/usr/bin/env bash

CURDIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
. "$CURDIR"/../../../shell_env.sh


echo "drop table if exists ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
## create ontime table
cat $CURDIR/../ddl/ontime.sql | sed 's/ontime/ontime_streaming_load/g' | $MYSQL_CLIENT_CONNECT


aws --endpoint-url http://127.0.0.1:9900/ s3 cp s3://testbucket/admin/data/ontime_200.csv /tmp/ontime_200.csv > /dev/null 2>&1
aws --endpoint-url http://127.0.0.1:9900/ s3 cp s3://testbucket/admin/data/ontime_200.csv.gz /tmp/ontime_200.csv.gz > /dev/null 2>&1
aws --endpoint-url http://127.0.0.1:9900/ s3 cp s3://testbucket/admin/data/ontime_200.csv.zst /tmp/ontime_200.csv.zst > /dev/null 2>&1
aws --endpoint-url http://127.0.0.1:9900/ s3 cp s3://testbucket/admin/data/ontime_200.csv.bz2 /tmp/ontime_200.csv.bz2 > /dev/null 2>&1
aws --endpoint-url http://127.0.0.1:9900/ s3 cp s3://testbucket/admin/data/ontime_200.parquet /tmp/ontime_200.parquet  > /dev/null 2>&1
aws --endpoint-url http://127.0.0.1:9900/ s3 cp s3://testbucket/admin/data/ontime_200.ndjson /tmp/ontime_200.ndjson  > /dev/null 2>&1


# do the Data integrity check
echo "33b1243ecd881e701a1c33cc8d621ecbf9817be006dce8722cfc6dd7ef0637f9 /tmp/ontime_200.csv" | sha256sum --check > /dev/null 2>&1
if [ $? -ne 0 ]; then
	echo "The downloaded dataset has been corrupted, please remove and fetch it again."
	exit 1
fi

echo "ace34c0f551f7cdaf8733deeb34e6bc2e19798512d3289f422c5209a3fccdf23 /tmp/ontime_200.parquet" | sha256sum --check > /dev/null 2>&1
if [ $? -ne 0 ]; then
	echo "The downloaded dataset has been corrupted, please remove and fetch it again."
	exit 1
fi

echo "8e6e663cf6fdaedf99516f8969512e40954bfa863822e8ef2d61e50c182c8d91 /tmp/ontime_200.ndjson" | sha256sum --check > /dev/null 2>&1
if [ $? -ne 0 ]; then
	echo "The downloaded dataset has been corrupted, please remove and fetch it again."
	exit 1
fi

# load csv
curl -H "insert_sql:insert into ontime_streaming_load format Csv" -H "skip_header:1" -F "upload=@/tmp/ontime_200.csv" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "truncate table ontime_streaming_load" | $MYSQL_CLIENT_CONNECT

# load csv gz
curl -H "insert_sql:insert into ontime_streaming_load format Csv" -H "skip_header:1" -H "compression:gzip" -F "upload=@/tmp/ontime_200.csv.gz" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "truncate table ontime_streaming_load" | $MYSQL_CLIENT_CONNECT

# load csv zstd
curl -H "insert_sql:insert into ontime_streaming_load format Csv" -H "skip_header:1" -H "compression:zstd" -F  "upload=@/tmp/ontime_200.csv.zst" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "truncate table ontime_streaming_load" | $MYSQL_CLIENT_CONNECT

# load csv bz2
curl -H "insert_sql:insert into ontime_streaming_load format Csv" -H "skip_header:1" -H "compression:bz2" -F  "upload=@/tmp/ontime_200.csv.bz2" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "truncate table ontime_streaming_load" | $MYSQL_CLIENT_CONNECT

# load parquet
curl -H "insert_sql:insert into ontime_streaming_load format Parquet" -H "skip_header:1" -F "upload=@/tmp/ontime_200.parquet" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "truncate table ontime_streaming_load" | $MYSQL_CLIENT_CONNECT

# load ndjson
curl -H "insert_sql:insert into ontime_streaming_load format NdJson" -H "skip_header:1" -F "upload=@/tmp/ontime_200.ndjson" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "truncate table ontime_streaming_load" | $MYSQL_CLIENT_CONNECT


# load parquet with less schema
echo 'CREATE TABLE ontime_less
(
    Year                            SMALLINT UNSIGNED,
    Quarter                         TINYINT UNSIGNED,
    Month                           TINYINT UNSIGNED,
    DayofMonth                      TINYINT UNSIGNED,
    DayOfWeek                       TINYINT UNSIGNED
)' | $MYSQL_CLIENT_CONNECT

curl -s -H "insert_sql:insert into ontime_less format Parquet" -H "skip_header:1" -F "upload=@/tmp/ontime_200.parquet" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load"  > /dev/null 2>&1
echo "select count(1), avg(Year), sum(DayOfWeek)  from ontime_less;" | $MYSQL_CLIENT_CONNECT

# load parquet with mismatch schema
cat $CURDIR/../ddl/ontime.sql | sed 's/ontime/ontime_test_mismatch/g' | sed 's/DATE/VARCHAR/g' | $MYSQL_CLIENT_CONNECT
curl -s -H "insert_sql:insert into ontime_test_mismatch format Parquet" -H "skip_header:1" -F "upload=@/tmp/ontime_200.parquet" -u root: -XPUT "http://localhost:${QUERY_HTTP_HANDLER_PORT}/v1/streaming_load" | grep -c 'Code: 1024'


echo "drop table ontime_streaming_load;" | $MYSQL_CLIENT_CONNECT
echo "drop table ontime_test_mismatch;" | $MYSQL_CLIENT_CONNECT
echo "drop table ontime_less;" | $MYSQL_CLIENT_CONNECT
