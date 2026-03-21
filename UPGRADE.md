Full Engineering Upgrade Blueprint

Project: temm1e hardened agent runtime

This blueprint assumes the current branch includes the hardening work already added:
	•	host shell locked down and runner-preferred
	•	workspace-only file access
	•	web_fetch private-target blocking
	•	clean-by-default browser profile
	•	custom tools disabled by default
	•	no first-user auto-admin by default
	•	browser remote-isolation mode added
	•	Docker/Compose reference stacks for shell and browser isolation

It does not assume the system has been compiled, tested, or proven end-to-end. In this environment that part was not verified.

⸻

1. Executive summary

temm1e is now in the right direction, but it is still between two states:
	•	past state: powerful local agent with soft guardrails
	•	target state: bounded operator-controlled agent platform with enforceable execution policy

The upgrade path should turn it into a system with one clear security model:

every dangerous action runs through a bounded runtime, under explicit capability policy, with traceable audit records, predictable failure modes, and clean operator override points.

The main engineering objective is to stop relying on prompt behavior and partial validation, and instead enforce safety through runtime boundaries, policy compilation, and reproducible infrastructure.

⸻

2. Target product definition

What this system should be

A self-hosted agent platform that can:
	•	read and modify a mounted workspace
	•	run bounded commands inside isolated runners
	•	browse the public web through an isolated browser service
	•	use tools only when policy allows them
	•	expose chat and channel interfaces without silent privilege escalation
	•	generate complete audit trails
	•	recover safely from worker corruption or timeout
	•	support extension without weakening the security model

What this system should not be

Not yet:
	•	unrestricted local autonomy on a personal workstation
	•	open multi-tenant SaaS
	•	implicit inheritance of the operator’s browser identity
	•	arbitrary host-code execution dressed up as “sandboxing”

⸻

3. Current state assessment

What is already strong
	•	substantial Rust workspace architecture
	•	large implementation surface
	•	real CI/test presence
	•	hardening direction is correct
	•	shell runner and browser remote mode now exist as concrete concepts
	•	defaults are moving from permissive to fail-closed

What still blocks a strong production claim
	•	compile/test/build verification is missing
	•	capability declarations are still not fully authoritative
	•	shell isolation still depends on external runtime availability
	•	browser sandboxing is still infrastructure-assisted, not kernel-guaranteed
	•	network policy is still mostly app-layer enforcement
	•	observability and release truth need tightening
	•	operational runbooks and failure recovery are not yet first-class

⸻

4. Engineering principles

Principle 1 — one security model

Every dangerous action must map to one of these categories:
	•	read
	•	write
	•	exec
	•	network
	•	browser automation
	•	secret access
	•	persistent extension
	•	channel administration

Each category must have:
	•	explicit policy
	•	runtime enforcement
	•	audit logging
	•	operator-visible denial reason

Principle 2 — no hidden privilege

A tool must not gain more power than its declaration says.
If a tool writes outside the workspace, opens sockets, persists executables, or inherits user state, that must be explicit and policy-gated.

Principle 3 — fail closed

If policy cannot be evaluated, if an isolated backend is unavailable, or if a runtime guarantee cannot be established, the action must fail.

Principle 4 — defaults matter

The system must boot into the safest working mode, not the most convenient legacy mode.

Principle 5 — docs must match code

Every feature claim must be backed by executable paths, tests, and current configuration.

⸻

5. Target architecture

5.1 Control plane

Responsible for:
	•	task ingestion
	•	tool selection
	•	policy evaluation
	•	runner dispatch
	•	event logging
	•	operator approvals
	•	recovery and retries

Suggested major modules:
	•	core/policy
	•	core/runtime
	•	core/audit
	•	core/approvals
	•	core/config
	•	core/recovery

5.2 Execution plane

Split into independent bounded runtimes:

Shell runner
	•	containerized
	•	workspace mount only
	•	no host home
	•	no inherited environment
	•	no ambient credentials
	•	optional network off by default

Browser runner
	•	remote browser only by default
	•	no local Chrome on agent host
	•	isolated network path
	•	proxy or namespace-bound egress
	•	clean profile only
	•	no live session inheritance unless explicitly granted

File access layer
	•	path resolution under policy
	•	mount-limited where possible
	•	deny absolute path escape
	•	deny home expansion by default

Extension runner
	•	no direct self-authored executable persistence by default
	•	reviewed/signed tool bundles only

5.3 Data plane

Artifacts and state should separate into:
	•	workspace data
	•	runtime temp state
	•	agent state
	•	audit state
	•	secrets/config
	•	browser ephemeral state
	•	quarantined extension bundles

These should not share one filesystem root.

⸻

6. Major workstreams

Workstream A — make capability policy authoritative

Objective

Turn declarations into enforceable runtime policy.

Required changes
	•	create a single policy schema for tool capabilities
	•	compile tool capabilities into runner constraints
	•	reject tools whose implementation exceeds declared capability class
	•	move from advisory checks to execution-bound policy

Deliverables
	•	policy_schema.rs
	•	policy evaluation engine
	•	tool capability registry
	•	policy-to-runner adapter
	•	denial reason taxonomy

Acceptance criteria
	•	every tool invocation produces a capability decision record
	•	tool cannot access undeclared file roots
	•	tool cannot access undeclared network targets
	•	tool cannot gain exec indirectly through hidden code paths

⸻

Workstream B — finish shell isolation

Objective

Make host shell irrelevant for supported operation.

Required changes
	•	keep host backend disabled by default
	•	formalize container runner contract
	•	support direct argv execution only
	•	add image pinning and version validation
	•	add runtime health checks before task execution
	•	add resource policy profiles: strict, normal, build-heavy

Deliverables
	•	stable shell runner interface
	•	pinned trusted shell-runner image
	•	runtime probe command
	•	runner startup validation
	•	task timeout/kill/reap logic
	•	structured stdout/stderr/exit reporting

Acceptance criteria
	•	no supported shell action requires host shell
	•	shell runner unavailable -> task denied
	•	shell tasks only see mounted workspace + tempfs
	•	no inherited host secret material
	•	reproducible smoke test passes

⸻

Workstream C — finish browser isolation

Objective

Make browser automation run outside the agent host trust boundary.

Required changes
	•	remote browser only by default
	•	require CDP endpoint health validation
	•	pin browser image version
	•	route all egress through isolated proxy or network namespace
	•	restrict destinations to public-web allowlist policy
	•	disable JS eval unless explicit policy permits it
	•	enforce profile ephemerality

Deliverables
	•	browser runtime manager
	•	remote browser health probe
	•	browser proxy configuration module
	•	destination validation shared library
	•	browser session lifecycle cleanup
	•	artifact download quarantine path

Acceptance criteria
	•	agent host never launches local Chrome in default mode
	•	browser cannot reach loopback/private/internal targets
	•	downloaded files land in quarantine, not workspace, unless approved
	•	session teardown removes browser state
	•	remote browser startup and smoke script pass

⸻

Workstream D — network policy enforcement

Objective

Move network control from app logic toward enforceable boundaries.

Required changes
	•	define per-tool egress classes
	•	centralize public/private target classification
	•	add DNS resolution guard
	•	add redirect validation everywhere
	•	integrate proxy allowlisting
	•	eventually enforce network policy at container/runtime level

Deliverables
	•	net_policy.rs
	•	CIDR/private-host classifier
	•	domain allowlist support
	•	proxy contract
	•	outbound audit records

Acceptance criteria
	•	no tool can silently reach localhost or RFC1918 space
	•	all redirects are revalidated
	•	final destination logged
	•	egress allow/deny visible in audit trail

⸻

Workstream E — remove unsafe extension paths

Objective

Replace persistent self-authored executable tools with controlled extension bundles.

Required changes
	•	keep self-create disabled
	•	replace with reviewed extension packaging
	•	require signed or operator-approved bundle import
	•	run extensions inside same bounded runtime model
	•	separate extension manifest from executable payload

Deliverables
	•	extension manifest format
	•	bundle import flow
	•	review/approval flow
	•	signature or checksum validation
	•	extension capability registration

Acceptance criteria
	•	agent cannot silently create durable future code paths
	•	every extension has manifest, checksum, and capability declaration
	•	extension execution is auditable and sandboxed

⸻

Workstream F — identity, channel, and admin hardening

Objective

Make chat/channel exposure safe by configuration, not assumption.

Required changes
	•	no bootstrap auto-admin
	•	require explicit admin identity provisioning
	•	add operator onboarding flow
	•	separate human roles: viewer, operator, admin
	•	add channel-specific allowlists
	•	add signed config validation for channel tokens and admin IDs

Deliverables
	•	role model
	•	onboarding CLI
	•	admin bootstrap docs
	•	config validator
	•	channel auth integration tests

Acceptance criteria
	•	empty allowlist never escalates the first inbound user
	•	admin assignment requires explicit config
	•	channels fail closed when misconfigured

⸻

Workstream G — observability and audit truth

Objective

Make every high-risk action reconstructable.

Required changes
	•	complete real OTLP export or remove false readiness claims
	•	emit structured events for:
	•	policy decision
	•	tool invocation
	•	runner start/stop
	•	network destination
	•	file writes
	•	approval request
	•	denial
	•	timeout
	•	recovery
	•	add trace correlation IDs across task lifecycle
	•	add local immutable audit log mode

Deliverables
	•	event schema
	•	trace IDs
	•	audit sink abstraction
	•	OTLP exporter
	•	local JSONL fallback
	•	run replay tool

Acceptance criteria
	•	every tool action has correlated event records
	•	failure diagnosis does not rely on console logs
	•	health status reflects actual exporter state

⸻

Workstream H — repo truth, build system, and release discipline

Objective

Make the repo buildable, testable, and honest.

Required changes
	•	run cargo check, cargo test, clippy, fmt
	•	fix compile drift introduced by upgrades
	•	pin image tags and dependency versions
	•	generate feature inventory from code
	•	generate test counts from CI
	•	split oversized src/main.rs
	•	define supported vs experimental features

Deliverables
	•	green CI
	•	modularized main
	•	generated feature manifest
	•	support matrix
	•	release checklist
	•	security notes

Acceptance criteria
	•	clean workspace build
	•	smoke tests for shell and browser paths
	•	docs match code paths
	•	support matrix current at release time

⸻

7. Proposed repo restructuring

A cleaner product shape:

temm1e/
  crates/
    temm1e-core/
      src/
        policy/
        runtime/
        approvals/
        audit/
        recovery/
        config/
    temm1e-tools/
      src/
        file/
        shell/
        browser/
        network/
        extensions/
        common/
    temm1e-observable/
    temm1e-channels/
    temm1e-agent/
  docker/
    shell-runner/
    browser-sandbox/
    browser-proxy/
  scripts/
    build_shell_runner.sh
    smoke_shell_runner.sh
    build_browser_sandbox.sh
    smoke_browser_sandbox.sh
    validate_config.sh
    run_local_stack.sh
  docs/
    architecture/
    security/
    operations/
    runbooks/
    support_matrix.md
    release_checklist.md
  tests/
    integration/
    security/
    smoke/

src/main.rs should become a thin entrypoint only.

⸻

8. Configuration contract

Default security posture

Recommended defaults:
	•	TEMM1E_SHELL_BACKEND=container
	•	TEMM1E_ENABLE_HOST_SHELL=0
	•	TEMM1E_BROWSER_ISOLATION_MODE=remote
	•	TEMM1E_INHERIT_BROWSER_SESSION=0
	•	TEMM1E_BROWSER_ALLOW_EVAL=0
	•	TEMM1E_PUBLIC_WEB_ALLOWLIST= empty means public-web only, not unrestricted internal access
	•	TEMM1E_ENABLE_SELF_CREATE_TOOL=0
	•	explicit admin/channel allowlists required

Config classes

Split config into:

Required runtime config

things without which the system should not boot

Optional feature config

things that enable extra capabilities

Unsafe override config

things that materially weaken safety and should:
	•	log loudly
	•	appear in startup summary
	•	require explicit acknowledgment in non-dev modes

Examples:
	•	host shell enabled
	•	browser local mode enabled
	•	browser session inheritance enabled
	•	self-create enabled

⸻

9. Test strategy

9.1 Unit tests

Cover:
	•	path normalization
	•	policy evaluation
	•	net classification
	•	redirect validation
	•	tool capability mapping
	•	config parsing
	•	audit event generation

9.2 Integration tests

Cover:
	•	shell task execution in container runner
	•	shell denial when runner absent
	•	browser connection to remote CDP
	•	browser denial on private/internal URLs
	•	file tool refusal outside workspace
	•	channel auth failure when allowlist missing
	•	extension import denial without approval

9.3 Security tests

Cover:
	•	traversal attempts
	•	redirect-to-local attacks
	•	DNS rebinding style checks
	•	command injection against shell interface
	•	extension persistence bypass attempts
	•	unsafe env inheritance checks

9.4 Smoke tests

One command should validate:
	•	config
	•	shell runner health
	•	browser remote health
	•	audit sink writeability
	•	channel config sanity

⸻

10. Deployment topology

Recommended local/lab deployment

Host

Runs:
	•	control plane
	•	audit sink
	•	config loader

Containers
	•	shell runner
	•	browser sandbox
	•	browser proxy
	•	optional local OTLP collector

Mounts
	•	workspace only where needed
	•	dedicated audit volume
	•	no host home mount
	•	no implicit Docker socket exposure

Network
	•	browser on isolated network
	•	proxy-mediated egress
	•	shell runner with no network by default
	•	deny local/internal target classes

⸻

11. Approval model

Not every tool should have the same friction.

Low-risk automatic
	•	read-only workspace inspection
	•	public-web fetch to allowed domains
	•	non-mutating browser navigation

Approval-required
	•	file write outside generated artifact area
	•	shell exec with write intent
	•	dependency installation
	•	browser downloads into workspace
	•	extension import/activation
	•	secret use
	•	channel admin changes

Always blocked by default
	•	host shell
	•	live browser session inheritance
	•	self-authored persistent tools
	•	undeclared external network targets
	•	home-directory writes

⸻

12. Audit model

Each task should generate:
	•	task ID
	•	parent conversation/request ID
	•	model/tool decision trace
	•	policy decision
	•	runner selected
	•	tool arguments after normalization
	•	file paths touched
	•	network destinations
	•	approval requests and outcomes
	•	start/end timestamps
	•	exit code / error type
	•	artifact hashes for generated files

This is what turns the build from “agent with logs” into an operable system.

⸻

13. Phase plan

Phase 0 — stabilization

Goal: make the current patch set buildable and testable.

Deliver:
	•	compile fixes
	•	CI green
	•	smoke scripts usable
	•	docs corrected

Gate:
	•	cargo check
	•	cargo test
	•	shell/browser smoke path pass

Phase 1 — authoritative policy

Goal: central capability engine.

Deliver:
	•	policy schema
	•	policy decision logging
	•	tool capability registry

Gate:
	•	tool actions denied correctly when undeclared

Phase 2 — shell productionization

Goal: shell runner becomes default supported execution path.

Deliver:
	•	pinned image
	•	health checks
	•	runner policy profiles
	•	no host dependency for supported flows

Gate:
	•	all supported exec tasks pass through container runner

Phase 3 — browser productionization

Goal: remote browser path becomes reliable and bounded.

Deliver:
	•	health checks
	•	proxy enforcement
	•	quarantine downloads
	•	remote-only default

Gate:
	•	no local browser required for supported flows

Phase 4 — channel/admin hardening

Goal: safe operator exposure.

Deliver:
	•	explicit role system
	•	onboarding
	•	channel auth tests

Gate:
	•	no silent admin bootstrap

Phase 5 — observability and replay

Goal: complete auditability.

Deliver:
	•	OTLP or honest local-only telemetry
	•	replay tool
	•	structured traces

Gate:
	•	every tool action reconstructable

Phase 6 — extension system redesign

Goal: controlled extensibility.

Deliver:
	•	reviewed/signed extension bundles
	•	extension policy model

Gate:
	•	no raw persistent self-authored code path

Phase 7 — release discipline

Goal: supportable product release.

Deliver:
	•	support matrix
	•	release checklist
	•	hardened docs
	•	semver and migration notes

Gate:
	•	release candidate reproducibly built and tested

⸻

14. Engineering backlog by file area

Highest-priority file groups

crates/temm1e-tools/src/shell*
	•	finalize runner abstraction
	•	remove fragile legacy paths
	•	improve structured exec model

crates/temm1e-tools/src/browser*
	•	unify remote browser path
	•	quarantine downloads
	•	remove local implicit fallbacks

crates/temm1e-agent/src/executor*
	•	centralize policy evaluation
	•	remove scattered special-case validation

crates/temm1e-channels/src/*
	•	role enforcement
	•	explicit admin bootstrap

crates/temm1e-core/src/orchestrator_impl.rs
	•	either complete it or narrow the claim
	•	do not ship placeholder orchestration as if it were real

crates/temm1e-observable/src/otel.rs
	•	complete exporter or relabel behavior clearly

src/main.rs
	•	split into maintainable modules
	•	minimize startup/control complexity

⸻

15. Honest readiness model

Current honest rating

After the hardening upgrades already applied:
	•	architecture: good
	•	safety direction: good
	•	runtime verification: incomplete
	•	production readiness: not yet
	•	lab/dev readiness: plausible after build/test pass

What must be true before “production-ready” is credible
	•	verified compile and test green
	•	shell runner is the supported exec path
	•	browser remote isolation passes smoke tests
	•	capability policy is authoritative
	•	audit trail complete
	•	docs match code
	•	no silent privilege escalations
	•	no known fallback paths that bypass the model

⸻

16. Final recommendation

This build is worth continuing.

Not because it is finished, but because it has enough real structure to justify hardening into a serious bounded agent runtime. The right move is not more surface area. The right move is to close the loop on:
	•	policy
	•	isolation
	•	audit
	•	build truth
	•	operational clarity

That will do more for the system than adding ten more tools.
