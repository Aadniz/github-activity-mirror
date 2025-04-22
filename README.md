# Github Activity Mirror

*Untested on Windows*

Application to mirror your off-platform Git activity from various platforms (Gitea, GitLab, Codeberg, etc.) to Github so it doesn't look like you're just sitting on your ass all day.

This application **DOES NOT** push the code to GitHub, rather it only changes the README.md of the repository created, changing it to the commit message it synced with. This way, it will only reflect your activity and not any code.

## Motivation

Many people have stepped away from the popular Github website, switching it out their main activity with alternatives such as Codeberg, GitLab, Forgejo. You won't know what GitHub does behind your back, *EVEN* when the repositories are set to private.

## Getting started

You start by setting up a `settings.toml` config file. This defines the username, api key, host address nessesary to read through all your activity.

Here is an example `setting.toml` up activity syncing from Gitea and Codeberg to GitHub:

``` toml
[[services]]
service_type = "gitea"
url = "https://gitea.yourhost.com"
username = "myusername"
token = "<your token here>"

[[services]]
service_type = "codeberg"
url = "https://codeberg.org"
username = "myusername"
token = "<your token here>"

[github]
username = "aadniz"
# Optional
email = "aadniz@example.com"
token = "ghp_<token>"
push_method = "ssh"
# Optional, default "1" (private repos)
redact_level = 4
```

Then compile the application with `cargo build --release`, run it with `./target/release/github-activity-mirror /path/to/settings.toml`.

## How it works

The application looks at the recent activities on set platforms, and notes down the title, description and date when these activities happened. It then compare the repositories with what's available on your GitHub profile, syncing up everything that doesn't exist, or has a marker at the bottom of the README.md. The marker it looks for looks like this:

``` markdown
<sub>This repo was mirrored using [github-activity-mirror](https://codeberg.org/Aadniz/github-activity-mirror), preserving the privacy while at the same time display your actual activity</sub>
```

This marker is included as a way to distinguish what is your own independent repositories, and what is created by this application.

Because altering commit dates are not supported by GitHub API, bash cli commands are used instead to overcome this. A local repo is cloned to your `$TEMP` folder for this purpose.

### Redact Level

Despite the code not being pushed to GitHub, there are still redact levels based on how little information to be shown in the repo name, description and commits.

The levels go from 0 (public) to 4 (private + hashed), with a default of 1 (private only).

| Redact Level | Private Repo | Visible Information                          | Example commit message                                                                                                                    |
|:-------------|:-------------|:---------------------------------------------|:------------------------------------------------------------------------------------------------------------------------------------------|
| 0            | No           | Full commit messages                         | Original commit message \n\n Mirrored from: https://v11.next.forgejo.org/username/my-repo/commit/1c7cf690f7a423d82f5f79b30fb28d0af24a47a1 |
| 1            | Yes          | Full commit messages                         | Original commit message \n\n Mirrored from: https://v11.next.forgejo.org/username/my-repo/commit/1c7cf690f7a423d82f5f79b30fb28d0af24a47a1 |
| 2            | Yes          | Full commit messages, excluding source links | Original commit message                                                                                                                   |
| ~~3~~        | Yes          | Encrypted commit messages                    | \<not implemented yet\>                                                                                                                     |
| 4            | Yes          | Sha1 hashed commit messages                  | 7a4743cbbfe504dcb1a5091f592a403d619135e6                                                                                                  |


With a redaction level of 4, the result might look something like this:

![Redact Level 4 repositories](/screenshots/Screenshot_20250418_160149.png)
![Redact Level 4 repository](/screenshots/Screenshot_20250418_163247.png)
![Redact Level 4 commits](/screenshots/Screenshot_20250418_162913.png)

If set to redact level 1 or higher, remember to enable Private contributions in the [Contribution settings](https://github.com/settings/profile#contributions-activity-heading).

## Contributing

I would very appreciate to see some more services than Gitea and Codeberg supported, implement a new service by creating a Rust file under `src/services/service_name.rs` having the `ServiceClient` trait.

For the moment, only Gitea and Codeberg are supported, but wish to support all the following git-like (or work-like) services in the future:

- [Bitbucket](https://bitbucket.org)
- [Cgit](https://git.zx2c4.com/cgit/about/)
- [Codebase](https://www.codebasehq.com)
- [Forgejo](https://forgejo.org)
- [Fossil](https://www.fossil-scm.org/index.html/doc/trunk/www/index.wiki)
- [Gerrit](https://www.gerritcodereview.com/)
- [Gitblit](https://www.gitblit.com/)
- [gitbucket](https://gitbucket.github.io/gitbucket-news/)
- [GitLab](https://about.gitlab.com)
- [Gitolite](https://gitolite.com/gitolite/index.html)
- [Gogs](https://gogs.io/)
- [Huly](https://huly.io)
- [Kallithea](https://kallithea-scm.org/)
- [Klaus](https://github.com/jonashaag/klaus)
- [Launchpad](https://launchpad.net)
- [Leantime](https://leantime.io)
- [Mindwendel](https://www.mindwendel.com/)
- [minimal-git-server](https://github.com/mcarbonne/minimal-git-server)
- [Octobox](https://octobox.io/)
- [OneDev](https://onedev.io/)
- [OpenProject](https://www.openproject.org)
- [Pagure](https://pagure.io/pagure)
- [Phorge](https://we.phorge.it/)
- [Plane](https://plane.so)
- [ProjeQtOr](https://www.projeqtor.org/)
- [Redmine](https://www.redmine.org/)
- [Review Board](https://www.reviewboard.org/)
- [rgit](https://github.com/w4/rgit)
- [RhodeCode](https://rhodecode.com/)
- [Rukovoditel](https://www.rukovoditel.net/)
- [SCM Manager](https://www.scm-manager.org/)
- [Smederee](https://smeder.ee)
- [Sourcehut](https://sourcehut.org/)
- [Taiga](https://www.taiga.io/)
- [Titra](https://titra.io/)
- [Trac](https://trac.edgewall.org/)
- [Traq](https://traq.io/)
- [Tuleap](https://www.tuleap.org/)
- [UVDesk](https://www.uvdesk.com/)
- [ZenTao](https://www.zentao.pm/)

Cross support would also be very interesting.
