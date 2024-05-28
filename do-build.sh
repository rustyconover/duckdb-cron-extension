#!/bin/sh
set -e

export BUILD_ROOT=`pwd`

# It is assumed that the patched version of duckdb is built
# under this directory called duckdb

# It is also assumed the patched duckdb-rs is present in the
# current directory under duckdb-rs

export DUCKDB_LIB_DIR="$BUILD_ROOT/duckdb/build/debug/src/"
export DUCKDB_INCLUDE_DIR="$BUILD_ROOT/duckdb/src/include/"
cargo build


EXTENSION_FILE=./target/debug/libcrontab.duckdb_extension

cp ./target/debug/libcrontab.dylib $EXTENSION_FILE

/bin/echo -n "osx_arm64" > duckdb_platform_out

install_name_tool -add_rpath $BUILD_ROOT/duckdb/build/debug/src/ \
  $EXTENSION_FILE

# There needs to be some signing step performed here to add metadata to the extension.
cmake -DEXTENSION=$EXTENSION_FILE \
  -DDUCKDB_VERSION="caef2cd0c7" \
  -DEXTENSION_VERSION="0.0.1" \
  -DPLATFORM_FILE=./duckdb_platform_out \
  -DNULL_FILE=./duckdb/scripts/null.txt \
  -P ./duckdb/scripts/append_metadata.cmake

$BUILD_ROOT/duckdb/build/debug/duckdb \
  -unsigned \
  -c "load '$EXTENSION_FILE'; select * from cron('5 * * * *', start=get_current_timestamp(), until=date '2028-08-20'); "