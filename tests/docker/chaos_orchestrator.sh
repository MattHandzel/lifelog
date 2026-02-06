#!/bin/sh
## Tier 2 chaos orchestrator
## Runs through phases: normal → latency → packet loss → partition → verify
##
## Uses the Toxiproxy HTTP API at toxiproxy:8474

set -e

TOXI="http://toxiproxy:8474"

echo "=== Phase 1: Normal sync (30s) ==="
sleep 30

echo "=== Phase 2: Add 500ms latency to collector→server ==="
curl -s -X POST "$TOXI/proxies/server-grpc/toxics" \
  -H 'Content-Type: application/json' \
  -d '{"name":"latency-500","type":"latency","attributes":{"latency":500}}'
sleep 20

echo "=== Phase 3: Add 10% packet loss to collector→server ==="
curl -s -X DELETE "$TOXI/proxies/server-grpc/toxics/latency-500"
curl -s -X POST "$TOXI/proxies/server-grpc/toxics" \
  -H 'Content-Type: application/json' \
  -d '{"name":"loss-10","type":"timeout","attributes":{"timeout":2000}}'
sleep 20

echo "=== Phase 4: Full partition of server→db ==="
curl -s -X DELETE "$TOXI/proxies/server-grpc/toxics/loss-10"
curl -s -X POST "$TOXI/proxies/surrealdb/toxics" \
  -H 'Content-Type: application/json' \
  -d '{"name":"db-partition","type":"timeout","attributes":{"timeout":1}}'
sleep 10

echo "=== Phase 5: Remove partition, allow recovery ==="
curl -s -X DELETE "$TOXI/proxies/surrealdb/toxics/db-partition"
sleep 30

echo "=== Phase 6: Verify ==="
echo "Chaos test phases complete. Check collector and server logs for errors."
echo "Exit 0 = all phases ran without orchestrator failure."
