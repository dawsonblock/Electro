# Browser sandbox

This upgrade moves the preferred browser runtime out of the TEMM1E host process and into a dedicated containerized Chrome runtime exposed over the Chrome DevTools Protocol on `http://127.0.0.1:9223`.

## What changed

- `TEMM1E_BROWSER_ISOLATION_MODE=remote` is now the default recommended mode.
- The Rust browser tool connects to an already running remote browser instead of spawning a local Chrome process.
- The reference compose stack puts the browser on an **internal-only Docker network**.
- The browser reaches the public internet only through a dedicated proxy container.
- The proxy denies loopback, link-local, and RFC1918/private destinations.
- The Chrome DevTools port is published to `127.0.0.1` only.

## Start it

```bash
./scripts/run_browser_sandbox.sh
./scripts/smoke_browser_sandbox.sh
export TEMM1E_BROWSER_ISOLATION_MODE=remote
export TEMM1E_BROWSER_REMOTE_URL=http://127.0.0.1:9223
```

## Intentional failure mode

If the remote browser is not running, TEMM1E now fails closed instead of silently falling back to a local host browser. Set `TEMM1E_BROWSER_ISOLATION_MODE=local` only when you deliberately accept the weaker boundary.

## Local fallback

Local browser launch still exists for recovery and development, but it now requires an explicit `TEMM1E_BROWSER_PROXY_SERVER` when `TEMM1E_BROWSER_PROXY_REQUIRED=1`.
