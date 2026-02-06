#!/bin/bash

set -e

[ -z "$VERSION" ] && echo "You have to set a VERSION to run this script." && exit 1;

# bump version to $VERSION
sed -e "s/\"version\":\s\".*\"/\"version\": \"$VERSION\"/" -i package.json
npm install

# commit new version
git add package.json
git add package-lock.json
git commit -am "bumps version to $VERSION"

git push

git checkout release
git merge main -m "automatic merge to finish v$VERSION"

git push

git tag -a "v$VERSION"
git push --tags

git checkout main
