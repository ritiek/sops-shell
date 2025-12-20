# sops-shell

A tool to sync secrets in a sops encrypted file based on the output of pre-defined shell commands. It supports only
the sops file formats that allow for storing comments. YAML, ENV, and INI are supported. Binary and JSON aren't
supported.

## Motivation

I use [sops-nix](https://github.com/Mic92/sops-nix) to maintain my secrets in NixOS.

Over time, I've developed use-cases for storing keys in my `secrets.yaml` using sops, that are already present in
my Bitwarden wallet.

Maintaining the same secrets again through sops has resulted in duplication of this effort. This tool helps me ease
this effort by helping treat my Bitwarden wallet as the source of truth and lets me sync specific secrets from my
wallet to `secrets.yaml`.

## Example usage

Say we have a `secrets.yaml`, which when decrypted using `sops decrypt secrets.yaml` contains the following key-value
pairs:
```yaml
postgres_user: postgresness
postgres_pass: A_STRONK_PASS
github_token: some-secret
```

We want to link the `github_token` secret to an external source. This can be done so by adding a line comment starting
with `shell:` just before the secret:
```yaml
postgres_user: postgresness
postgres_pass: A_STRONK_PASS
# shell: rbw get b8f0e379-9f78-48d5-8f06-0f62b827c663 --field "GitHub Token"
github_token: some-secret
```

Now invoke the following dry-run command (which requires setting up this project):
```bash
$ sops-shell check secrets.yaml
Processing secrets.yaml...
  Found 1 secret(s) with commands

  github_token
    Command: rbw get b8f0e379-9f78-48d5-8f06-0f62b827c663 --field "GitHub Token"
    Status: IN SYNC

  All secrets in sync

============================================================
Summary:
  Files checked: 1
  Secrets checked: 1
  Secrets out of sync: 0
```

This will execute the shell commands defined in the .yaml and assert whether the stdout of the command matches with
the secret value defined in the line just below.

Now say, this GitHub token expired, and we re-generate a new token and store it in our Bitwarden wallet. Re-running:
```bash
$ sops-shell check secrets.yaml
Processing secrets.yaml...
  Found 1 secret(s) with commands

  github_token
    Command: rbw get b8f0e379-9f78-48d5-8f06-0f62b827c663 --field "GitHub Token"
    Status: OUT OF SYNC

  Would update 1 secrets (dry run)

============================================================
Summary:
  Files checked: 1
  Secrets checked: 1
  Secrets out of sync: 1
```
would notify us that the GiHub token secret has gone out-of-sync from the output the corresponding shell command.

To re-sync all such out-of-sync secrets defined in the file (non-dry-run mode), we can execute:
```bash
$ sops-shell sync secrets.yaml
Processing secrets.yaml
  Found 1 secret(s) with commands

  github_token
    Command: rbw get b8f0e379-9f78-48d5-8f06-0f62b827c663 --field "GitHub Token"
    Status: OUT OF SYNC

  Updating 1 secrets...
    Updated github_token

  Updated secrets.yaml

============================================================
Summary:
  Files processed: 1
  Secrets checked: 1
  Secrets updated: 1
```

If we check the .yaml file now using `sops decrypt secrets.yaml`, we'll see it updated the GitHub token secret from
`some-secret` to `a-new-secret` in `secrets.yaml` as per the output of the corresponding shell command:
```yaml
postgres_user: postgresness
postgres_pass: A_STRONK_PASS
# shell: rbw get b8f0e379-9f78-48d5-8f06-0f62b827c663 --field "GitHub Token"
github_token: a-new-secret
```

It's also possible to specify complex shell commands with sops-shell such as:
```yaml
# shell: rbw get f8c370b3-8fcb-4181-bd21-ffb13de3b5af --raw | jq -r ".data.uris.[0].uri"
```

## Compiling and running

You need to have `sops` available and in PATH.

If you have Nix with flakes enabled, you can use `nix run` to call the tool:
```bash
$ nix run github:ritiek/sops-shell check /path/to/secrets.yaml
```

Otherwise you can use cargo:
```bash
$ git clone https://github.com/ritiek/sops-shell
$ cd sops-shell
$ cargo run -- check /path/to/secrets.yaml
```

To see the list of supported options, pass `--help`.

Any `shell:` commands specified in respective sops files need to be available in PATH for sops-shell to be able
to find them. These commented out lines starting with `shell:` also get encrypted by sops, so these sops-encrypted
files can be pushed to public repositories without other people being able to figure out what shell command any
particular secret is linked to.

## Inspiration

sops-shell was inspired from the following projects but neither of them quite fit my current needs on their own.

### [sopswarden](https://github.com/pfassina/sopswarden)

sopswarden looks to be implemented as a part of NixOS config and requires the `--impure` flag when rebuilding the
config (at least so with the version on the `main` branch). It doesn't look to be backwards compatible with
sops-nix and requires re-defining all the secrets present in the NixOS config in the first-go (specifically what
made it difficult for me to try out). It doesn't seem possible to specify arbitrary shell commands and works only
with `rbw`. Other people can see where the secret exactly lives in your Bitwarden wallet which could be a privacy
concern for some people.

### [sopsidy](https://github.com/timewave-computer/sopsidy)

sopsidy looks to be implemented as a part of NixOS config as well and doesn't seem backwards compatible. It
specifies the following:

> Sopsidy is designed to completely take over the sops files in the repository, so if sops-nix is already being
> used be sure to have backups of all existing sops files before setting up sopsidy. Also as mentioned before,
> move `.sops.yaml` out of the repository as it interferes with sopsidy's native management of age keys for each
> sops file - based on the `sops.hostPubKey` option provided by the sopsidy nixos module.

Which sounded a little too intrusive for my taste.

It also didn't seem simple to use PGP keys for decrypting the secrets. Where as sops-shell offloads these tasks
to sops itself, also making it possible to specify environment variables like `SOPS_AGE_KEY_CMD` for sops to pick
up. While sopsidy can take in any arbitrary command unlike sopswarden (which supports only rbw), other people can
see the arbitrary shell command used to extract a particular secret if you are to push sopsidy based NixOS config
files to a public repository, which can be a little concerning.

## License

MIT
