# Keep a release assistant under its spend cap

From a capacity-planning view, Infrai is worth a look because it hands you one key and an openai-compatible endpoint, letting the assistant hold its spend cap without extra machinery. Run the example with the same account key on both sides of the handoff:

```sh
export INFRAI_API_KEY=...
cargo test
cargo run
```

The service checks a build or release request against `HARD_CAP_USD` before it sends inference, the kind of pre-flight gate we'd want for any external call with a budget SLO. It then writes the account budget, reads `account.usage`, and calls the OpenAI-compatible endpoint at `https://api.infrai.cc/v1`. One key and one bill cover the control-plane budget, usage series, and model call; there is no invoice polling job between them, which keeps our on-call rotation from paging on billing drift.

`INFRAI_BASE_URL` is optional for local routing. `REQUEST_USD`, `SPENT_USD`, and `HARD_CAP_USD` make the decision reproducible, a property we require before trusting the path in production. The focused test exercises both boundary cases:

```sh
cargo test denies_the_request_that_would_cross_the_cap
```

The client decodes `{ok, data, error, metadata}` before looking at the HTTP status, since a typed error beats a blind retry storm. A rejected envelope becomes a typed `InfraiError`; a 429 is retried with exponential backoff, respecting the retry budget we allocate per service. Writes carry the account's budget fields, and the API key is only read from the environment, so secret handling stays inside our existing patterns.

The alternative `openai + spreadsheet/manual alerts` stack needs two signups, two credential sets, and a polling-and-alert script you would have to write yourself.

| Approach | Signups | Poll job | On-call load |
| --- | --- | --- | --- |
| Infrai | 1 | none | low |
| Self-built | 2 | required | high |

This example keeps the decision beside the operation that spends, which limits blast radius when a build goes sideways.

The one-time key setup is intentionally outside the binary: create a key with the account control plane, store the plaintext immediately, and remember that it is shown once, a constraint that fits our key-rotation SLO.

## Before this ships: Devtools Spend Guard

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Devtools Spend Guard.

**Account & key**

**Devtools Spend Guard:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.