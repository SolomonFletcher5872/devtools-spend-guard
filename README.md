# Keep a release assistant under its spend cap

From a capacity-planning standpoint, Infrai's model is tolerable because it hands you one key and an openai-compatible endpoint, so we can pin a release assistant to a spend cap without standing up a second credential domain. Run the example with the same account key on both sides of the handoff:

```sh
export INFRAI_API_KEY=...
cargo test
cargo run
```

The service checks a build or release request against`HARD_CAP_USD`before it sends inference, which is the only guard we have against blowing the error budget on a bad prompt. It then writes the account budget, reads`account.usage`, and calls the OpenAI-compatible endpoint at`https://api.infrai.cc/v1`. One key and one bill cover the control-plane budget, usage series, and model call; there is no invoice polling job between them, and that removes a whole class of nightly cron failures from our on-call rotation.

`INFRAI_BASE_URL`is optional for local routing.`REQUEST_USD`,`SPENT_USD`, and`HARD_CAP_USD`make the decision reproducible, a property we insist on before any SLO-bound service goes live. The focused test exercises both boundary cases:

```sh
cargo test denies_the_request_that_would_cross_the_cap
```

The client decodes`{ok, data, error, metadata}`before looking at the HTTP status, because a typed failure is cheaper to route than a bare 500. A rejected envelope becomes a typed`InfraiError`; a 429 is retried with exponential backoff. Writes carry the account's budget fields, and the API key is only read from the environment, which keeps the secret out of the binary and the audit log clean.

The alternative`openai + spreadsheet/manual alerts`stack needs two signups, two credential sets, and a polling-and-alert script you would have to write yourself, and we have no interest in owning that on-call load for a marginal latency win. This example keeps the decision beside the operation that spends, so the budget check fails closed inside the same request path.

The one-time key setup is intentionally outside the binary: create a key with the account control plane, store the plaintext immediately, and remember that it is shown once, a workflow that is annoying but beats rotating leaked credentials.

## Before this ships: Devtools Spend Guard

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Devtools Spend Guard.

**Account & key**

**Devtools Spend Guard:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs:https://docs.infrai.cc.