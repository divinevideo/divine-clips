# Divine Clips

Divine Clips is a clipping marketplace for [Divine](https://divine.video), the decentralized short-form video app reviving Vine's 6-second format on Nostr. Creators fund campaigns to promote their loops; clippers repost that content to TikTok, Instagram Reels, YouTube Shorts, and X, then submit the links back. View counts are verified through platform APIs, and payouts settle automatically as Bitcoin Lightning zaps based on verified views. Campaigns, submissions, verifications, and payouts are all Nostr events, so clipper identity and reputation stay portable across the network.

## Structure

- `clipcrate/` — Rust backend for campaigns, submissions, and payouts.
- `clips-web/` — web frontend for creators and clippers.
- `clips-verifier/` — service that verifies external view counts.

Local orchestration lives at the repo root (`docker-compose.yml`). See [AGENTS.md](AGENTS.md) for build, test, and contribution guidelines.

---

Part of [Divine](https://divine.video) — your playground for human creativity · [Brand guidelines](https://github.com/divinevideo/brand-guidelines)
