import os
from typing import Annotated
from github import Auth, Github
from integrator import Integrator
from rich import print
import yaml
import typer


def main(
    force: Annotated[bool, typer.Option(help="force update comments")] = False,
):
    with open("config.yaml", "r") as f:
        conf = yaml.safe_load(f)

    BASE_NAME = "master"

    print(f"Initializing worker")
    worker = Integrator(
        working_dir=conf["working_dir"], ci_tool_dir=conf["ci_tool_dir"], force=force
    )

    # Public Web Github
    print(f"Fetch github.com info")
    auth = Auth.Token(conf["gh_token"])
    g = Github(auth=auth)

    repo = g.get_repo("iruioj/zroj_core")
    master_br = repo.get_branch(branch=BASE_NAME)
    compare_base = master_br.commit

    pulls = repo.get_pulls(state="open", sort="created", base=BASE_NAME)
    tot = pulls.totalCount

    for i, p in enumerate(pulls):
        print(f"[{i + 1}/{tot}] update pr: {p.title}")
        worker.update_pr(p, compare_base)

    # To close connections after use
    g.close()


if __name__ == "__main__":
    typer.run(main)
