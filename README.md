# mod_jar_inspector
`mod_jar_inspector` looks at Fabric mods in the current folder and prints various metadata using the `fabric.mod.json` and Mixin configuration files in each JAR.

## Commands

### Mixin listing
`mod_jar_inspector mixin` lists all the mixins in Fabric mods in the current folder. The `--filter` argument can be used to filter the mixins that are shown.

Example output:

```
$ mod_jar_inspector mixin --filter crash
Reading mods in the current folder...
fabric-crash-report-info-v1 (fabric-crash-report-info-v1-0.1.2+b7f9825d4e.jar)
    MixinCrashReport
patchouli (Patchouli-1.16-40-FABRIC.jar)
Client:
    client.MixinCrashReport
```

### Jar in jar listing
`mod_jar_inspector jij` displays a tree of included mods in Fabric mods in the current folder. The `--reverse` argument reverses the order of the tree, so mods are shown with mods that include them, and the `--filter` argument can be used to filter the top-level list of mods.

Example output:

```
$ mod_jar_inspector jij --filter astromine
Reading mods in the current folder...
astromine (astromine-1.9.2+fabric-1.16.2.jar)
    astromine-core (astromine-core-1.9.2+fabric-1.16.2.jar)
        patchouli (Patchouli-1.16-40-FABRIC.jar)
            fiber (fiber-0.23.0-1.jar)
        autoconfig1u (autoconfig1u-3.2.2.jar)
        blade (blade-fbdf8790.jar)
            blue_endless_jankson (jankson-1.2.0.jar)
        cardinal-components-base (cardinal-components-base-2.5.4.jar)
        cardinal-components-block (cardinal-components-block-2.5.4.jar)
        cardinal-components-chunk (cardinal-components-chunk-2.5.4.jar)
...
```

### Access widener listing
`mod_jar_inspector aw` lists all the access wideners in Fabric mods in the current folder. The `--filter` argument can be used to filter the access wideners that are shown.

Example output:

```
$ mod_jar_inspector aw --filter class_1011
Reading mods in the current folder...
slight-gui-modifications (slight-gui-modifications-1.3.0.jar)
    accessWidener       v1      intermediary
    accessible  class   net/minecraft/class_500
    accessible  class   net/minecraft/class_526
    accessible  class   net/minecraft/class_437
    accessible  class   net/minecraft/class_1011
    accessible  class   net/minecraft/class_473$class_5234
    accessible  class   net/minecraft/class_310
    accessible  class   net/minecraft/class_332
    accessible  class   net/minecraft/class_442
    accessible  class   net/minecraft/class_473$class_5233
    accessible  method  net/minecraft/class_473$class_5234      <init>  (II)V
    accessible  method  net/minecraft/class_500 method_20378    (Z)V
    accessible  method  net/minecraft/class_500 method_20377    (Z)V
...
```

## Install
### From releases
1. Download the latest `mod_jar_inspector` binary at https://github.com/comp500/mod_jar_inspector/releases/
2. Rename it to `mod_jar_inspector` (or any name you like) so it's easier to remember
3. Put it in a folder that is in your PATH (e.g. `/usr/local/bin` on Linux) or execute it from the folder you downloaded it to

### From source
1. Install Rust at https://www.rust-lang.org/tools/install
2. Run `cargo install --git https://github.com/comp500/mod_jar_inspector/`
