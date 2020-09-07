# mod_jar_inspector
`mod_jar_inspector` looks at Fabric mods in the current folder and prints various metadata using the `fabric.mod.json` and Mixin configuration files in each JAR.

## Commands

### Mixin listing
`mod_jar_inspector mixin` lists all the mixins in Fabric mods in the current folder. The `--filter` argument can be used to filter the mixins that are shown.

### Jar in jar listing
`mod_jar_inspector jij` displays a tree of included mods in Fabric mods in the current folder. The `--reverse` argument reverses the order of the tree, so mods are shown with mods that include them, and the `--filter` argument can be used to filter the top-level list of mods.

## Install
### From releases
1. Download the latest `mod_jar_inspector` binary at https://github.com/comp500/mod_jar_inspector/releases/
2. Rename it to `mod_jar_inspector` (or any name you like) so it's easier to remember
3. Put it in a folder that is in your PATH (e.g. `/usr/local/bin` on Linux) or execute it from the folder you downloaded it to

### From source
1. Install Rust at https://www.rust-lang.org/tools/install
2. Run `cargo install --git https://github.com/comp500/mod_jar_inspector/`
