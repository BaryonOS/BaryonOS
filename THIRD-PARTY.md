# Third-party components and licenses

Third-party code and fonts retain their own license terms. Applying MPL-2.0 to project-owned software does not change those terms. [LICENSING.md](LICENSING.md) and [MPL-2.0-FILES.txt](MPL-2.0-FILES.txt) define the project-owned software scope; that register grants no additional rights to third-party material.

## Included code and fonts

| Component | Terms and existing evidence |
|---|---|
| Patched zune-jpeg 0.5.16-rc3 | MIT, Apache-2.0, or Zlib; full upstream texts are included under `services/image-codec/vendor/zune-jpeg-0.5.16-rc3/`; changes and provenance are documented in [image-codec/THIRD-PARTY.md](services/image-codec/THIRD-PARTY.md) |
| Patched hkdf 0.13.0 and hmac 0.13.0 | MIT or Apache-2.0; full upstream texts are included under the two directories in `services/keyring/vendor/`; changes and provenance are documented in [vendor/README.md](services/keyring/vendor/README.md) |
| Open Sans atlas `ui/graphics/assets/open-sans-atlas.rle` and generated glyph metadata `ui/graphics/src/atlas.rs` | Apache-2.0; both remain outside the MPL-covered-file register. See [graphics/THIRD-PARTY.md](ui/graphics/THIRD-PARTY.md), original provenance/hashes in [font-provenance.json](ui/graphics/assets/font-provenance.json), and the full license text under `ui/graphics/licenses/` |
| Noto Sans Mono bitmap data | OFL-1.1 for the font data, MIT for the Rust crate code; full notices are included in [native/THIRD-PARTY.md](ui/native/THIRD-PARTY.md) and [compositor/THIRD-PARTY.md](sdk/examples/compositor/THIRD-PARTY.md), together with their `licenses/` directories |
| SplitMix64 algorithm | Public-domain statement by Sebastiano Vigna; source attribution is preserved in `experiments/quantum/qsim-core/src/sampler.rs` and [quantum/THIRD-PARTY.md](experiments/quantum/THIRD-PARTY.md) |
| `days_from_civil` calendar calculation | Public-domain statement by Howard Hinnant; source attribution is preserved in `drivers/rtc/src/lib.rs` |

## Registered dependencies

Component-specific registers describe dependencies and selected notice terms:

- [quantum](experiments/quantum/THIRD-PARTY.md)
- [heap-allocator](sdk/heap-allocator/THIRD-PARTY.md)
- [document-import](sdk/examples/document-import/THIRD-PARTY.md)
- [image-crop](sdk/examples/image-crop/THIRD-PARTY.md)
- [image-codec](services/image-codec/THIRD-PARTY.md)
- [net-stack](services/net-stack/THIRD-PARTY.md)
- [net-tls](services/net-tls/README.md)
- [r-efi](docs/third-party/README.md)

A dependency entry in `Cargo.toml` or `Cargo.lock` does not mean that the dependency source or every optional dependency is included in this repository or in every binary. Binary distribution requires the applicable notices for the components actually included.

The dependency inventory requires particular attention to **Apache-2.0 AND ISC** for `ring` and **(MIT OR Apache-2.0) AND Unicode-3.0** for `unicode-ident`. The additional font terms for Noto Sans Mono are not fully represented by the MIT label of the Rust crate.

## Bundled knowledge collection

The `.agents/skills/os-dev/` collection includes external OSDev Wiki and Little OS Book material alongside project-authored tools. Source attribution, revision links, and recorded terms are preserved in [BRONNEN-EN-LICENTIES.md](.agents/skills/os-dev/BRONNEN-EN-LICENTIES.md), the [OSDev license notices](docs/third-party/osdev-license-notices.md), and [source-index.json](.agents/skills/os-dev/references/source-index.json).

The book and adaptations retain CC BY-NC-SA 3.0 US; the authors separately offer the code examples as public domain. The OSDev Wiki describes its older collection as mixed-license and identifies CC0 for content added after its stated cutoff. Individual examples may have other terms, including the HenipaOS example W004744 labelled BSD-3-Clause-No-Nuclear-License. This register does not classify the complete collection as CC0 or MPL-2.0, and does not resolve missing historical author declarations.

## Firmware and distribution packages

This source repository is not a general license grant for third-party QEMU, OVMF, SeaBIOS, or host packages. Any assembled runner or release package must track the required firmware licenses and component notices according to [tools/windows-qemu/README.md](tools/windows-qemu/README.md).
