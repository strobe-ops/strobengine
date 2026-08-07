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
