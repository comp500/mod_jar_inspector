use rayon::prelude::*;
use clap::{Clap, crate_version, AppSettings};
use anyhow::Result;
use std::{io::{Seek, Read, Cursor, BufReader}, collections::{BTreeMap, BTreeSet}, fs::File, ffi::OsStr, path::PathBuf};
use serde::Deserialize;
use zip::{ZipArchive};
use enum_map::{enum_map, Enum, EnumMap};

#[derive(Debug, Clone, Deserialize, Enum, Copy)]
#[serde(rename_all = "camelCase")]
enum Environment {
	#[serde(rename = "*")]
	Both,
	Client,
	Server
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Both
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JarInJarListEntry {
	file: String
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
enum MixinConfigListEntry {
	Name(String),
	WithEnvironment { config: String, environment: Option<Environment> }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FabricModJson {
	id: String,
	version: String,
	name: Option<String>,
	#[serde(default)]
	environment: Environment,
	#[serde(default)]
	jars: Vec<JarInJarListEntry>,
	#[serde(default)]
	mixins: Vec<MixinConfigListEntry>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MixinConfigJson {
	plugin: Option<String>,
	#[serde(default)]
	mixins: Vec<String>,
	#[serde(default)]
	client: Vec<String>,
	#[serde(default)]
	server: Vec<String>
}

#[derive(Debug)]
enum TraversedJar {
	NonMod,
	FabricJar{
		mod_name: Option<String>,
		mod_id: String,
		mod_version: String,
		environment: Environment,
		mixins: EnumMap<Environment, Vec<String>>,
		mixin_config_plugins: Vec<String>,
		contained_jars: BTreeMap<String, TraversedJar>
	}
}

fn read_mod_json<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<FabricModJson> {
	Ok(serde_json::from_reader(zip.by_name("fabric.mod.json")?)?)
}

fn read_mixin_config<R: Read + Seek>(zip: &mut ZipArchive<R>, name: &str) -> Result<MixinConfigJson> {
	Ok(serde_json::from_reader(zip.by_name(name)?)?)
}

fn traverse<R: Read + Seek>(source: R) -> Result<TraversedJar> {
	let mut zip = zip::ZipArchive::new(source)?;

	if let Ok(fabric_mod_json) = read_mod_json(&mut zip) {
		let mut contained_jars = BTreeMap::new();
		for jar_entry in fabric_mod_json.jars {
			let mut jar_file = BufReader::new(zip.by_name(jar_entry.file.as_str())?);
			let mut file_contents = vec![];
			jar_file.read_to_end(&mut file_contents)?;

			contained_jars.insert(jar_entry.file.split("/").last().map(|s| s.to_owned()).unwrap_or(jar_entry.file), 
			traverse(Cursor::new(file_contents))?);
		}

		let mut mixins: EnumMap<Environment, Vec<String>> = enum_map! { _ => vec![] };
		let mut mixin_config_plugins = vec![];
		for mixin_entry in fabric_mod_json.mixins {
			if let (env_forced, Ok(mixin_config_file)) = match mixin_entry {
				MixinConfigListEntry::Name(name) => (None, read_mixin_config(&mut zip, name.as_str())),
				MixinConfigListEntry::WithEnvironment {config, environment: Some(Environment::Both)} => (None, read_mixin_config(&mut zip, config.as_str())),
				MixinConfigListEntry::WithEnvironment {config, environment: None} => (None, read_mixin_config(&mut zip, config.as_str())),
				MixinConfigListEntry::WithEnvironment {config, environment} => (environment, read_mixin_config(&mut zip, config.as_str())),
			} {
				for mixin in mixin_config_file.mixins {
					mixins[env_forced.unwrap_or(Environment::Both)].push(mixin);
				}
				for mixin in mixin_config_file.client {
					mixins[env_forced.unwrap_or(Environment::Client)].push(mixin);
				}
				for mixin in mixin_config_file.server {
					mixins[env_forced.unwrap_or(Environment::Server)].push(mixin);
				}
				if let Some(plugin) = mixin_config_file.plugin {
					mixin_config_plugins.push(plugin);
				}
			}
		}

		return Ok(TraversedJar::FabricJar{
		    mod_name: fabric_mod_json.name,
		    mod_id: fabric_mod_json.id,
		    mod_version: fabric_mod_json.version,
			environment: fabric_mod_json.environment,
			mixins,
			mixin_config_plugins,
		    contained_jars,
		});
	}

	Ok(TraversedJar::NonMod)
}

#[derive(Clap, Debug)]
#[clap(version = crate_version!(), setting(AppSettings::UnifiedHelpMessage), setting(AppSettings::GlobalVersion))]
struct Opts {
	#[clap(subcommand)]
	subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
	Mixin(MixinCommand),
	#[clap(alias = "jij")]
	JarInJar(JarInJarCommand),
	Raw(RawCommand)
}

/// Lists mixins in mods in the current folder
#[derive(Clap, Debug)]
#[clap(setting(AppSettings::UnifiedHelpMessage))]
struct MixinCommand {
	/// Filter the list of mixins using this search string
	#[clap(long)]
	filter: Option<String>
}

/// Displays the Jar in Jar tree for the current folder
#[derive(Clap, Debug)]
#[clap(setting(AppSettings::UnifiedHelpMessage))]
struct JarInJarCommand {
	/// Display the reverse tree, only showing jars which are contained by other jars
	#[clap(short, long)]
	reverse: bool,
	/// Filter the list of top-level mods (by mod id) using this search string
	#[clap(long)]
	filter: Option<String>
}

/// Prints raw traversal output
#[derive(Clap, Debug)]
#[clap(setting(AppSettings::UnifiedHelpMessage))]
struct RawCommand {}

fn main() -> Result<()> {
	let opts: Opts = Opts::parse();

	println!("Reading mods in the current folder...");

	let jar_list: Vec<_> = std::fs::read_dir(".")?.into_iter()
		.filter_map(Result::ok)
		.filter(|f| f.path().is_file())
		.collect();
	
	let processed_jars: Vec<_> = jar_list.par_iter()
		.filter(|entry| entry.path().extension().and_then(OsStr::to_str) == Some("jar"))
		.map::<_, Result<(PathBuf, TraversedJar)>>(|entry| {
			let file = BufReader::new(File::open(entry.path())?);
			Ok((entry.path(), traverse(file)?))
		})
		.map(|entry| {
			entry.unwrap()
		})
		.collect();

	match opts.subcmd {
		SubCommand::Mixin(mixin_cmd) => {
			let mut collated_jars: BTreeMap<String, (BTreeSet<String>, EnumMap<Environment, BTreeSet<String>>)> = BTreeMap::new();

			fn matches(dest: &str) -> impl FnMut(&String) -> bool + '_ {
				move |name: &String| name.to_lowercase().contains(dest)
			}

			fn recursively_collate(dest: &mut BTreeMap<String, (BTreeSet<String>, EnumMap<Environment, BTreeSet<String>>)>, jar: TraversedJar, file_name: &str, filter: Option<String>) {
				if let TraversedJar::FabricJar{ mod_id, contained_jars, mixins, .. } = jar {
					let collate_dest = dest.entry(mod_id).or_insert((BTreeSet::new(), enum_map! { _ => BTreeSet::new() }));

					collate_dest.0.insert(file_name.to_owned());
					if let Some(ref filter) = filter {
						collate_dest.1[Environment::Both].extend((&mixins[Environment::Both]).into_iter().cloned().filter(matches(filter)));
						collate_dest.1[Environment::Client].extend((&mixins[Environment::Client]).into_iter().cloned().filter(matches(filter)));
						collate_dest.1[Environment::Server].extend((&mixins[Environment::Server]).into_iter().cloned().filter(matches(filter)));
					} else {
						collate_dest.1[Environment::Both].extend((&mixins[Environment::Both]).into_iter().cloned());
						collate_dest.1[Environment::Client].extend((&mixins[Environment::Client]).into_iter().cloned());
						collate_dest.1[Environment::Server].extend((&mixins[Environment::Server]).into_iter().cloned());
					}

					for contained_jar in contained_jars {
						recursively_collate(dest, contained_jar.1, contained_jar.0.as_str(), (&filter).to_owned());
					}
				}
			}
			
			let filter = (&mixin_cmd.filter).as_ref();
			for jar in processed_jars {
				recursively_collate(&mut collated_jars, jar.1, jar.0.file_name().map(|f| f.to_str().unwrap()).unwrap_or(jar.0.to_str().unwrap()),
				filter.map(|filter| filter.as_str().to_lowercase()));
			}

			let mut matched_jars = false;
			for jar in &collated_jars {
				// If there is a filter, hide jars that don't match the filter
				if mixin_cmd.filter.is_some() && jar.1.1.values().all(|v| v.len() == 0) {
					continue;
				}

				matched_jars = true;
				println!("{} ({})", jar.0, (&jar.1.0).into_iter().cloned().collect::<Vec<String>>().join(", "));
				for mixin in jar.1.1[Environment::Both].iter() {
					println!("    {}", mixin);
				}
				if jar.1.1[Environment::Client].len() > 0 {
					println!("Client:");
					for mixin in jar.1.1[Environment::Client].iter() {
						println!("    {}", mixin);
					}
				}
				if jar.1.1[Environment::Server].len() > 0 {
					println!("Server:");
					for mixin in jar.1.1[Environment::Server].iter() {
						println!("    {}", mixin);
					}
				}
			}
			if !matched_jars {
				if mixin_cmd.filter.is_some() {
					println!("No jars that match the given filter found!");
				} else {
					println!("No valid jars found!");
				}
			}
		},
		SubCommand::JarInJar(jar_in_jar) => {
			if jar_in_jar.reverse {
				let mut reverse_tree: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
				
				fn build_recurse(jar: TraversedJar, file_name: &str, parent: Option<&str>, tree: &mut BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)>) {
					match jar {
						TraversedJar::NonMod => {}
						TraversedJar::FabricJar{mod_id, contained_jars, ..} => {
							let entry = tree.entry(mod_id.clone()).or_insert((BTreeSet::new(), BTreeSet::new()));

							entry.0.insert(file_name.to_string());
							if let Some(parent) = parent {
								entry.1.insert(parent.to_owned());
							}
							for jar in contained_jars {
								build_recurse(jar.1, jar.0.as_str(), Some(mod_id.as_str()), tree);
							}
						}
					}
				}

				fn print_recurse(id: &str, tree: &BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)>, padding: usize) {
					let mod_data = &tree[id];

					// Don't print on first level if it has no children
					if padding == 0 && mod_data.1.len() == 0 {
						return;
					}

					println!("{}{} ({})", "    ".repeat(padding), id, (&mod_data.0).into_iter().cloned().collect::<Vec<_>>().join(", "));
					for parent_id in &mod_data.1 {
						print_recurse(&parent_id, tree, padding + 1);
					}
				}

				for jar in processed_jars {
					build_recurse(jar.1, jar.0.file_name().map(|f| f.to_str().unwrap()).unwrap_or(jar.0.to_str().unwrap()), None, &mut reverse_tree);
				}
	
				for jar in &reverse_tree {
					if let Some(ref filter) = jar_in_jar.filter {
						if !jar.0.to_lowercase().contains(filter.to_lowercase().as_str()) {
							continue;
						}
					}
					print_recurse(&jar.0, &reverse_tree, 0);
				}
			} else {
				fn print_recurse(jar: TraversedJar, name: &str, padding: usize) {
					match jar {
						TraversedJar::NonMod => {
							println!("{}{} (Not a mod)", "    ".repeat(padding), name);
						}
						TraversedJar::FabricJar{mod_id, contained_jars, ..} => {
							println!("{}{} ({})", "    ".repeat(padding), mod_id, name);
							for jar in contained_jars {
								print_recurse(jar.1, jar.0.as_str(), padding + 1);
							}
						}
					}
				}
	
				for jar in processed_jars {
					if let Some(ref filter) = jar_in_jar.filter {
						if let TraversedJar::FabricJar{mod_id, ..} = &jar.1 {
							if !mod_id.to_lowercase().contains(filter.to_lowercase().as_str()) {
								continue;
							}
						}
					}
					print_recurse(jar.1, jar.0.file_name().map(|f| f.to_str().unwrap()).unwrap_or(jar.0.to_str().unwrap()), 0);
				}
			}
		}
		SubCommand::Raw(_raw) => {
			for jar in processed_jars {
				println!("{} {:#?}", jar.0.file_name().map(|f| f.to_str().unwrap()).unwrap_or(jar.0.to_str().unwrap()), jar.1);
			}
		}
	}
	
	Ok(())
}
