import http.server
import socketserver
import threading
from collections.abc import Generator

import pytest


class QuietHTTPRequestHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:
        pass

    def do_GET(self) -> None:
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"status": "ok"}')


@pytest.fixture(scope="session")
def local_server() -> Generator[str]:
    server = socketserver.TCPServer(("127.0.0.1", 0), QuietHTTPRequestHandler)
    ip, port = server.server_address
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://{ip}:{port}"
    server.shutdown()
    server.server_close()
