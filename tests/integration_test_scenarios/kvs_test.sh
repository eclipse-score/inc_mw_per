#!/bin/sh
set -eu


mkdir -p /var/db/kvs
cd /var/db/kvs


echo "[kvs] creating values"
kvs_tool -o setkey -k MyKey -p 'Hello World'
kvs_tool -o setkey -k MyKey -p 'true'
kvs_tool -o setkey -k MyKey -p 15
kvs_tool -o setkey -k MyKey -p '[456,false,"Second"]'
kvs_tool -o setkey -k MyKey -p '{"sub-number":789,"sub-array":[1246,false,"Fourth"]}'

echo "[kvs] reading"
kvs_tool -o getkey -k MyKey || exit 1

echo "[kvs] removing"
kvs_tool -o removekey -k MyKey

echo "[kvs] listing (expect no MyKey)"
kvs_tool -o listkeys | grep -v '^MyKey$' >/dev/null 2>&1 || {
  echo "MyKey still present after removal" >&2
  exit 1
}

echo "[kvs] done"
# clean exit so the test runner can detect success
/sbin/shutdown -f
