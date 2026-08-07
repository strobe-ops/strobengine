import asyncio

from aiohttp import web


async def handle_status(request: web.Request) -> web.Response:
    code = int(request.match_info["code"])
    return web.Response(status=code, text=f"status {code}")


async def handle_delay(request: web.Request) -> web.Response:
    seconds = float(request.match_info["seconds"])
    await asyncio.sleep(seconds)
    return web.Response(status=200, text=f"delayed {seconds}s")


async def handle_echo(request: web.Request) -> web.Response:
    body = None
    content_type = request.content_type or ""

    if "json" in content_type:
        try:
            body = await request.json()
        except (ValueError, Exception):
            body = None
    else:
        raw = await request.read()
        if raw:
            body = raw.decode("utf-8", errors="replace")

    return web.json_response(
        {
            "method": request.method,
            "headers": dict(request.headers),
            "body": body,
        }
    )


async def handle_flaky(request: web.Request) -> web.Response:
    current = request.app["flaky_state"]
    request.app["flaky_state"] = not current
    if current:
        return web.Response(status=200, text="ok")
    return web.Response(status=500, text="internal error")


def create_app() -> web.Application:
    app = web.Application()
    app["flaky_state"] = False
    app.router.add_route("*", "/echo", handle_echo)
    app.router.add_get("/status/{code}", handle_status)
    app.router.add_get("/delay/{seconds}", handle_delay)
    app.router.add_get("/flaky", handle_flaky)
    return app
