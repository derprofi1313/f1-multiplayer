// A code directory for six-digit server codes over the internet (packages/core/src/multiplayer/servercode.rs).
// A Cloudflare Worker with one KV namespace bound as CODES; see README.md.
//
//   PUT /v1/codes/<code>  {"host": "<host code>", "key": "<owner key>"}  -> 204, or 409 when another key owns it
//   GET /v1/codes/<code>                                                -> 200 {"host": "<host code>"}, or 404
//
// An entry lasts a week after its last PUT; the host's launcher puts it again every time it starts. Only a
// hash of the owner key is stored.

const TTL_SECONDS = 7 * 24 * 3600;
const CODE = /^\/v1\/codes\/(\d{6})$/;
const HOST_CODE = /^[A-Za-z0-9.\-\[\]:]{1,300}#[A-Za-z0-9_-]{43}$/;

async function sha256(text) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

function json(status, body) {
  return new Response(body === undefined ? null : JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

export default {
  async fetch(request, env) {
    const match = new URL(request.url).pathname.match(CODE);
    if (!match) return json(404, { error: "not found" });
    const code = match[1];
    const stored = await env.CODES.get(code, "json");
    if (request.method === "GET") {
      return stored ? json(200, { host: stored.host }) : json(404, { error: "no server has that code" });
    }
    if (request.method !== "PUT") return json(405, { error: "GET or PUT" });
    let body;
    try {
      body = await request.json();
    } catch {
      return json(400, { error: "the body is not JSON" });
    }
    if (typeof body.host !== "string" || !HOST_CODE.test(body.host) || typeof body.key !== "string" || body.key.length < 16) {
      return json(400, { error: "needs host (a host code) and key" });
    }
    const owner = await sha256(body.key);
    if (stored && stored.owner !== owner) return json(409, { error: "another server owns this code" });
    await env.CODES.put(code, JSON.stringify({ host: body.host, owner }), { expirationTtl: TTL_SECONDS });
    return json(204);
  },
};
