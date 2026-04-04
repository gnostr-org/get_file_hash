# Managing Repository and Relay Sets

Sets are a convenience feature for contributing to projects that do not have a
`nostr-address` file. Instead of manually specifying the project's repositories
and relays for every command, you can define them once as a named "set". You can
then reference this set by its name in commands. This allows you to use the set
as a shortcut for a list of relays (`--relays <set_name>`) or as the project's
address in commands like `issue` and `patch`.

Sets are defined in your configuration file. To use a specific configuration
file, pass its path using the `--config` option.
