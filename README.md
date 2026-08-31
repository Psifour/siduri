# siduri

Password manager built on the [gilgamesh](https://github.com/psifour/gilgamesh) vault: logins, TOTP,
and a generator, stored as `login.v1` records in the standard gilgamesh vault
file. Uses a running `gilgamesh agent` when one is up (no password prompt);
falls back to unlocking the vault in-process.

```sh
siduri add github --url https://github.com --username you --generate
siduri show github            # metadata; pipe it for the raw password
siduri show github | wl-copy  # or: siduri clip github (30s timed clear)
siduri totp github            # RFC 6238 code from the stored secret
siduri search work
siduri edit github --generate
siduri rm github
siduri gen --words 7          # ~77 bits, BIP39 wordlist
```

Passwords are random and stored — never derived per-site — so a breached
site rotates without fighting the identity's append-only labels. Record ids
in the vault are opaque hex: the plaintext id list does not leak your
account list. See [design-doc.md](design-doc.md).

A browser extension (native-messaging host talking to the gilgamesh agent)
is the planned phase 2.

## Development

```sh
./bin/activate-hermit
just ci
```

## License

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE)
or the [MIT license](LICENSE-MIT), at your option.

This software handles passwords and other secrets. It is provided **as is**,
without warranty of any kind, and the authors accept **no responsibility or
liability** for any loss — of secrets, accounts, data, or access — arising
from its use. Keep your gilgamesh entropy backup and verify recovery before
you depend on it.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
