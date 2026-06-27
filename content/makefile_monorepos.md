---
date: 06/27/2026
title: Helpful Makefile tips for Monorepos 
blurb: A description of .PHONY and Sentinel files.
---

Recently for managing some of my projects, I opted to use a Monorepo setup because - hey why not? For smaller projects I definitely prefer this, especially when the backend serves some type of static content to the user, and the API isn't super bloated. It's also a nice warm fuzzy feeling having everything operate in one repository.

However I ran into issues on having to manage dependencies and building in production. I didn't have an easy flow to do those things. I'd work on individual components separately, take a break from the project for 3 months, then forget how everything tied together. Of course that also due to a lack of documentation - but the issues stem beyond that.

I needed something to manage the monorepo - independent of what the components are. Sure there are tools like `nx`, but they're a bit too bloated for simple projects - and managing more dependencies to manage components just doesn't feel right. What about using the classic Makefile?

# Makefile

(GNU) Make is a utility to compile binaries from various target files. It's most commonly used in the C and C++ world, but comes in almost every Linux distribution imaginable. It also has a great way to manage how to build applications.

Even though it's (generally) used to compile binaries - there are a few tricks to use to be able to manage different components within a monorepo - making it sufficient to run scripts, build components, and install packages within a MonoRepo. Essentially the main parts are
- The filesystem is an important part of Make - adopt it.
- Use Sentinel files for targets that don't yield exactly one file - such as a `vite` distribution

## Makefile Rules
The general outline of a Make rule is
```
<output-file>: <input file1> <...> <input file n>
  <script to create output-file from input files>
  ...
  <script to create output-file from input files>
```

When you create a rule - Make essentially looks to see if the output file's timestamp is older than any of the input files listed. If it is - that must mean that the input files have been updated - making the output file obsolete. This rule makes a strong dependency on the output and input.

## Tip \#1: Make use of .PHONY
Have you caught something? What if I just want to run commands without actually building something? What if in my monorepo I want to run all tests in all my components?

For example if I have
```
test-client:
  cd client && yarn run test
test-server:
  cd server && pytest

test: test-server test-client


deploy: test
  deploy script...
```
Then I can run `make test` that will run tests for my client and server. Right?

Well no, because Make is heavily integrated with the file system. If I have a folder called test-client and test-server - then running `make test` won't yield anything because test exists. This problem seems obvious - but in large or even medium sized Makefiles ~200 lines plus, this could be an issue.

You should be using `.PHONY` to specify that the output file is never generated. THUS it causes the scripts to always run.

`.PHONY: test test-client test-server`

## Tip \#2 Sentinel Files

Sometimes a rule can generate many output files, like for example `npm install` creating thousands of files - which is note a reliable way to compare to an output file. If your monorepo gets large this can take a long time to even determine if an output should be built.

A handy trick is to use a Sentinel file to determine if a large amount of inputs should be reprocessed again.

```
tmp/.sentinel: $(shell find src -type f)
  mkdir -p $(@D)
  yarn build
  touch $@
```
This rule basically checks if any file in source is newer than the `tmp.sentinel` - then rebuild the application. After building has finished, update the .sentinel timestamp. 

This allows us to be flexible on when certain `.PHONY` commands are run - resulting in checkpoints of what has occurred in each component.

