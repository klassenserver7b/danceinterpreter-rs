# Contributing

Thank you for considering contributing to the danceinterpreter! Take a look at [the roadmap] to get an idea of the
current state of the application.

Please open an issue for your contribution according to the feature / enhancement issue template.
Then you can start coding.

Besides directly writing code, there are many other different ways you can contribute. To name a few:

- Improving the documentation
- Submitting bug reports and use cases
- Sharing, discussing, researching and exploring new ideas or crates

## Git Workflow

### Fork the project

If you have GitHub CLI run `gh repo fork --clone klassenserver7b/danceinterpreter-rs && cd danceinterpreter-rs` and go
to step 3

1. [Fork the danceinterpreter-rs repository](https://github.com/klassenserver7b/danceinterpreter-rs/fork) on GitHub
2. Clone the fork: `git clone git@github.com:your_github_username/danceinterpreter-rs.git && cd danceinterpreter-rs`
3. Create a new branch for your code: `git checkout -b my-feature` Make sure to follow the branch [naming convention](#branch-naming) below.

### `pre-commit`

* Install [prek](https://prek.j178.dev/installation/) to automatically ensure that your commits comply with our code style for both Rust and JavaScript/TypeScript. This saves time reviewing, so I don't have to point out nitpicky style issues. Once you have prek installed on your computer, set it up in your local Git repository:

      cd /path/to/your/git/repo
      prek install
      prek install -t pre-push

  If you have problems with particular hooks, you can use the `SKIP` environment variable to disable hooks:

      SKIP=end-of-file-fixer git commit

  This can also be used to separate logic changes and autoformatting into two subsequent commits. Using the SKIP environment variable is preferable to using `git commit --no-verify` (which also disables the checks) because it won't prevent catching other, unrelated issues.

### Branch Naming

- Features: `feat/your-feature`
- Enhancement: `enh/your-enhancement`
- Bug-fix: `bug/your-bugfix`
- Documentation: `docs/your-doc-changes`

### Publishing to the repo

If your contribution is finished and tested working on your forked repo branch it is time to open a pr against this
main branch and wait for review.

[the roadmap]: https://github.com/users/klassenserver7b/projects/1
