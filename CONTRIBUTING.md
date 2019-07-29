# Contributing

## Synopsis

Until the first stable release of Sylan, it will not expect to receive pull
requests or issues about subjective language design issues. Design by commitee
usually yields poor results in the early stage of a project, especially
programming language design.

Once the first cut of the language is complete, it will open up to more
contributions.

Feel free, however, to leave issues for substantial problems in existing code
that has clearly already migrated beyond the initial work in progress stage.

## Branching Model

Until the first stable release, Sylan commits will just be against master.

After the first stable release, [GitHub
Flow](https://guides.github.com/introduction/flow/) will be followed (not to be
confused with GitFlow), which is a sort of middle ground between GitFlow and
Trunk-based Development: there are pull request branches, but there is only one
trunk branch. Releases are tracked with tags.

Pull requests merged within this model will use [squash
commits](https://github.blog/2016-04-01-squash-your-commits/) rather than merge
commits.

## Testing

Try to keep the test coverage up for new contributions. Tests are to be written
in the same file as that which they test, towards the bottom of the file.

## Safe Code

So far, Sylan has been able to remove all references to Rust's `unsafe` via the
attribute `#![forbid(unsafe_code)]`. It's unrealistic this will be able to be
kept on forever, but let's avoid adding unsafe code until it a absolutely
necessary for passable performance.

## Rust Versions

Assume that Sylan will run on the latest _stable_ Rust unless there is a good
reason to suggest otherwise.

## Formatting

Whatever `rustfmt` does is almost certainly right. In fact, the CI will fail
your pushed commits if they don't match `rustfmt`'s expectations, so integrate
it into your local editor or IDE.

## Continuous Integration

CircleCI provides [Sylan's build pipeline](https://circleci.com/gh/LouisJackman/sylan).
This should not trigger except for merges into master, after the approvers (i.e.
me) have verified that the PR is worth merging and it isn't doing anything nefarious
with the CI process.
