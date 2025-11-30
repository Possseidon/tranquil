use std::collections::BTreeMap;

enum Command {
    Command {
        localization: Localization,
        options: CommandOptions,
    },
    SubCommands(BTreeMap<String, SubCommand>),
}

enum SubCommand {
    Command {
        localization: Localization,
        options: CommandOptions,
    },
    Group(BTreeMap<String, CommandOptions>),
}

struct CommandOptions(Vec<CommandOption>);

struct CommandOption {
    autocomplete: bool,
}
