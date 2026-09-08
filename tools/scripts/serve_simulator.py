#!/usr/bin/env python3
"""
Simple HTTP server to serve the KlipperScreen simulator for Playwright testing
"""
import http.server
import socketserver
import os
import sys

PORT = 8085
DIRECTORY = "/home/jrad/.gemini/antigravity-ide/brain/541af069-311c-4f5b-a976-162c43a84133"

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

if __name__ == "__main__":
    os.chdir(DIRECTORY)
    # Enable address reuse
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("127.0.0.1", PORT), Handler) as httpd:
        print(f"Serving KlipperScreen simulator at http://127.0.0.1:{PORT}/klipperscreen_simulator.html")
        sys.stdout.flush()
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            pass
