# Venom
Linux package manager that doesn't clobber on upgrade

I'm pretty particular about how my system should behave. So I did the only reasonable thing and wrote my own package manager. Written purely in Ruby, this project has a couple core goals:
  - When I upgrade packages, don't overwrite the old ones
  - Never put the system in an unstable state. Or, if it must, do it as quickly as possible, and in such a way that the system can be recovered in the event of error or crash
  - Have good support for both source and binary packages, and provide an easy to use method for injecting patches into a source package on build
  - Keep packages untainted, with simple package metadata format

In order to achieve this, Venom installs each package into its own dir and symlinks it into the main filesystem. It draws inspiration from [GNU Stow](https://www.gnu.org/software/stow/) in that it tries to use as few symlinks as possible. However, Venom builds onto Stow in that it handles dependency resolution and better control over multiple versions of the same package being installed, as well as the ability to sync with a remote repository and install packages from that.

This project still has a loooonnnnnggg way to go, but it will get there eventually.

Currently, when fed in test data, it can perform the first phase of installation/uninstallation, which is to resolve the records to the database and figure out what needs to be added and removed.
