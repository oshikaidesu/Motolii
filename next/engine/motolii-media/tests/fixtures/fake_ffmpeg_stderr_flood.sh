#!/bin/sh
cat >/dev/null
i=0
while [ "$i" -lt 20000 ]; do
  echo "err $i" 1>&2
  i=$((i+1))
done
exit 0
