# Security Policy

## Supported Versions

Until Sylan has an initial stable release, any security problems on HEAD can be
reported.

## What to Report

As this is a compiler that is merely transforming input to output and is
written in a memory-safe language, one would hope that vulnerabilities would be
thin on the ground.

As this is an early version of a codebase still in flux, security problems
should only be submitted for:

* Blatant misuse of Rust APIs that could violate user expectations like memory
  safety.
* Evidence of my access to GitHub being compromised and being used to push
  nefarious commits.
* References to versions of crates that have known vulnerabilities. (When third
  party crates start to be used.)

General langsec discussions around the language design itself are better
submitted as general issues.

## Reporting a Vulnerability

See [my personal contact page](https://volatilethunk.com/pages/contact.html).

If Sylan actually becomes more than an unheard of toy project, a more formal
channel for reporting vulnerabilities will be set up.

While I would appreciate [Coordinated
Disclosure](https://en.wikipedia.org/wiki/Responsible_disclosure) of ten days
via the private communication channels above, I firmly believe that developers
should not feel entitled to demand disclosure timelines from security
researchers.

I would therefore still prefer [Full
Disclosure](https://en.wikipedia.org/wiki/Full_disclosure_(computer_security))
posted as a public GitHub issue over not being told about a vulnerability at
all. As this is an opensource project, there are no bounty rewards or the like.
