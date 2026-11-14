#!/usr/bin/env bash
set -euo pipefail

repo_url="${REPO_URL:-https://github.com/mihlefeld/Alg-Trainers.git}"
checkout="$HOME/.cache/alg-trainers-deploy"
web_root="${WEB_ROOT:-/web}"

for command in git tar rsync sudo; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "Missing required command: $command" >&2
        exit 1
    fi
done

for environment in main beta; do
    destination="$web_root/$environment/Alg-Trainers"
    if [[ -L "$destination" || -e "$destination/.git" || -L "$destination/.git" ]]; then
        echo "Refusing to overwrite a symlink or Git checkout: $destination" >&2
        exit 1
    fi
done

if [[ ! -d "$checkout/.git" ]]; then
    mkdir -p "$(dirname "$checkout")"
    git clone --no-checkout -- "$repo_url" "$checkout"
fi

git -C "$checkout" fetch --prune origin
for branch in master dev; do
    if ! git -C "$checkout" rev-parse --verify --quiet "refs/remotes/origin/$branch^{commit}" >/dev/null; then
        echo "Branch origin/$branch does not exist; nothing was deployed." >&2
        exit 1
    fi
done

stage="$(mktemp -d)"
trap 'rm -rf -- "$stage"' EXIT

for environment in main beta; do
    if [[ "$environment" == main ]]; then
        branch=master
    else
        branch=dev
    fi
    destination="$web_root/$environment/Alg-Trainers"
    mkdir -p "$stage/$environment" "$destination"
    git -C "$checkout" archive "refs/remotes/origin/$branch" | tar -xf - -C "$stage/$environment"
    rsync -rl --delete "$stage/$environment/" "$destination/"
    sudo restorecon -R "$destination"
    echo "Deployed origin/$branch to $destination"
done
