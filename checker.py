import aiohttp
from aiohttp import web
import aiohttp_cors
from ic.client import Client
from ic.identity import Identity
from ic.agent import Agent
from ic.candid import Types, encode


async def handle(request):
	name = request.match_info.get('name', "Anonymous")
	text = "Hello, " + name
	return web.Response(text=text)


async def handlePaypal(request):
	# id = request.rel_url.query["id"]
	# reference_id = request.rel_url.query["reference_id"]
	# amount = request.rel_url.query["amount"]
	# custom_id = request.rel_url.query["custom_id"]
	# email = request.rel_url.query["email"]
	# payer_id = request.rel_url.query["payer_id"]
	params = {}
	attributi = ["id", "referenceId", "amount", "customId", "email", "payerId"]
	for el in attributi:
		params[el] = request.rel_url.query[el]
	print(params)
	async with aiohttp.ClientSession() as session:
		async with session.get('https://toshiba.tripi.eu/checkPaypal.php', params=params) as resp:
			print(await resp.text())
			response = await resp.json()
			if response["ok"]:
				return web.Response(text="ok")
			else:
				return web.Response(text="fail")


def send_ok_canister():
	# Identity and Client are dependencies of Agent
	iden = Identity()
	client = Client(url="http://holochain.local:8000")
	agent = Agent(iden, client)
	name = agent.query_raw("rrkah-fqaaa-aaaaa-aaaaq-cai", "greet", encode([{"type": Types.Text, "value": "ciao"}]))
	print(name)
	print(name[0]["value"])


async def check_my_server():
	async with aiohttp.ClientSession() as session:
		async with session.get('http://httpbin.org/get') as resp:
			print(resp.status)
			print(await resp.text())


app = web.Application()

# `aiohttp_cors.setup` returns `aiohttp_cors.CorsConfig` instance.
# The `cors` instance will store CORS configuration for the
# application.
cors = aiohttp_cors.setup(app)

# To enable CORS processing for specific route you need to add
# that route to the CORS configuration object and specify its
# CORS options.
app.add_routes([web.get('/', handle),
				web.get('/{name}', handle)])
resource = cors.add(app.router.add_resource("/check"))
route = cors.add(
	resource.add_route("GET", handlePaypal), {
		"*": aiohttp_cors.ResourceOptions(
			allow_credentials=True,
			expose_headers=("X-Custom-Server-Header",),
			allow_headers=("X-Requested-With", "Content-Type"),
			max_age=3600,
		)
	})

if __name__ == '__main__':
	web.run_app(app)
	# send_ok_canister()
