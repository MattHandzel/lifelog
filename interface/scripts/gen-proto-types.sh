#!/usr/bin/env bash
# Generate TypeScript type-only interfaces from proto files.
# Uses the protoc bundled in grpc-tools + ts-proto plugin.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INTERFACE_DIR="$(dirname "$SCRIPT_DIR")"
PROTO_DIR="$(dirname "$INTERFACE_DIR")/proto"
OUT_DIR="$INTERFACE_DIR/src/generated"

PROTOC="$INTERFACE_DIR/node_modules/grpc-tools/bin/protoc"
TS_PROTO_PLUGIN="$INTERFACE_DIR/node_modules/.bin/protoc-gen-ts_proto"

# Include google well-known types from grpc-tools
GOOGLE_PROTOS="$INTERFACE_DIR/node_modules/grpc-tools/bin/google"

mkdir -p "$OUT_DIR"

"$PROTOC" \
  --plugin="protoc-gen-ts_proto=$TS_PROTO_PLUGIN" \
  --ts_proto_out="$OUT_DIR" \
  --ts_proto_opt=onlyTypes=true \
  --ts_proto_opt=esModuleInterop=true \
  --ts_proto_opt=useOptionals=messages \
  --ts_proto_opt=exportCommonSymbols=false \
  --ts_proto_opt=snakeToCamel=true \
  --ts_proto_opt=outputServices=false \
  -I "$PROTO_DIR" \
  -I "$(dirname "$GOOGLE_PROTOS")" \
  "$PROTO_DIR/lifelog_types.proto" \
  "$PROTO_DIR/lifelog.proto"

echo "Generated TypeScript types in $OUT_DIR"
