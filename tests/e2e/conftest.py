import shutil
import sys
from pathlib import Path

import pytest
from aiohttp.test_utils import TestServer

from .mock_server import create_app


@pytest.fixture(scope="session")
async def mock_server():
    app = create_app()
    server = TestServer(app)
    await server.start_server()
    yield f"http://127.0.0.1:{server.port}"
    await server.close()


def _find_cli_bin() -> str:
    path = shutil.which("strobengine")
    if path:
        return path

    bin_dir = "Scripts" if sys.platform == "win32" else "bin"
    name = "strobengine.exe" if sys.platform == "win32" else "strobengine"
    return str(Path(sys.prefix) / bin_dir / name)


@pytest.fixture(scope="session")
def cli_bin() -> str:
    return _find_cli_bin()
