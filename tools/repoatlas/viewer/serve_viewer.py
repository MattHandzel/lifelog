#!/usr/bin/env python3
from __future__ import annotations

import argparse
import http.server
import socketserver
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description="Serve RepoAtlas viewer")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8123)
    parser.add_argument("--root", default=".", help="Repository root to serve")
    args = parser.parse_args()

    root = Path(args.root).resolve()

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *h_args, **h_kwargs):
            super().__init__(*h_args, directory=str(root), **h_kwargs)

    with socketserver.TCPServer((args.host, args.port), Handler) as httpd:
        print(f"RepoAtlas viewer: http://{args.host}:{args.port}/tools/repoatlas/viewer/index.html")
        httpd.serve_forever()


if __name__ == "__main__":
    main()
